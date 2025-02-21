use anchor_lang::prelude::*;

// The LoanRegistry account is a public directory of active loan requests.
#[account]
#[derive(InitSpace)]
pub struct LoanRegistryState {
    #[max_len(10)]
    pub loan_requests: Vec<Pubkey>, // List of active loan request PDAs.
}