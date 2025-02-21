use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LoanRequestState {
    pub loan_id: u64,
    pub borrower: Pubkey,       // Borrower's wallet address
    pub lender: Option<Pubkey>, // Address of the lender
    pub loan_amount: u64,       // Desired loan amount in USDC
    pub collateral: u64,        // Collateral in SOL (must be at least 150% of loan amount in USDC value)
    pub duration_days: u64,     // Loan duration in days (set by borrower)
    pub repayment_time: i64,    // Unix Timestamp when the lender funds the loan
}