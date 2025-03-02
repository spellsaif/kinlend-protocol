use anchor_lang::prelude::*;

// The LoanRegistry account is a public directory of active loan requests.
#[account]
#[derive(InitSpace)]
pub struct LoanRegistryState {
    #[max_len(20)]
    pub loan_requests: Vec<Pubkey>, //stores loan request pubkeys
    pub total_loans: u64, //tracking total numbers of active loan requests.
}