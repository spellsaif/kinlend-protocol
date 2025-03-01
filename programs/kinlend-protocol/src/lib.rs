use anchor_lang::prelude::*;

declare_id!("GJSshH3kYxm6JL9vqgUwiczFDE1tRxaZZ47VUohkjCVu");

pub mod errors;
pub mod state;
pub mod helpers;
pub mod contexts;
pub mod constants;

pub use contexts::*;

#[program]
pub mod kinlend_protocol {



    use super::*;

    //Instruction for creating loan request by borrower
    pub fn create_loan_request(
        ctx:Context<CreateLoanRequest>, 
        loan_id:u64, 
        loan_amount:u64, 
        collateral:u64,
        duration_days:u64,
        sol_price: u64,

    ) -> Result<()> {

        ctx.accounts.create_loan_request(
            loan_id, 
            loan_amount, 
            collateral,
            duration_days,
            sol_price,
            ctx.bumps
        )


    }


    //Instruction for cancelling Loan request by borrower
    pub fn cancel_loan_request(ctx:Context<CancelLoanRequest>) -> Result<()> {
        ctx.accounts.cancel_loan_request()
    }

    //Instruction for funding loan by by lender
    pub fn fund_loan(ctx:Context<FundLoan>, _loan_id:u64) -> Result<()> {
        ctx.accounts.fund_loan()
    }


    //Instruction for repaying loan by borrower
    pub fn repay_loan(ctx: Context<RepayLoan>, _loan_id:u64) -> Result<()> {
        ctx.accounts.repay_loan()
    }

    //Instruction for claiming collateral by Lender if Borrower fails to repay
    pub fn claim_collateral(ctx:Context<ClaimCollateral>) -> Result<()> {
        ctx.accounts.claim_collateral()
    }

    //Instruction for liquidating Loan
    pub fn liquidate_loan(ctx: Context<LiquidateLoan>, sol_price: u64) -> Result<()> {
        ctx.accounts.liquidate_loan(sol_price)
    }

    //Instruction for creating Protocol Vault which for receiving SOL as FEE
    pub fn create_protocol_vault(ctx:Context<CreateProtocolVault>) -> Result<()> {
        ctx.accounts.create_protocol_vault(ctx.bumps)
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
