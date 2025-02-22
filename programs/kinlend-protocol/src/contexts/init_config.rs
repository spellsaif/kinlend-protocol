use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::ConfigState;


#[derive(Accounts)]
pub struct InitConfig<'info>{

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init, 
        payer = admin,
        space = 8 + ConfigState::INIT_SPACE,
        seeds = [b"config"],
        bump

    )]
    pub config: Box<Account<'info, ConfigState>>,

    pub usdc_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>
}

impl<'info> InitConfig<'info> {

    pub fn init_config(&mut self) -> Result<()> {
        self.config.usdc_mint = self.usdc_mint.key();
        self.config.authority = self.admin.key();

        Ok(())
    }
}