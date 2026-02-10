use crate::model::{Account, Transaction, TransactionType};

/// Applies a deposit transaction to an account, increasing its available funds.
///
/// # Panics
///
/// Panics if `tx.type` is not [`TransactionType::Deposit`].
pub fn deposit(account: &mut Account, tx: &Transaction) {
    if !matches!(tx.r#type, TransactionType::Deposit) {
        panic!("failed to deposit transaction: {tx:?}");
    }

    account.available += tx.amount;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_deposit(client: u16, tx: u32, amount: f64) -> Transaction {
        Transaction {
            r#type: TransactionType::Deposit,
            client,
            tx,
            amount,
        }
    }

    #[test]
    fn deposit_increases_available() {
        let mut account = Account::new(1);
        deposit(&mut account, &make_deposit(1, 1, 100.0));
        assert_eq!(account.available, 100.0);
    }

    #[test]
    fn deposit_does_not_affect_held() {
        let mut account = Account::new(1);
        deposit(&mut account, &make_deposit(1, 1, 50.0));
        assert_eq!(account.held, 0.0);
    }

    #[test]
    fn deposit_total_equals_available_when_no_held() {
        let mut account = Account::new(1);
        deposit(&mut account, &make_deposit(1, 1, 75.0));
        assert_eq!(account.total(), account.available);
    }

    #[test]
    fn multiple_deposits_accumulate() {
        let mut account = Account::new(1);
        deposit(&mut account, &make_deposit(1, 1, 1.0));
        deposit(&mut account, &make_deposit(1, 2, 2.0));
        deposit(&mut account, &make_deposit(1, 3, 3.0));
        assert_eq!(account.available, 6.0);
    }

    #[test]
    #[should_panic]
    fn deposit_panics_on_wrong_type() {
        let mut account = Account::new(1);
        let tx = Transaction {
            r#type: TransactionType::Withdrawal,
            client: 1,
            tx: 1,
            amount: 10.0,
        };
        deposit(&mut account, &tx);
    }
}
