use anchor_lang::prelude::*;

use crate::state::{CollateralVaultState, LoanRegistryState, LoanRequestState};

#[derive(Accounts)]
pub struct ClaimCollateral<'info> {

    //Lender will be the one who will be able to claim collateral if borrower fails to repay
    //Lender is signer here
    #[account(mut)]
    pub lender: Signer<'info>,

    //Loan Request Account
    #[account(
        mut,
        seeds = [b"loan_request", loan_request.borrower.as_ref(), &loan_request.loan_id.to_le_bytes()],
        bump
    )]
    pub loan_request: Box<Account<'info, LoanRequestState>>,

    //Collateral Vault
    #[account(
        mut,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Account<'info, CollateralVaultState>,

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump,
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    //program
    pub system_program: Program<'info, System>
}