use crate::{
    model::{Account, Transaction, TransactionType},
    rules::RuleError,
};

/// Applies a withdrawal transaction to an account, decreasing its available funds.
///
/// # Errors
///
/// Returns [`RuleError::InsuficientFunds`] if `account.available` is less than `tx.amount`.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Withdrawal`].
pub fn withdrawal(account: &mut Account, tx: &Transaction) -> Result<(), RuleError> {
    if !matches!(tx.r#type, TransactionType::Withdrawal) {
        panic!("failed to withdraw transaction: {tx:?}");
    }

    if account.available < tx.amount {
        return Err(RuleError::InsuficientFunds);
    }

    account.available -= tx.amount;

    Ok(())
}
