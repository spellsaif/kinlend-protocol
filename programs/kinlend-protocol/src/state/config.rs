use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ConfigState{
    pub usdc_mint: Pubkey, //stores correcr usdc mint address
    pub authority: Pubkey, //stores wallet address of admin
}