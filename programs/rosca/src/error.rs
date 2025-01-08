use anchor_lang::prelude::*;

#[error_code]
pub enum ChitFundError {
    #[msg("Maximum number of participants reached.")]
    MaxParticipantsReached,
    #[msg("Insufficient collateral provided.")]
    InsufficientCollateral,
    #[msg("Contribution for this cycle has already been made.")]
    ContributionAlreadyMade,
    #[msg("Not all participants have made their contributions.")]
    PendingContributions,
    #[msg("Participant not found.")]
    ParticipantNotFound,
    #[msg("Invalid borrower account.")]
    InvalidBorrowerAccount,
    #[msg("The chit fund is not active.")]
    ChitFundInactive,
    #[msg("The chit fund is still active.")]
    ChitFundActive,
    #[msg("The cycle is not yet complete.")]
    CycleNotComplete,
    #[msg("Invalid Collateral Mint.")]
    InvalidCollateralMint,
    #[msg("Invalid Collateral Vault Owner.")]
    InvalidCollateralVaultOwner,
    #[msg("Invalid Contribution Mint.")]
    InvalidContributionMint,
    #[msg("Invalid Contribution Vault Owner.")]
    InvalidContributionVaultOwner,
    #[msg("Exceeds the maximum number of cycles.")]
    ExceedsMaximumCycles,
    #[msg("Invalid cycle for disbursement.")]
    InvalidCycle,
    #[msg("Participant has already borrowed.")]
    AlreadyBorrowed,
    #[msg("Unauthorized access.")]
    Unauthorized,
    #[msg("Invalid disbursement schedule.")]
    InvalidDisbursementSchedule,
    #[msg("Exceeds the maximum number of participants allowed.")]
    ExceedsMaximumParticipants,
    #[msg("Total disbursement exceeds total contributions")]
    InvalidDisbursementTotal,
    #[msg("Cycle duration is too short, minimum duration required")]
    CycleDurationTooShort,
    #[msg("Invalid cycle duration")]
    InvalidCycleDuration,
}
