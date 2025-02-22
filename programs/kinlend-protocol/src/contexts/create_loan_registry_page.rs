use anchor_lang::prelude::*;

use crate::state::{LoanRegistryPageState, LoanRegistryState};

#[derive(Accounts)]
pub struct CreateLoanRegistryPage<'info> {
    
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump 
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    #[account(
        init,
        payer = admin,
        space = 8 + LoanRegistryPageState::INIT_SPACE,
        seeds = [b"loan_registry_page", loan_registry.key().as_ref()],
        bump
    )]
    pub loan_registry_page: Box<Account<'info, LoanRegistryPageState>>,

    pub system_program: Program<'info, System>,
}


