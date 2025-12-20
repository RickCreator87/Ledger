ledger/src/entry.rs
```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub amount: Decimal,
    pub entry_type: EntryType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance_after: Decimal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EntryType {
    Debit,
    Credit,
}

impl Entry {
    pub fn new(
        transaction_id: Uuid,
        account_id: Uuid,
        amount: Decimal,
        entry_type: EntryType,
        balance_after: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_id,
            account_id,
            amount,
            entry_type,
            timestamp: chrono::Utc::now(),
            balance_after,
        }
    }
}
```
