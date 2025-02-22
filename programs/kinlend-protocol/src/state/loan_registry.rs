use anchor_lang::prelude::*;

// The LoanRegistry account is a public directory of active loan requests.
#[account]
#[derive(InitSpace)]
pub struct LoanRegistryState {
    #[max_len(10)]
    pub first_page: Option<Pubkey>, //Pointer to first LoanRegistryStatePage
    pub total_loans: u64, //tracking total numbers of active loan requests.
}