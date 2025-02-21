use anchor_lang::prelude::*;

// The LoanRegistry account is a public directory of active loan requests.
#[account]
pub struct LoanRegistryState {
    pub loan_requests: Vec<Pubkey>, // List of active loan request PDAs.
}