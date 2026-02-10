mod deposit;
mod withdrawal;

#[derive(Debug, thiserror::Error)]
pub enum RuleError {
    #[error("insuficient funds")]
    InsuficientFunds,
}

pub use deposit::*;
pub use withdrawal::*;
