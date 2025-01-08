use anchor_lang::prelude::*;
use instructions::*;
use constants::*;

mod state;
mod instructions;
mod constants;
mod error;

declare_id!("6AQ26gghMkL77dWnoGhxX5iW1sU13zKVA4yy1fs8C4qr");

#[program]
pub mod rosca {
    use super::*;

    pub fn init_chit_fund(ctx: Context<InitializeChitFund>, contribution_amount: u64, cycle_duration: i64, total_cycles: u8, collateral_requirement: u64, max_participants: u8, disbursement_schedule: [u64; MAX_CYCLES]) -> Result<()> {
        initialize_chit_fund(ctx, contribution_amount, cycle_duration, total_cycles, collateral_requirement, max_participants, disbursement_schedule)
    }

    

}


