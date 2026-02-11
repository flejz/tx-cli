use rust_decimal::{Decimal, RoundingStrategy};

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

fn deserialize_amount_4_dp<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = <Decimal as serde::Deserialize>::deserialize(deserializer)?;
    Ok(value.round_dp_with_strategy(4, RoundingStrategy::ToZero))
}

#[derive(Debug, serde::Deserialize)]
pub struct Transaction {
    pub r#type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "deserialize_amount_4_dp")]
    pub amount: Decimal,
}
