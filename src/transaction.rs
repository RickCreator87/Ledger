ledger/src/transaction.rs
```rust
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::entry::Entry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub source_account_id: Option<Uuid>,
    pub destination_account_id: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub reason_code: String,
    pub entries: Vec<Entry>,
    pub metadata: serde_json::Value,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
    Transfer,
    Reversal,
    Adjustment,
}

impl Transaction {
    pub fn new(
        transaction_type: TransactionType,
        amount: Decimal,
        source_account_id: Option<Uuid>,
        destination_account_id: Option<Uuid>,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_type,
            amount,
            source_account_id,
            destination_account_id,
            timestamp: chrono::Utc::now(),
            reason_code: reason_code.to_string(),
            entries: Vec::new(),
            metadata: serde_json::json!({}),
            idempotency_key: idempotency_key.to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.amount <= Decimal::ZERO {
            return Err(TransactionError::InvalidAmount);
        }

        match self.transaction_type {
            TransactionType::Credit => {
                if self.destination_account_id.is_none() {
                    return Err(TransactionError::MissingDestinationAccount);
                }
            }
            TransactionType::Debit => {
                if self.source_account_id.is_none() {
                    return Err(TransactionError::MissingSourceAccount);
                }
            }
            TransactionType::Transfer => {
                if self.source_account_id.is_none() || self.destination_account_id.is_none() {
                    return Err(TransactionError::MissingAccountForTransfer);
                }
                if self.source_account_id == self.destination_account_id {
                    return Err(TransactionError::SameAccountTransfer);
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Transaction amount must be positive")]
    InvalidAmount,
    #[error("Missing destination account for credit")]
    MissingDestinationAccount,
    #[error("Missing source account for debit")]
    MissingSourceAccount,
    #[error("Both source and destination accounts required for transfer")]
    MissingAccountForTransfer,
    #[error("Cannot transfer to same account")]
    SameAccountTransfer,
    #[error("Transaction already processed")]
    DuplicateTransaction,
}
```
