ledger/src/ledger_store.rs
```rust
use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::{account::Account, transaction::{Transaction, TransactionError}, entry::Entry};

#[async_trait]
pub trait LedgerStore: Send + Sync {
    async fn create_account(&self, account: &Account) -> Result<(), LedgerError>;
    async fn get_account(&self, account_id: &Uuid) -> Result<Option<Account>, LedgerError>;
    async fn get_account_balance(&self, account_id: &Uuid) -> Result<Decimal, LedgerError>;
    async fn record_transaction(
        &self,
        transaction: &Transaction,
        entries: &[Entry],
    ) -> Result<(), LedgerError>;
    async fn get_transaction(&self, transaction_id: &Uuid) -> Result<Option<Transaction>, LedgerError>;
    async fn get_transaction_by_key(&self, idempotency_key: &str) -> Result<Option<Transaction>, LedgerError>;
    async fn get_account_transactions(
        &self,
        account_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError>;
    async fn get_entries_for_transaction(
        &self,
        transaction_id: &Uuid,
    ) -> Result<Vec<Entry>, LedgerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Account not found")]
    AccountNotFound,
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Transaction error: {0}")]
    TransactionError(#[from] TransactionError),
    #[error("Idempotency violation")]
    IdempotencyViolation,
}

pub struct PostgresLedgerStore {
    pool: PgPool,
}

impl PostgresLedgerStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LedgerStore for PostgresLedgerStore {
    async fn create_account(&self, account: &Account) -> Result<(), LedgerError> {
        sqlx::query!(
            r#"
            INSERT INTO accounts (id, account_type, currency, created_at, metadata)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            account.id,
            account.account_type as _,
            &account.currency,
            account.created_at,
            &account.metadata
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    async fn get_account(&self, account_id: &Uuid) -> Result<Option<Account>, LedgerError> {
        let account = sqlx::query_as!(
            Account,
            r#"
            SELECT id, account_type as "account_type: _", currency, created_at, metadata
            FROM accounts WHERE id = $1
            "#,
            account_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(account)
    }

    async fn get_account_balance(&self, account_id: &Uuid) -> Result<Decimal, LedgerError> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(
                CASE 
                    WHEN entry_type = 'Debit' THEN amount
                    ELSE -amount
                END
            ), 0) as balance
            FROM entries
            WHERE account_id = $1
            "#,
            account_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.balance.unwrap_or(Decimal::ZERO))
    }

    async fn record_transaction(
        &self,
        transaction: &Transaction,
        entries: &[Entry],
    ) -> Result<(), LedgerError> {
        let mut db_transaction = self.pool.begin().await?;

        // Check idempotency
        let existing = sqlx::query!(
            "SELECT id FROM transactions WHERE idempotency_key = $1",
            &transaction.idempotency_key
        )
        .fetch_optional(&mut *db_transaction)
        .await?;

        if existing.is_some() {
            return Err(LedgerError::IdempotencyViolation);
        }

        // Insert transaction
        sqlx::query!(
            r#"
            INSERT INTO transactions (
                id, transaction_type, amount, source_account_id,
                destination_account_id, timestamp, reason_code,
                metadata, idempotency_key
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            transaction.id,
            transaction.transaction_type as _,
            transaction.amount,
            transaction.source_account_id,
            transaction.destination_account_id,
            transaction.timestamp,
            &transaction.reason_code,
            &transaction.metadata,
            &transaction.idempotency_key
        )
        .execute(&mut *db_transaction)
        .await?;

        // Insert entries
        for entry in entries {
            sqlx::query!(
                r#"
                INSERT INTO entries (
                    id, transaction_id, account_id, amount,
                    entry_type, timestamp, balance_after
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
                entry.id,
                entry.transaction_id,
                entry.account_id,
                entry.amount,
                entry.entry_type as _,
                entry.timestamp,
                entry.balance_after
            )
            .execute(&mut *db_transaction)
            .await?;
        }

        db_transaction.commit().await?;
        Ok(())
    }

    async fn get_transaction(&self, transaction_id: &Uuid) -> Result<Option<Transaction>, LedgerError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT id, transaction_type as "transaction_type: _", amount,
                   source_account_id, destination_account_id, timestamp,
                   reason_code, metadata, idempotency_key
            FROM transactions WHERE id = $1
            "#,
            transaction_id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(transaction)
    }

    async fn get_transaction_by_key(&self, idempotency_key: &str) -> Result<Option<Transaction>, LedgerError> {
        let transaction = sqlx::query_as!(
            Transaction,
            r#"
            SELECT id, transaction_type as "transaction_type: _", amount,
                   source_account_id, destination_account_id, timestamp,
                   reason_code, metadata, idempotency_key
            FROM transactions WHERE idempotency_key = $1
            "#,
            idempotency_key
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(transaction)
    }

    async fn get_account_transactions(
        &self,
        account_id: &Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError> {
        let transactions = sqlx::query_as!(
            Transaction,
            r#"
            SELECT DISTINCT t.id, t.transaction_type as "transaction_type: _", t.amount,
                   t.source_account_id, t.destination_account_id, t.timestamp,
                   t.reason_code, t.metadata, t.idempotency_key
            FROM transactions t
            JOIN entries e ON t.id = e.transaction_id
            WHERE e.account_id = $1
            ORDER BY t.timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
            account_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(transactions)
    }

    async fn get_entries_for_transaction(
        &self,
        transaction_id: &Uuid,
    ) -> Result<Vec<Entry>, LedgerError> {
        let entries = sqlx::query_as!(
            Entry,
            r#"
            SELECT id, transaction_id, account_id, amount,
                   entry_type as "entry_type: _", timestamp, balance_after
            FROM entries WHERE transaction_id = $1
            ORDER BY timestamp
            "#,
            transaction_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(entries)
    }
}
```
