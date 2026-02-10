mod deposit;
mod dispute;
mod resolve;
mod withdrawal;

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("insuficient funds")]
    InsuficientFunds,

    #[error("transaction not found: {0}")]
    TrasactionNotFound(u32),
}

pub use deposit::*;
pub use dispute::*;
pub use resolve::*;
pub use withdrawal::*;
