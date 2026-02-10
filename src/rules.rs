mod chargeback;
mod deposit;
mod dispute;
mod resolve;
mod withdrawal;

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("insufficient funds")]
    InsuficientFunds,

    #[error("deposit not found: {0}")]
    DepositNotFound(u32),

    #[error("transaction not being disputed: {0}")]
    TrasactionNotOnDispute(u32),
}

pub use chargeback::*;
pub use deposit::*;
pub use dispute::*;
pub use resolve::*;
pub use withdrawal::*;
