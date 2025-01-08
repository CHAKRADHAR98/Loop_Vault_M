use anchor_lang::prelude::*;

#[derive(Accounts)]
    pub struct DisburseFunds<'info> {
        #[account(
            mut,
            seeds = [b"chit_fund", chit_fund.creator.as_ref()],
            bump
        )]
        pub chit_fund: Account<'info, ChitFund>,

        /// CHECK: We manually verify the token account
        #[account(mut)]
        pub contribution_vault: AccountInfo<'info>,

        /// CHECK: We manually verify the token account
        #[account(mut)]
        pub borrower_token_account: AccountInfo<'info>,

        #[account(mut, has_one = chit_fund)]
        pub borrower: Account<'info, Participant>,

        pub token_program: Program<'info, Token>,
    }