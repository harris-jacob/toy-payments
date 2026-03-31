use rust_decimal::Decimal;

pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TransactionId(u32);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ClientId(u16);

#[derive(PartialEq, PartialOrd, Default, Eq, Debug, Clone, Copy)]
pub struct Amount(Decimal);

pub struct Transaction {
    pub r#type: TransactionType,
    pub id: TransactionId,
    pub client_id: ClientId,
    pub amount: Amount,
}

impl ClientId {
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    pub fn inner(&self) -> u16 {
        self.0
    }
}

impl Amount {
    pub fn new(value: Decimal) -> Self {
        Self(value)
    }

    pub fn sub(&mut self, value: Amount) {
        self.0 -= value.0
    }

    pub fn add(&mut self, value: Amount) {
        self.0 += value.0
    }

    pub fn display(&self) -> Decimal {
        self.0.round_dp(4)
    }

    #[cfg(test)]
    pub fn from_float(value: f32) -> anyhow::Result<Self> {
        use rust_decimal::prelude::FromPrimitive;

        let value = Decimal::from_f32(value)
            .ok_or(anyhow::anyhow!("cannot represent decimal as amount"))?;

        Ok(Self(value))
    }
}

impl std::ops::Add for Amount {
    type Output = Amount;

    fn add(self, rhs: Self) -> Self::Output {
        Amount(self.0 + rhs.0)
    }
}

impl TransactionId {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
}

impl TransactionType {
    pub fn new(value: String) -> anyhow::Result<Self> {
        match value.as_str() {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::Chargeback),
            _ => anyhow::bail!("Unknown transaction type"),
        }
    }
}
