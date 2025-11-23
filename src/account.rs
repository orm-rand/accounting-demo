use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AccountError {
    #[error("Insufficient funds. Requested {requested} of {available}.")]
    InsufficientFunds { requested: f64, available: f64 },

    #[error("Account is locked")]
    Locked,
}

pub type AccountResult<T> = Result<T, AccountError>;

#[derive(Debug, Clone, Default)]
pub struct Account {
    available: f64,
    disputed: f64,
    locked: bool,
}

impl Account {
    pub fn new() -> Self {
        Self {
            available: 0.0,
            disputed: 0.0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: f64) {
        self.available += amount;
    }

    pub fn withdraw(&mut self, amount: f64) -> AccountResult<()> {
        self.check_locked()?;
        self.check_sufficient_funds(amount)?;

        self.available -= amount;
        Ok(())
    }

    pub fn dispute(&mut self, amount: f64) -> AccountResult<()> {
        self.check_locked()?;
        self.check_sufficient_funds(amount)?;

        self.available -= amount;
        self.disputed += amount;
        Ok(())
    }

    pub fn resolve(&mut self, amount: f64) {
        self.available += amount;
        self.disputed -= amount;
    }

    pub fn chargeback(&mut self, amount: f64) {
        self.disputed -= amount;
        self.locked = true;
    }

    pub fn available(&self) -> f64 {
        self.available
    }

    pub fn total(&self) -> f64 {
        self.available + self.disputed
    }

    pub fn disputed(&self) -> f64 {
        self.disputed
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    fn check_sufficient_funds(&self, requested: f64) -> AccountResult<()> {
        if requested > self.available {
            return Err(AccountError::InsufficientFunds {
                requested,
                available: self.available,
            });
        }
        Ok(())
    }

    fn check_locked(&self) -> AccountResult<()> {
        if self.locked {
            return Err(AccountError::Locked);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deposit_increases_total_and_available_amounts() {
        let mut account = Account::new();
        assert_eq!(account.available(), 0.0);
        assert_eq!(account.total(), 0.0);
        assert_eq!(account.disputed(), 0.0);

        let amount = 1.0;
        account.deposit(amount);

        assert_eq!(account.available(), amount);
        assert_eq!(account.total(), amount);
        assert_eq!(account.disputed(), 0.0);
    }

    #[test]
    fn withdrawal_fails_if_not_enough_funds() {
        let mut account = Account::new();

        let amount = 1.0;
        let err = account.withdraw(amount).unwrap_err();
        assert_eq!(
            err,
            AccountError::InsufficientFunds {
                requested: amount,
                available: 0.0
            }
        );
    }

    #[test]
    fn withdrawal_fails_if_not_enough_funds_due_to_dispute() {
        let mut account = Account::new();

        let amount = 1.0;
        account.deposit(amount);
        let dispute_amount = 0.4;
        assert!(account.dispute(dispute_amount).is_ok());

        let err = account.withdraw(amount).unwrap_err();
        let expected_available = 0.6;
        assert_eq!(
            err,
            AccountError::InsufficientFunds {
                requested: amount,
                available: expected_available
            }
        );
    }

    #[test]
    fn withdrawal_succeeds_if_enough_funds() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);

        let withdrawal_amount = 0.4;
        assert!(account.withdraw(withdrawal_amount).is_ok());

        let expected_remaining_amount = 0.6;
        assert_eq!(account.available(), expected_remaining_amount);
        assert_eq!(account.total(), expected_remaining_amount);
        assert_eq!(account.disputed(), 0.0);
    }

    #[test]
    fn dispute_locks_funds() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);
        let dispute_amount = 0.4;
        assert!(account.dispute(dispute_amount).is_ok());

        let expected_available = 0.6;
        assert_eq!(account.available(), expected_available);
        assert_eq!(account.total(), deposit_amount);
        assert_eq!(account.disputed(), dispute_amount);
    }

    #[test]
    fn dispute_fails_if_insufficient_funds() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);
        let dispute_amount = 1.4;
        let err = account.dispute(dispute_amount).unwrap_err();
        assert_eq!(
            err,
            AccountError::InsufficientFunds {
                requested: dispute_amount,
                available: deposit_amount
            }
        );
    }

    #[test]
    fn resolve_unlocks_funds() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);
        let dispute_amount = 0.4;
        assert!(account.dispute(dispute_amount).is_ok());
        account.resolve(dispute_amount);

        assert_eq!(account.available(), deposit_amount);
        assert_eq!(account.total(), deposit_amount);
        assert_eq!(account.disputed(), 0.0);
    }

    #[test]
    fn chargeback_removes_disputed_funds() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);
        let dispute_amount = 0.4;
        assert!(account.dispute(dispute_amount).is_ok());
        account.chargeback(dispute_amount);

        let expected_available = 0.6;
        assert_eq!(account.available(), expected_available);
        assert_eq!(account.total(), expected_available);
        assert_eq!(account.disputed(), 0.0);
        assert!(account.locked());
    }

    #[test]
    fn after_chargeback_account_is_locked() {
        let mut account = Account::new();

        let deposit_amount = 1.0;
        account.deposit(deposit_amount);
        let dispute_amount = 0.4;
        assert!(account.dispute(dispute_amount).is_ok());
        account.chargeback(dispute_amount);

        let err = account.withdraw(deposit_amount).unwrap_err();
        assert_eq!(err, AccountError::Locked);
    }
}
