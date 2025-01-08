use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenInterface;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::state::ChitFund;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeChitFund<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = creator,
        space = 8 + ChitFund::INIT_SPACE,
        seeds = [mint.key().as_ref()],
        bump,
    )]
    pub chit_fund: Account<'info, ChitFund>,

    #[account(
        init,
        token::mint = mint,
        token::authority = contribution_vault,
        payer = creator,
        seeds = [b"contribution_vault", mint.key().as_ref()],
        bump,

    )]
    pub contribution_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        token::mint = mint,
        token::authority = collateral_vault,
        payer = creator,
        seeds = [b"collateral_vault", mint.key().as_ref()],
        bump

    )]
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}


pub fn initialize_chit_fund(
    ctx: Context<InitializeChitFund>,
    contribution_amount: u64,
    cycle_duration: i64,
    total_cycles: u8,
    collateral_requirement: u64,
    max_participants: u8,
    disbursement_schedule: [u64; MAX_CYCLES],
) -> Result<()> {
    // Validations
    require!(
        total_cycles as usize <= MAX_CYCLES,
        ChitFundError::ExceedsMaximumCycles
    );
    require!(
        cycle_duration >= MIN_CYCLE_DURATION,
        ChitFundError::InvalidCycleDuration
    );
    require!(
        max_participants as usize <= MAX_PARTICIPANTS,
        ChitFundError::ExceedsMaximumParticipants
    );

    let chit_fund = &mut ctx.accounts.chit_fund;
    
    // Admin/Config data
    chit_fund.creator = ctx.accounts.creator.key();
    chit_fund.mint_address = ctx.accounts.mint.key();
    chit_fund.contribution_amount = contribution_amount;
    chit_fund.cycle_duration = cycle_duration;
    chit_fund.total_cycles = total_cycles;
    chit_fund.collateral_requirement = collateral_requirement;
    chit_fund.max_participants = max_participants;

    // State tracking
    chit_fund.current_cycle = 0;
    chit_fund.is_active = true;
    chit_fund.last_disbursement_time = Clock::get()?.unix_timestamp;

    // Participant tracking
    chit_fund.participants = [Pubkey::default(); MAX_PARTICIPANTS];
    chit_fund.participants_count = 0;

    // Financial tracking
    chit_fund.disbursement_schedule = disbursement_schedule;
    chit_fund.contribution_vault = ctx.accounts.contribution_vault.key();
    chit_fund.collateral_vault = ctx.accounts.collateral_vault.key();
    chit_fund.total_contribution_amount = 0;

    emit!(ChitFundInitialized {
        chit_fund: chit_fund.key(),
        creator: chit_fund.creator,
        contribution_amount,
        total_cycles,
        max_participants,
        mint_address: chit_fund.mint_address,
    });

    Ok(())
}


#[event]
pub struct ChitFundInitialized {
    pub chit_fund: Pubkey,
    pub creator: Pubkey,
    pub contribution_amount: u64,
    pub total_cycles: u8,
    pub max_participants: u8,
    pub mint_address: Pubkey,
}