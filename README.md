# accounting demo

### Usage

* build: `cargo build`
* run tests: `cargo test`
* run: `cargo run -- <CSV_TRANSACTION_FILE>`

### Components
 * struct Account (account.rs): responsible for tracking the balance in a user account
 * struct AccountManager (account_manager.rs): holds a map of accounts and a tx cache, responsible for updating accounts for different transactions

#### Testing
Being the most low-level component, Account has the highest unit test coverage. Additional cases are handled in the unit tests of the AccountManager. 

In `tests/test_scenarios.rs` there are two functional tests involving a sequence of transactions and two clients.

### Transaction handling
 * `deposit`: deposit funds to a clients account
 * `withdrawal`: withdraws funds from a clients account<br>
   fails if <br>
   - account is locked
   - the withdrawal exceeds the available balance
 * `dispute`: disputes a deposit transaction (locks the disputed amount)<br>
   fails if <br>
   * account is locked
   * disputed amount exceeds the available balance
   * transaction is not owned by client
   * transaction is already under dispute
* `resolve`: resolves a dispute if the clients dispute is rejected (unlocks the disputed amount, the transaction can be disputed again)
* `chargeback`: resolves a dispute if the clients dispute is accepted and locks the account (unlocks and removes the disputed amount, the transaction can not be disputed again)

### Notes:
 * withdrawals can not be disputed, that may be worth adding
 * errors that are ignored through `let _ = ` should be replaced with logging of the errors
 * it is assumed that locking/freezing of an account after a chargeback implies that no withdrawals/disputes are possible for the client until the account gets unlocked (possibly after human review)
 * deposits can be disputed if the final account state is valid but there is an invalid state between dispute and current state. I.e. consider
   * deposit of 1 (tx 1)
   * withdrawal of 1 (tx 2)
   * deposit of 1 (tx 3)<br>
   In this case it is possible to dispute (tx 1). Changing this requires to analyze the full sequence of transactions after the disputed transaction and requires more time to implement.
 * thx tx-cache inside the AccountManager is never cleaned up, so large amounts of transactions will lead to memory issues, so transactions should be stored in a database.
