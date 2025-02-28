use anchor_lang::prelude::*;

use crate::state::LoanRegistryState;


#[derive(Accounts)]
pub struct CreateLoanRegistry<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = 8 + LoanRegistryState::INIT_SPACE,
        seeds = [b"loan_registry"],
        bump
    )]

    pub loan_registry: Box<Account<'info, LoanRegistryState>>,

    //program
    pub system_program: Program<'info, System>
}

impl<'info> CreateLoanRegistry<'info> {

    pub fn create_loan_registry(&mut self) -> Result<()> {

        //creating loan registry
        self.loan_registry.set_inner(LoanRegistryState{
            total_loans: 0,
            loan_requests: Vec::new()
        });

        Ok(())
    }
}