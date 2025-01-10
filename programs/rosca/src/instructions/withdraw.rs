use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{self,Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::{ChitFund, Participant};
use crate::error::*;
#[derive(Accounts)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub user: Signer<'info>, 

    pub mint: InterfaceAccount<'info, Mint>,
 
    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
        constraint = !chit_fund.is_active @ ChitFundError::ChitFundActive, // Can only withdraw after chit fund ends
    )]
    pub chit_fund: Box<Account<'info, ChitFund>>,

    #[account(
        mut,
        seeds = [b"collateral_vault", mint.key().as_ref()],
        bump,
        constraint = collateral_vault.mint == mint.key() @ ChitFundError::InvalidContributionMint,
    )]
    pub collateral_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut, 
        seeds = [user.key().as_ref()],
        bump,
        constraint = participant.owner == user.key() @ ChitFundError::Unauthorized,
        constraint = participant.has_borrowed @ ChitFundError::WithdrawBeforeBorrowing, // Can only withdraw after borrowing
    )]
    pub participant: Account<'info, Participant>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn withdraw_collateral(ctx: Context<WithdrawCollateral>) -> Result<()> {
    let chit_fund = &mut ctx.accounts.chit_fund;
    let participant = &mut ctx.accounts.participant;

    // Transfer funds
    let transfer_cpi_accounts = TransferChecked {
        from: ctx.accounts.collateral_vault.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.collateral_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let mint_key = ctx.accounts.mint.key();

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"collateral_vault",
        mint_key.as_ref(),
        &[ctx.bumps.collateral_vault]
    ]];

    let cpi_ctx = CpiContext::new(
        cpi_program,
        transfer_cpi_accounts,
    ).with_signer(signer_seeds);

    let decimals = ctx.accounts.mint.decimals;

    token_interface::transfer_checked(cpi_ctx, chit_fund.collateral_requirement, decimals)?;

    // Update token amounts based on mint verification
    match ctx.accounts.mint.key() {
        key if key == participant.usdc_address => {
            chit_fund.total_contribution_amount -= chit_fund.collateral_requirement;
            participant.total_contributed -= chit_fund.collateral_requirement;
        }
        _ => return Err(ChitFundError::InvalidContributionMint.into())
    }

    emit!(CollateralWithdrawn {
        chit_fund: chit_fund.key(),
        participant: participant.key(),
        amount: chit_fund.collateral_requirement,
        withdraw_time: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

#[event]
pub struct CollateralWithdrawn {
    pub chit_fund: Pubkey,
    pub participant: Pubkey,
    pub amount: u64,
    pub withdraw_time: i64,
}
