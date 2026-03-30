use crate::transaction::Amount;

#[derive(Debug)]
pub struct Account {
    pub available: Amount,
    pub held: Amount,
    pub frozen: bool,
}

impl Account {
    pub fn new() -> Self {
        Self {
            available: Amount::default(),
            held: Amount::default(),
            frozen: false,
        }
    }

    // Remove the requested amount from the account, errors if there
    // are insufficent funds and leaves account state unchanged.
    pub fn withdraw(&mut self, amount: Amount) -> anyhow::Result<()> {
        if self.available < amount {
            anyhow::bail!("insufficient funds to withdraw");
        }

        self.available.sub(amount);

        Ok(())
    }

    // Deposit the requested amount into the account.
    pub fn deposit(&mut self, amount: Amount) {
        self.available.add(amount);
    }

    // Put funds on hold, lowering available balance and increasing
    // held funds. NOTE: here available funds might go negative, if
    // a disputed tx is more than the available balance of a client's
    // account.
    pub fn hold(&mut self, amount: Amount) {
        self.available.sub(amount);
        self.held.add(amount);
    }

    // Reduces the amount of held funds and increases the available funds by 
    // the amount, panics if there are insufficient held funds to release.
    pub fn release(&mut self, amount: Amount) {
        // Defensively protecting this invariant. In practice this should
        // never happen as only transactions marked as disputed can be
        // released.
        if self.held < amount {
            panic!("cannot release funds that are not held");
        }

        self.held.sub(amount);
        self.available.add(amount);
    }

    // Reduces the held funds by amount and freezes the account. Panics
    // if the required amount is not held.
    pub fn chargeback(&mut self, amount: Amount) {
        // Defensively protecting this invariant. In practice this should
        // never happen as only transactions marked as disputed can be
        // released.
        if self.held < amount {
            panic!("cannot chargeback funds that are not held");
        }

        self.frozen = true;
        self.held.sub(amount);
    }
}

#[cfg(test)]
mod test {
    use rust_decimal::Decimal;

    use super::*;

    #[test]
    pub fn deposit() {
        let mut account = Account::new();
        let amount = Amount::from_float(123.4567).expect("valid float");
        account.deposit(amount);

        assert_eq!(account.available, amount);
    }

    #[test]
    pub fn withdraw_with_sufficient_funds() {
        let mut account = account_with_funds();
        let amount = Amount::from_float(70.10).expect("valid float");

        let result = account.withdraw(amount);

        assert!(result.is_ok());
        assert_eq!(account.available.display(), Decimal::new(100230, 4))
    }

    #[test]
    pub fn withdraw_insuffificent_funds() {
        let mut account = account_with_funds();
        let amount = Amount::from_float(90.10).expect("valid float");

        let result = account.withdraw(amount);

        assert!(result.is_err());
        assert_eq!(account.available.display(), Decimal::new(801230, 4))
    }

    #[test]
    pub fn hold_payment() {
        let mut account = account_with_funds();
        let amount = Amount::from_float(100.0).expect("valid float");

        account.hold(amount);

        assert_eq!(account.available.display(), Decimal::new(-198770, 4));
        assert_eq!(account.held.display(), Decimal::new(1000000, 4));
    }

    #[test]
    pub fn release_payment() {
        let mut account = account_with_funds();
        let amount = Amount::from_float(100.0).expect("valid float");

        account.hold(amount);

        account.release(amount);
        assert_eq!(account.available.display(), Decimal::new(801230, 4));
        assert_eq!(account.held.display(), Decimal::new(000000, 4));
    }

    #[test]
    #[should_panic]
    pub fn cannot_release_more_than_held() {
        let mut account = Account::new();
        let amount = Amount::from_float(100.0).expect("valid float");

        account.release(amount);
    }

    #[test]
    pub fn chargeback_payment() {
        let mut account = account_with_funds();
        let amount = Amount::from_float(100.0).expect("valid float");

        account.hold(amount);

        account.chargeback(amount);

        assert_eq!(account.available.display(), Decimal::new(-198770, 4));
        assert_eq!(account.held.display(), Decimal::new(000000, 4));

    }

    #[test]
    #[should_panic]
    pub fn cannot_chargeback_more_than_held() {
        let mut account = Account::new();
        let amount = Amount::from_float(100.0).expect("valid float");

        account.chargeback(amount);
    }

    fn account_with_funds() -> Account {
        let mut account = Account::new();
        let amount = Amount::from_float(80.123).expect("valid float");

        account.deposit(amount);

        account
    }
}
