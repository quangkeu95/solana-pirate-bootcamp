use anchor_lang::prelude::*;

declare_id!("GAWy29t3Z2JrKQ8Rt3RjE1hyEXsxvhdAJvWFyKvwjTdN");

#[program]
pub mod anchor_fundamental {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
