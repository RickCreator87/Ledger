```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub account_type: AccountType,
    pub currency: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl Account {
    pub fn new(account_type: AccountType, currency: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            account_type,
            currency: currency.to_string(),
            created_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}
```
