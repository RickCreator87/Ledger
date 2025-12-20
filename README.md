# Ledger
Core ledger and transaction
```markdown
# Ledger Service - Financial Source of Truth

## Overview
The Ledger service is the authoritative source for all financial transactions and balances in the system. It implements double-entry bookkeeping principles with full audit trails.

## Core Principles
- **Immutable Transactions**: Once recorded, transactions cannot be modified
- **Double-Entry Bookkeeping**: Every transaction affects at least two accounts
- **Balance Integrity**: Guaranteed through database constraints
- **Idempotency**: All operations are idempotent via idempotency keys
- **No Floating-Point**: Uses Decimal types for financial calculations

## Architecture
```

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│Transaction   │───▶│     Entry       │───▶│    Account      │
│(Journal)    │    │  (Ledger Post)  │    │   (Chart of    │
└─────────────────┘└─────────────────┘    │    Accounts)    │
│                                     └─────────────────┘
▼
┌─────────────────┐
│Reconciliation │
│Engine      │
└─────────────────┘

```

## API Endpoints (Internal Only)
- `POST /accounts` - Create new ledger account
- `POST /transactions` - Record new transaction
- `GET /accounts/:id/balance` - Get current balance
- `GET /accounts/:id/transactions` - Get transaction history
- `POST /reconcile` - Perform reconciliation

## Database Schema
See `migrations/001_initial_schema.sql` for complete schema.

## Transaction Types
1. **Credit** - Add funds to account (debit expense/liability, credit asset)
2. **Debit** - Remove funds from account (debit asset, credit revenue/liability)
3. **Transfer** - Move funds between accounts
4. **Reversal** - Reverse previous transaction
5. **Adjustment** - Manual adjustment with audit trail

## Safety Guarantees
- ACID transactions
- No negative balances (configurable per account type)
- Full audit trail
- Automatic reconciliation checks
```
