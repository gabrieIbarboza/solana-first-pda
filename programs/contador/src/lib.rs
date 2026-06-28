use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod error;
pub mod constants;

pub use instructions::*;

declare_id!("4iVmETSsCe6t3srUPBCdGwJgojFwSA5fYwz6ZDWXfHym");

#[program]
pub mod contador {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        crate::instructions::initialize::handle_initialize(ctx)
    }

    pub fn increment(ctx: Context<Increment>) -> Result<()> {
        crate::instructions::increment::handle_increment(ctx)
    }
}
