use anchor_lang::prelude::*;

declare_id!("CqzdqFZSNhvPUjPUKT141iQNvBcUzMjRgmWJ6MTWF21c");

pub mod contexts;
pub mod state;
pub mod errors;
pub mod helpers;

use crate::contexts::CreateLoanRequest;

#[program]
pub mod kinlend_protocol {


    use contexts::{CancelLoanRequest, CreateLoanRegistry, FundLoan, InitConfig, RepayLoan, UpdateConfig};

    use super::*;

    //Instruction for creating loan request by borrower
    pub fn create_loan_request(
        ctx:Context<CreateLoanRequest>, 
        loan_id:u64, 
        loan_amount:u64, 
        collateral:u64,
        duration_days:u64

    ) -> Result<()> {

        ctx.accounts.create_loan_request(
            loan_id, 
            loan_amount, 
            collateral,
            duration_days,
            ctx.bumps
        )


    }


    //Instruction for cancelling Loan request by borrower
    pub fn cancel_loan_request(ctx:Context<CancelLoanRequest>) -> Result<()> {
        ctx.accounts.cancel_loan_request()
    }

    //instruction for funding loan by by lender
    pub fn fund_loan(ctx:Context<FundLoan>) -> Result<()> {
        ctx.accounts.fund_loan()
    }


    //instruction for repaying loan by borrower
    pub fn repay_loan(ctx: Context<RepayLoan>) -> Result<()> {
        ctx.accounts.repay_loan()
    }

    // ADMIN ONLY: instruction for configuring usdc mint key
    pub fn init_config(ctx:Context<InitConfig>) -> Result<()> {
        ctx.accounts.init_config()
    }

    //ADMIN ONLY: instruction for updating usdc mint key 
    pub fn update_config(ctx:Context<UpdateConfig>) -> Result<()> {
        ctx.accounts.update_config()
    }

    //Instruction for creating LoanRegistry which store Loan Requests
    pub fn create_loan_registry(ctx:Context<CreateLoanRegistry>) -> Result<()> {
        ctx.accounts.create_loan_registry()
    }

    

}
