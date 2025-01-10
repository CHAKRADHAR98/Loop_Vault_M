use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self,Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::{ChitFund, Participant};
use crate::error::*;


#[derive(Accounts)]
pub struct DisburseFunds<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
        constraint = chit_fund.is_active @ ChitFundError::ChitFundInactive,
        constraint = Clock::get()?.unix_timestamp >= chit_fund.last_disbursement_time + chit_fund.cycle_duration @ ChitFundError::CycleNotComplete,
    )]
    pub chit_fund: Box<Account<'info, ChitFund>>,
    
    #[account(
        mut,
        seeds = [b"contribution_vault", mint.key().as_ref()],
        bump,
        constraint = contribution_vault.mint == mint.key() @ ChitFundError::InvalidContributionMint,
    )]
    pub contribution_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut, 
        seeds = [user.key().as_ref()],
        bump,
        constraint = participant.owner == user.key() @ ChitFundError::Unauthorized,
        constraint = !participant.has_borrowed @ ChitFundError::AlreadyBorrowed,
    )]
    pub participant: Box<Account<'info, Participant>>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn disburse_funds(ctx: Context<DisburseFunds>) -> Result<()> {
    let chit_fund = &mut ctx.accounts.chit_fund;
    let participant = &mut ctx.accounts.participant;
    
    let current_time = Clock::get()?.unix_timestamp;
    
    // Check if there are any participants
    require!(
        chit_fund.participants_count > 0, 
        ChitFundError::ParticipantNotFound  
    );

    // Create vector to store eligible borrowers
    let mut eligible_borrowers: Vec<Pubkey> = Vec::new();
    
    // Check each participant in the chit fund
    for i in 0..chit_fund.participants_count as usize {
        let current_pubkey = chit_fund.participants[i];
        
        // Add to eligible list if they haven't borrowed yet
        if current_pubkey != Pubkey::default() && !chit_fund.borrowed_participants[i] {
            eligible_borrowers.push(current_pubkey);
        }
    }

    // Ensure there are eligible borrowers
    require!(
        !eligible_borrowers.is_empty(),
        ChitFundError::NoEligibleBorrowers
    );

    // Select random borrower using current timestamp
    let random_index = current_time as usize % eligible_borrowers.len();
    let selected_borrower = eligible_borrowers[random_index];

    // Verify selected borrower
    require!(
        participant.owner == selected_borrower,
        ChitFundError::InvalidBorrowerAccount
    );

    // Get disbursement amount
    let disbursement_amount = chit_fund.disbursement_schedule[chit_fund.current_cycle as usize];

    // Transfer funds
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.contribution_vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.contribution_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let mint_key = ctx.accounts.mint.key();

    let signer_seeds: &[&[&[u8]]] = &[
        &[
            b"contribution_vault",
            mint_key.as_ref(),
            &[ctx.bumps.contribution_vault]
            ]];

    let cpi_ctx = CpiContext::new(cpi_program,transfer_cpi_accounts).with_signer(signer_seeds);

    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, disbursement_amount, decimals)?;

    // Update borrowed status in chit fund
    for i in 0..chit_fund.participants_count as usize {
        if chit_fund.participants[i] == participant.owner {
            chit_fund.borrowed_participants[i] = true;
            break;
        }
    }

    // Update participant state
    participant.has_borrowed = true;
    participant.borrowed_cycle = Some(chit_fund.current_cycle);

    // Update chit fund state
    chit_fund.current_cycle += 1;
    chit_fund.last_disbursement_time = current_time;

    // Check if chit fund is complete
    if chit_fund.current_cycle == chit_fund.total_cycles {
        chit_fund.is_active = false;
    }

    // Update token amounts based on mint verification
    match ctx.accounts.mint.key() {
        key if key == participant.usdc_address => {
            chit_fund.total_contribution_amount -= disbursement_amount;
            participant.total_contributed -= disbursement_amount;
        }
        _ => return Err(ChitFundError::InvalidContributionMint.into())
    }

    emit!(FundsDisbursed {
        chit_fund: chit_fund.key(),
        participant: participant.key(),
        amount: disbursement_amount,
        cycle: chit_fund.current_cycle - 1,
        disbursement_time: current_time,
    });

    Ok(())
}

#[event]
pub struct FundsDisbursed {
    pub chit_fund: Pubkey,
    pub participant: Pubkey,
    pub amount: u64,
    pub cycle: u8,
    pub disbursement_time: i64,
}