use std::collections::HashMap;

use thiserror::Error;

use crate::account::{Account, AccountError};
use crate::types::{Action, ClientId, Transaction, TransactionId};

#[derive(Error, Debug, PartialEq)]
pub enum AccountManagerError {
    #[error("{0}")]
    Account(#[from] AccountError),

    #[error("Unauthorized. {client_id} can't modify transactions of {owner_id}.")]
    Unauthorized {
        client_id: ClientId,
        owner_id: ClientId,
    },

    #[error("Transaction {id} is not disputed")]
    Undisputed { id: TransactionId },

    #[error("Transaction {id} is already disputed")]
    AlreadyDisputed { id: TransactionId },

    #[error("Transaction {id} not found")]
    TransactionNotFound { id: TransactionId },
}

pub type AccountManagerResult<T> = Result<T, AccountManagerError>;

#[derive(Clone)]
struct TxCacheEntry {
    pub client_id: ClientId,
    pub amount: f64,
    pub disputed: bool,
}

impl TxCacheEntry {
    pub fn new(client_id: ClientId, amount: f64) -> Self {
        Self {
            client_id,
            amount,
            disputed: false,
        }
    }
}

fn check_authorization(tx: &TxCacheEntry, client_id: ClientId) -> AccountManagerResult<()> {
    if tx.client_id != client_id {
        return Err(AccountManagerError::Unauthorized {
            client_id,
            owner_id: tx.client_id,
        });
    }
    Ok(())
}

fn check_disputed(tx: &TxCacheEntry, id: TransactionId) -> AccountManagerResult<()> {
    if !tx.disputed {
        return Err(AccountManagerError::Undisputed { id });
    }
    Ok(())
}

fn check_undisputed(tx: &TxCacheEntry, id: TransactionId) -> AccountManagerResult<()> {
    if tx.disputed {
        return Err(AccountManagerError::AlreadyDisputed { id });
    }
    Ok(())
}

#[derive(Default)]
pub struct AccountManager {
    accounts: HashMap<ClientId, Account>,
    tx_cache: HashMap<TransactionId, TxCacheEntry>,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            tx_cache: HashMap::new(),
        }
    }

    pub fn accounts(&self) -> Vec<(ClientId, Account)> {
        self.accounts.clone().into_iter().collect()
    }

    pub fn deposit(&mut self, tx_id: TransactionId, client_id: ClientId, amount: f64) {
        self.accounts.entry(client_id).or_default().deposit(amount);
        self.tx_cache
            .insert(tx_id, TxCacheEntry::new(client_id, amount));
    }

    pub fn withdraw(&mut self, client_id: ClientId, amount: f64) -> AccountManagerResult<()> {
        Ok(self
            .accounts
            .entry(client_id)
            .or_default()
            .withdraw(amount)?)
    }

    pub fn dispute(
        &mut self,
        tx_id: TransactionId,
        client_id: ClientId,
    ) -> AccountManagerResult<()> {
        let tx = self
            .tx_cache
            .get_mut(&tx_id)
            .ok_or(AccountManagerError::TransactionNotFound { id: tx_id })?;
        check_authorization(tx, client_id)?;
        check_undisputed(tx, tx_id)?;

        let account = self.accounts.entry(client_id).or_default();
        tx.disputed = account.dispute(tx.amount).is_ok();
        Ok(())
    }

    pub fn resolve(
        &mut self,
        tx_id: TransactionId,
        client_id: ClientId,
    ) -> AccountManagerResult<()> {
        let tx = self
            .tx_cache
            .get_mut(&tx_id)
            .ok_or(AccountManagerError::TransactionNotFound { id: tx_id })?;
        check_authorization(tx, client_id)?;
        check_disputed(tx, tx_id)?;

        let account = self.accounts.entry(client_id).or_default();
        account.resolve(tx.amount);
        tx.disputed = false;
        Ok(())
    }

    pub fn chargeback(
        &mut self,
        tx_id: TransactionId,
        client_id: ClientId,
    ) -> AccountManagerResult<()> {
        let tx = self
            .tx_cache
            .get(&tx_id)
            .ok_or(AccountManagerError::TransactionNotFound { id: tx_id })?;
        check_authorization(tx, client_id)?;
        check_disputed(tx, tx_id)?;

        let account = self.accounts.entry(client_id).or_default();
        account.chargeback(tx.amount);
        self.tx_cache.remove(&tx_id);
        Ok(())
    }
}

