use anchor_lang::prelude::*;

declare_id!("CqzdqFZSNhvPUjPUKT141iQNvBcUzMjRgmWJ6MTWF21c");

mod contexts;
mod state;
mod errors;

#[program]
pub mod kinlend_protocol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
