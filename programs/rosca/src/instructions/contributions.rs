use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::{ChitFund, Participant};
use crate::error::*;

#[derive(Accounts)]
pub struct MakeContribution<'info> {
    #[account(mut)]
    pub user: Signer<'info>,    

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
        constraint = chit_fund.is_active @ ChitFundError::ChitFundInactive,
    )]
    pub chit_fund: Account<'info, ChitFund>,

    #[account(
        mut,
        seeds = [b"contribution_vault", mint.key().as_ref()],
        bump,
        constraint = contribution_vault.mint == mint.key() @ ChitFundError::InvalidContributionMint,
    )]
    pub contribution_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut, 
        seeds = [user.key().as_ref()],
        bump,
        constraint = participant.owner == user.key() @ ChitFundError::Unauthorized,
        constraint = !participant.contributions[chit_fund.current_cycle as usize] @ ChitFundError::ContributionAlreadyMade,
    )]
    pub participant: Account<'info, Participant>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
        constraint = user_token_account.amount >= chit_fund.contribution_amount @ ChitFundError::InsufficientFunds,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
pub fn make_contribution(ctx: Context<MakeContribution>) -> Result<()> {
    let chit_fund = &mut ctx.accounts.chit_fund;
    let participant = &mut ctx.accounts.participant;
    
    // Transfer contribution amount
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.contribution_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(
        cpi_program,
        transfer_cpi_accounts
    );

    let decimals = ctx.accounts.mint.decimals;
    token_interface::transfer_checked(cpi_ctx, chit_fund.contribution_amount, decimals)?;

    // Update state after successful transfer
    participant.contributions[chit_fund.current_cycle as usize] = true;
    participant.last_contribution_time = Clock::get()?.unix_timestamp;
    
    // Update chit fund state


    // Verify mint and update state
    match ctx.accounts.mint.to_account_info().key() {
        key if key == participant.usdc_address => {
            chit_fund.total_contribution_amount += chit_fund.contribution_amount;
            participant.total_contributed += chit_fund.contribution_amount;
        }
        _ => return Err(ChitFundError::InvalidContributionMint.into())
    }

    emit!(ContributionMade {
        chit_fund: chit_fund.key(),
        participant: participant.key(),
        cycle: chit_fund.current_cycle,
        amount: chit_fund.contribution_amount,
        contribution_time: participant.last_contribution_time,
    });

    Ok(())
}

#[event]
pub struct ContributionMade {
    pub chit_fund: Pubkey,           // The chit fund being contributed to
    pub participant: Pubkey,         // The participant making the contribution
    pub cycle: u8,                   // Current cycle number
    pub amount: u64,                 // Contribution amount
    pub contribution_time: i64,      // When the contribution was made
}