pub fn process_transaction(
    account_manager: &mut AccountManager,
    tx: Transaction,
) -> AccountManagerResult<()> {
    match tx.action {
        Action::Deposit => {
            if let Some(amount) = tx.amount {
                account_manager.deposit(tx.id, tx.client_id, amount);
            }
            Ok(())
        }
        Action::Withdrawal => {
            if let Some(amount) = tx.amount {
                account_manager.withdraw(tx.client_id, amount)
            } else {
                Ok(())
            }
        }
        Action::Dispute => account_manager.dispute(tx.id, tx.client_id),
        Action::Resolve => account_manager.resolve(tx.id, tx.client_id),
        Action::Chargeback => account_manager.chargeback(tx.id, tx.client_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispute_fails_if_transaction_is_not_owned_by_client() {
        let mut account_manager = AccountManager::new();

        let tx_id = 2;
        let client_id = 1;
        let amount = 1.0;
        account_manager.deposit(tx_id, client_id, amount);

        let other_tx_id = 3;
        let other_client_id = 2;
        account_manager.deposit(other_tx_id, other_client_id, amount);

        assert!(account_manager.dispute(tx_id, other_client_id).is_err());

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0].1.available(), amount);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), 0.0);
        assert_eq!(accounts[1].1.available(), amount);
        assert_eq!(accounts[1].1.total(), amount);
        assert_eq!(accounts[1].1.disputed(), 0.0);
    }

    #[test]
    fn dispute_transaction() {
        let mut account_manager = AccountManager::new();

        let tx_id = 2;
        let client_id = 1;
        let amount = 1.0;
        account_manager.deposit(tx_id, client_id, amount);
        assert!(account_manager.dispute(tx_id, client_id).is_ok());

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].1.available(), 0.0);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), amount);
    }

    #[test]
    fn resolve_disputed_transaction() {
        let mut account_manager = AccountManager::new();

        let tx_id = 2;
        let client_id = 1;
        let amount = 1.0;
        account_manager.deposit(tx_id, client_id, amount);
        assert!(account_manager.dispute(tx_id, client_id).is_ok());
        assert!(account_manager.resolve(tx_id, client_id).is_ok());

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].1.available(), amount);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), 0.0);
    }

    #[test]
    fn resolution_fails_if_dispute_failed() {
        let mut account_manager = AccountManager::new();

        let tx_id1 = 2;
        let tx_id2 = 3;
        let client_id = 1;
        let amount = 1.0;
        account_manager.deposit(tx_id1, client_id, amount);
        assert!(account_manager.withdraw(client_id, amount).is_ok());
        account_manager.deposit(tx_id2, client_id, amount);
        let err = account_manager.resolve(tx_id1, client_id).unwrap_err();
        assert_eq!(err, AccountManagerError::Undisputed { id: tx_id1 });

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].1.available(), amount);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), 0.0);
    }

    #[test]
    fn resolve_fails_if_transaction_is_from_other_client() {
        let mut account_manager = AccountManager::new();

        let tx_id = 2;
        let client_id = 1;
        let other_client_id = 2;
        let amount = 1.0;
        account_manager.deposit(tx_id, client_id, amount);
        assert!(account_manager.dispute(tx_id, client_id).is_ok());
        assert!(account_manager.resolve(tx_id, other_client_id).is_err());

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].1.available(), 0.0);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), amount);
    }

    #[test]
    fn resolve_fails_if_transaction_is_not_disputed() {
        let mut account_manager = AccountManager::new();

        let tx_id = 2;
        let client_id = 1;
        let amount = 1.0;
        account_manager.deposit(tx_id, client_id, amount);
        assert!(account_manager.resolve(tx_id, client_id).is_err());

        let accounts = account_manager.accounts();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].1.available(), amount);
        assert_eq!(accounts[0].1.total(), amount);
        assert_eq!(accounts[0].1.disputed(), 0.0);
    }
}
