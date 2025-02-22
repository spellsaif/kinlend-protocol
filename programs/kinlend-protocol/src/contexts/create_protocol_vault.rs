use anchor_lang::prelude::*;

use crate::state::ProtocolVaultState;

#[derive(Accounts)]
pub struct CreateProtocolVault<'info> {
    
    #[account(mut)]
    admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = ProtocolVaultState::INIT_SPACE,
        seeds = [b"protocol_vault"],
        bump
    )]
    pub protocol_vault: Box<Account<'info, ProtocolVaultState>>,

    pub system_program: Program<'info, System>

}

impl<'info> CreateProtocolVault<'info> {
    pub fn create_protocol_vault(&mut self, bumps: CreateProtocolVaultBumps) -> Result<()> {
        //creating protocol vault
        self.protocol_vault.set_inner(ProtocolVaultState{
            bump: bumps.protocol_vault
        });

        Ok(())
    }
}