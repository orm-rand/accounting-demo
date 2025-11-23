pub type ClientId = u16;
pub type TransactionId = u32;

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, serde::Deserialize)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    pub action: Action,
    #[serde(rename(deserialize = "client"))]
    pub client_id: ClientId,
    #[serde(rename(deserialize = "tx"))]
    pub id: TransactionId,
    pub amount: Option<f64>,
}
