use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::state::ConfigState;

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