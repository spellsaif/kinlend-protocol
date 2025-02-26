use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::ConfigState;
use crate::errors::ErrorCode;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump
    )]
    pub config: Box<Account<'info, ConfigState>>,

    pub new_usdc_mint: Account<'info, Mint>,
    
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateConfig<'info> {
    pub fn update_config(&mut self) -> Result<()> {
        //checking if the signer is admin
        let signer = self.admin.key();
        self.check_is_admin(signer)?;

        //update config
        let new_usdc_mint_address = self.new_usdc_mint.key();
        self.config.usdc_mint = new_usdc_mint_address;

        Ok(())
    }

    pub fn check_is_admin(&self, signer_key: Pubkey) -> Result<()> {
        if signer_key != self.config.authority {
            return Err(ErrorCode::NotAdmin.into());
        }

        Ok(())
    }
}