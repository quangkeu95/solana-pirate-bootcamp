use anchor_lang::prelude::*;

declare_id!("ByLuugnVw2LSxdZDapEe9NMXkNeoZR2KyhDMnF4QaXgD");

#[program]
pub mod token_minter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, metadata: Metadata) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[account]
#[derive(Default)]
pub struct Metadata {
    name: String,
    symbol: String,
    decimal: u16,
}
