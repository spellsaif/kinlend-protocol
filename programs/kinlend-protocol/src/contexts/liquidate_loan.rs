use anchor_lang::prelude::*;

use crate::state::{CollateralVaultState, LoanRegistryState, ProtocolVaultState};

#[derive(Accounts)]
pub struct LiquidateLoan<'info> {

    //signers
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(
        mut,
        close = borrower
    )]
    pub loan_request: Box<Account<'info, LoanRegistryState>>,

    #[account(
        mut,
        seeds = [b"collateral_vault", loan_request.key().as_ref()],
        bump = collateral_vault.bump
    )]
    pub collateral_vault: Box<Account<'info, CollateralVaultState>>,

    #[account(
        mut,
        seeds = [b"loan_registry"],
        bump 
    )]
    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    #[account(
        mut,
        seeds = [b"protocol_vault"],
        bump = protocol_vault.bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,


    #[account(
        address = "YOUR SOL PRICE FEED ADDRESS GOES HERE"
    )]
    pub sol_price_feed: AccountInfo<'info>,

    //program
    pub system_program: Program<'info, System>,

    
}