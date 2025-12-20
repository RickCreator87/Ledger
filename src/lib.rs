ledger/src/lib.rs


```rust
pub mod account;
pub mod transaction;
pub mod entry;
pub mod ledger_store;
pub mod reconciliation;

pub use account::*;
pub use transaction::*;
pub use entry::*;
pub use ledger_store::*;
pub use reconciliation::*;

pub struct LedgerService {
    store: Box<dyn LedgerStore>,
}

impl LedgerService {
    pub fn new(store: Box<dyn LedgerStore>) -> Self {
        Self { store }
    }

    pub async fn create_account(
        &self,
        account_type: AccountType,
        currency: &str,
    ) -> Result<Account, LedgerError> {
        let account = Account::new(account_type, currency);
        self.store.create_account(&account).await?;
        Ok(account)
    }

    pub async fn credit_account(
        &self,
        account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        let transaction = Transaction::new(
            TransactionType::Credit,
            amount,
            None,
            Some(account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_credit_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    pub async fn transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        // Check source balance
        let source_balance = self.store.get_account_balance(&from_account_id).await?;
        if source_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }

        let transaction = Transaction::new(
            TransactionType::Transfer,
            amount,
            Some(from_account_id),
            Some(to_account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_transfer_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    async fn create_credit_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let current_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_balance = current_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_balance,
            ));
        }
        
        Ok(entries)
    }

    async fn create_transfer_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(source_account_id) = transaction.source_account_id {
            let source_balance = self.store.get_account_balance(&source_account_id).await?;
            let new_source_balance = source_balance - transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                source_account_id,
                transaction.amount,
                EntryType::Debit,
                new_source_balance,
            ));
        }
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let dest_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_dest_balance = dest_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_dest_balance,
            ));
        }
        
        Ok(entries)
    }

    pub async fn get_account_balance(
        &self,
        account_id: Uuid,
    ) -> Result<rust_decimal::Decimal, LedgerError> {
        self.store.get_account_balance(&account_id).await
    }

    pub async fn get_account_transactions(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError> {
        self.store.get_account_transactions(&account_id, limit, offset).await
    }
}
```
```rust
pub mod account;
pub mod transaction;
pub mod entry;
pub mod ledger_store;
pub mod reconciliation;

pub use account::*;
pub use transaction::*;
pub use entry::*;
pub use ledger_store::*;
pub use reconciliation::*;

pub struct LedgerService {
    store: Box<dyn LedgerStore>,
}

impl LedgerService {
    pub fn new(store: Box<dyn LedgerStore>) -> Self {
        Self { store }
    }

    pub async fn create_account(
        &self,
        account_type: AccountType,
        currency: &str,
    ) -> Result<Account, LedgerError> {
        let account = Account::new(account_type, currency);
        self.store.create_account(&account).await?;
        Ok(account)
    }

    pub async fn credit_account(
        &self,
        account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        let transaction = Transaction::new(
            TransactionType::Credit,
            amount,
            None,
            Some(account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_credit_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    pub async fn transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        // Check source balance
        let source_balance = self.store.get_account_balance(&from_account_id).await?;
        if source_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }

        let transaction = Transaction::new(
            TransactionType::Transfer,
            amount,
            Some(from_account_id),
            Some(to_account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_transfer_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    async fn create_credit_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let current_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_balance = current_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_balance,
            ));
        }
        
        Ok(entries)
    }

    async fn create_transfer_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(source_account_id) = transaction.source_account_id {
            let source_balance = self.store.get_account_balance(&source_account_id).await?;
            let new_source_balance = source_balance - transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                source_account_id,
                transaction.amount,
                EntryType::Debit,
                new_source_balance,
            ));
        }
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let dest_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_dest_balance = dest_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_dest_balance,
            ));
        }
        
        Ok(entries)
    }

    pub async fn get_account_balance(
        &self,
        account_id: Uuid,
    ) -> Result<rust_decimal::Decimal, LedgerError> {
        self.store.get_account_balance(&account_id).await
    }

    pub async fn get_account_transactions(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError> {
        self.store.get_account_transactions(&account_id, limit, offset).await
    }
}
```
```rust
pub mod account;
pub mod transaction;
pub mod entry;
pub mod ledger_store;
pub mod reconciliation;

pub use account::*;
pub use transaction::*;
pub use entry::*;
pub use ledger_store::*;
pub use reconciliation::*;

pub struct LedgerService {
    store: Box<dyn LedgerStore>,
}

impl LedgerService {
    pub fn new(store: Box<dyn LedgerStore>) -> Self {
        Self { store }
    }

    pub async fn create_account(
        &self,
        account_type: AccountType,
        currency: &str,
    ) -> Result<Account, LedgerError> {
        let account = Account::new(account_type, currency);
        self.store.create_account(&account).await?;
        Ok(account)
    }

    pub async fn credit_account(
        &self,
        account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        let transaction = Transaction::new(
            TransactionType::Credit,
            amount,
            None,
            Some(account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_credit_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    pub async fn transfer(
        &self,
        from_account_id: Uuid,
        to_account_id: Uuid,
        amount: rust_decimal::Decimal,
        reason_code: &str,
        idempotency_key: &str,
    ) -> Result<Transaction, LedgerError> {
        // Check source balance
        let source_balance = self.store.get_account_balance(&from_account_id).await?;
        if source_balance < amount {
            return Err(LedgerError::InsufficientBalance);
        }

        let transaction = Transaction::new(
            TransactionType::Transfer,
            amount,
            Some(from_account_id),
            Some(to_account_id),
            reason_code,
            idempotency_key,
        );

        transaction.validate()?;

        // Create entries
        let entries = self.create_transfer_entries(&transaction).await?;
        
        // Record transaction
        self.store.record_transaction(&transaction, &entries).await?;
        
        Ok(transaction)
    }

    async fn create_credit_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let current_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_balance = current_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_balance,
            ));
        }
        
        Ok(entries)
    }

    async fn create_transfer_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(source_account_id) = transaction.source_account_id {
            let source_balance = self.store.get_account_balance(&source_account_id).await?;
            let new_source_balance = source_balance - transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                source_account_id,
                transaction.amount,
                EntryType::Debit,
                new_source_balance,
            ));
        }
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let dest_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_dest_balance = dest_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_dest_balance,
            ));
        }
        
        Ok(entries)
    }

    pub async fn get_account_balance(
        &self,
        account_id: Uuid,
    ) -> Result<rust_decimal::Decimal, LedgerError> {
        self.store.get_account_balance(&account_id).await
    }

    pub async fn get_account_transactions(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError> {
        self.store.get_account_transactions(&account_id, limit, offset).await
    }
}
```
transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let current_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_balance = current_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_balance,
            ));
        }
        
        Ok(entries)
    }

    async fn create_transfer_entries(
        &self,
        transaction: &Transaction,
    ) -> Result<Vec<Entry>, LedgerError> {
        let mut entries = Vec::new();
        
        if let Some(source_account_id) = transaction.source_account_id {
            let source_balance = self.store.get_account_balance(&source_account_id).await?;
            let new_source_balance = source_balance - transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                source_account_id,
                transaction.amount,
                EntryType::Debit,
                new_source_balance,
            ));
        }
        
        if let Some(dest_account_id) = transaction.destination_account_id {
            let dest_balance = self.store.get_account_balance(&dest_account_id).await?;
            let new_dest_balance = dest_balance + transaction.amount;
            
            entries.push(Entry::new(
                transaction.id,
                dest_account_id,
                transaction.amount,
                EntryType::Credit,
                new_dest_balance,
            ));
        }
        
        Ok(entries)
    }

    pub async fn get_account_balance(
        &self,
        account_id: Uuid,
    ) -> Result<rust_decimal::Decimal, LedgerError> {
        self.store.get_account_balance(&account_id).await
    }

    pub async fn get_account_transactions(
        &self,
        account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, LedgerError> {
        self.store.get_account_transactions(&account_id, limit, offset).await
    }
}
```
