use anchor_lang::prelude::*;
use crate::constants::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct ChitFund {
    // Admin/Config data
    pub creator: Pubkey,
    pub mint_address: Pubkey,
    pub contribution_amount: u64,
    pub cycle_duration: i64,
    pub total_cycles: u8,        // Using u8 since MAX_CYCLES is 12
    pub collateral_requirement: u64,
    pub max_participants: u8,

    // State tracking
    pub current_cycle: u8,
    pub is_active: bool,
    pub last_disbursement_time: i64,
    
    // Participant tracking
    pub participants: [Pubkey; MAX_PARTICIPANTS],
    pub participants_count: u8,
    pub borrowed_participants: [bool; MAX_PARTICIPANTS],
    
    // Financial tracking
    pub disbursement_schedule: [u64; MAX_CYCLES],
    pub contribution_vault: Pubkey,    
    pub collateral_vault: Pubkey,      
    pub total_contribution_amount: u64, 
}

#[account]
#[derive(InitSpace, Debug)]
pub struct Participant {
    // Identity
    pub owner: Pubkey,
    pub chit_fund: Pubkey,
    pub usdc_address: Pubkey,
    
    // State tracking
    pub has_borrowed: bool,
    pub is_emergency_requested: bool,
    pub contributions: [bool; MAX_CYCLES],
    
    // Time tracking
    pub join_time: i64,               
    pub last_contribution_time: i64,   
    
    // Financial tracking
    pub total_contributed: u64,        
    pub borrowed_cycle: Option<u8>,    
}