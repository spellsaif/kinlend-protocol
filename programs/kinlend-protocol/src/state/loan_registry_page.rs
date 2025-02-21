use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LoanRegistryPageState {

    #[max_len(10)]
    pub loan_requests: Vec<Pubkey>,
    pub next_page: Option<Pubkey>
}