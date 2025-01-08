use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken,
     token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::state::{ChitFund, Participant};
use crate::constants::*;
use crate::error::*;

#[derive(Accounts)]
pub struct JoinChitFund<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [mint.key().as_ref()],
        bump,
        constraint = chit_fund.is_active @ ChitFundError::ChitFundInactive,
        constraint = chit_fund.participants_count < chit_fund.max_participants @ ChitFundError::MaxParticipantsReached,
    )]
    pub chit_fund: Account<'info, ChitFund>,


    #[account(
        mut,
        seeds = [b"collateral_vault", mint.key().as_ref()],
        bump,
        constraint = collateral_vault.mint == mint.key() @ ChitFundError::InvalidContributionMint,
    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = user,
        space = 8 + Participant::INIT_SPACE,
        seeds = [user.key().as_ref()],
        bump
    )]
    pub participant: Account<'info, Participant>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
        constraint = user_token_account.amount >= chit_fund.collateral_requirement @ ChitFundError::InsufficientCollateral,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,


    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,

}


pub fn join_chit_fund(ctx: Context<JoinChitFund>) -> Result<()> {
    let chit_fund = &mut ctx.accounts.chit_fund;
    let participant = &mut ctx.accounts.participant;

    // Update participant data
    // Identity
    participant.owner = ctx.accounts.user.key();
    participant.chit_fund = chit_fund.key();
    participant.usdc_address = ctx.accounts.user_token_account.key();

    // State tracking
    participant.has_borrowed = false;
    participant.is_emergency_requested = false;
    participant.contributions = [false; MAX_CYCLES];

    // Time tracking
    participant.join_time = Clock::get()?.unix_timestamp;
    participant.last_contribution_time = Clock::get()?.unix_timestamp;

    // Financial tracking
    participant.total_contributed = 0;
    participant.borrowed_cycle = None;

    // Update chit fund participants list
    chit_fund.participants[chit_fund.participants_count as usize] = ctx.accounts.user.key();
    chit_fund.participants_count += 1;

    // Transfer collateral
    let transfer_accounts = anchor_spl::token_interface::Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.collateral_vault.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_accounts,
    );
    anchor_spl::token_interface::transfer(cpi_ctx, chit_fund.collateral_requirement)?;

    emit!(ParticipantJoined {
        chit_fund: chit_fund.key(),
        participant: participant.key(),
        owner: participant.owner,
        join_time: participant.join_time,
    });

    Ok(())
}

#[event]
pub struct ParticipantJoined {
    pub chit_fund: Pubkey,
    pub participant: Pubkey,
    pub owner: Pubkey,
    pub join_time: i64,
}

