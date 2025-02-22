use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LoanRegistryPageState {

    #[max_len(10)]
    pub loan_requests: Vec<Pubkey>, //list of loan_request (addressess are stored)
    pub next_page: Option<Pubkey> //pointing to next page if available
}