#![allow(clippy::bool_assert_comparison)]

use accounting_demo::account_manager::{process_transaction, AccountManager};
use accounting_demo::types::{Action, ClientId, Transaction, TransactionId};

const CLIENT_ID1: ClientId = 1;
const CLIENT_ID2: ClientId = 2;

fn new_transaction(
    action: Action,
    client_id: ClientId,
    id: TransactionId,
    amount: Option<f64>,
) -> Transaction {
    Transaction {
        action,
        client_id,
        id,
        amount,
    }
}

fn get_scenario1_transactions() -> Vec<Transaction> {
    vec![
        new_transaction(Action::Deposit, CLIENT_ID1, 1, Some(1.0)),
        new_transaction(Action::Deposit, CLIENT_ID2, 2, Some(2.0)),
        new_transaction(Action::Deposit, CLIENT_ID1, 3, Some(2.0)),
        new_transaction(Action::Withdrawal, CLIENT_ID1, 4, Some(1.5)),
        new_transaction(Action::Withdrawal, CLIENT_ID2, 5, Some(3.0)),
        new_transaction(Action::Dispute, CLIENT_ID1, 1, None),
        new_transaction(Action::Dispute, CLIENT_ID2, 2, None),
        new_transaction(Action::Dispute, CLIENT_ID2, 2, None),
        new_transaction(Action::Dispute, CLIENT_ID2, 1, None),
        new_transaction(Action::Resolve, CLIENT_ID1, 1, None),
        new_transaction(Action::Chargeback, CLIENT_ID2, 2, None),
    ]
}

#[test]
fn test_scenario1() {
    let txs = get_scenario1_transactions();
    let mut account_manager = AccountManager::new();

    for tx in txs.into_iter() {
        let _ = process_transaction(&mut account_manager, tx);
    }

    let mut accounts = account_manager.accounts();
    accounts.sort_by_key(|(client_id, _)| *client_id);
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].0, 1);
    assert_eq!(accounts[0].1.available(), 1.5);
    assert_eq!(accounts[0].1.total(), 1.5);
    assert_eq!(accounts[0].1.disputed(), 0.0);
    assert_eq!(accounts[0].1.locked(), false);
    assert_eq!(accounts[1].0, 2);
    assert_eq!(accounts[1].1.available(), 0.0);
    assert_eq!(accounts[1].1.total(), 0.0);
    assert_eq!(accounts[1].1.disputed(), 0.0);
    assert_eq!(accounts[1].1.locked(), true);
}

fn get_scenario2_transactions() -> Vec<Transaction> {
    vec![
        new_transaction(Action::Deposit, CLIENT_ID1, 1, Some(1.0)),
        new_transaction(Action::Dispute, CLIENT_ID2, 1, None),
        new_transaction(Action::Deposit, CLIENT_ID2, 2, Some(2.0)),
        new_transaction(Action::Deposit, CLIENT_ID2, 7, Some(2.0)),
        new_transaction(Action::Deposit, CLIENT_ID1, 3, Some(3.0)),
        new_transaction(Action::Withdrawal, CLIENT_ID1, 4, Some(1.5)),
        new_transaction(Action::Withdrawal, CLIENT_ID2, 5, Some(2.0)),
        new_transaction(Action::Dispute, CLIENT_ID1, 3, None),
        new_transaction(Action::Dispute, CLIENT_ID2, 2, None),
        new_transaction(Action::Dispute, CLIENT_ID2, 2, None),
        new_transaction(Action::Chargeback, CLIENT_ID1, 1, None),
        new_transaction(Action::Resolve, CLIENT_ID2, 2, None),
    ]
}

#[test]
fn test_scenario2() {
    let txs = get_scenario2_transactions();
    let mut account_manager = AccountManager::new();

    for tx in txs.into_iter() {
        let _ = process_transaction(&mut account_manager, tx);
    }

    let mut accounts = account_manager.accounts();
    accounts.sort_by_key(|(client_id, _)| *client_id);
    assert_eq!(accounts.len(), 2);
    assert_eq!(accounts[0].0, 1);
    assert_eq!(accounts[0].1.available(), 2.5);
    assert_eq!(accounts[0].1.total(), 2.5);
    assert_eq!(accounts[0].1.disputed(), 0.0);
    assert_eq!(accounts[0].1.locked(), false);
    assert_eq!(accounts[1].0, 2);
    assert_eq!(accounts[1].1.available(), 2.0);
    assert_eq!(accounts[1].1.total(), 2.0);
    assert_eq!(accounts[1].1.disputed(), 0.0);
    assert_eq!(accounts[1].1.locked(), false);
}
