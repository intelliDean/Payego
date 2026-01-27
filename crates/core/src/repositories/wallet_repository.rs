use diesel::prelude::*;
use payego_primitives::models::wallet::{NewWallet, Wallet};
use payego_primitives::models::wallet_ledger::NewWalletLedger;
use payego_primitives::schema::{wallet_ledger, wallets};
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::enum_types::CurrencyCode;
use uuid::Uuid;

pub struct WalletRepository;

impl WalletRepository {
    pub fn find_all_by_user(conn: &mut PgConnection, user_id: Uuid) -> Result<Vec<Wallet>, ApiError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .order(wallets::created_at.asc())
            .load::<Wallet>(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_user_and_currency(
        conn: &mut PgConnection, 
        user_id: Uuid, 
        currency: CurrencyCode
    ) -> Result<Option<Wallet>, ApiError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(currency))
            .first::<Wallet>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_user_and_currency_with_lock(
        conn: &mut PgConnection, 
        user_id: Uuid, 
        currency: CurrencyCode
    ) -> Result<Wallet, ApiError> {
        wallets::table
            .filter(wallets::user_id.eq(user_id))
            .filter(wallets::currency.eq(currency))
            .for_update()
            .first::<Wallet>(conn)
            .map_err(|e| {
                if matches!(e, diesel::result::Error::NotFound) {
                    ApiError::Payment("Wallet not found".into())
                } else {
                    ApiError::DatabaseConnection(e.to_string())
                }
            })
    }

    pub fn upsert_balance(
        conn: &mut PgConnection,
        user_id: Uuid,
        currency: CurrencyCode,
        amount: i64,
    ) -> Result<Uuid, ApiError> {
        diesel::insert_into(wallets::table)
            .values((
                wallets::user_id.eq(user_id),
                wallets::currency.eq(currency),
                wallets::balance.eq(amount),
            ))
            .on_conflict(diesel::dsl::sql::<
                diesel::sql_types::Record<(
                    diesel::sql_types::Uuid,
                    payego_primitives::schema::sql_types::CurrencyCode,
                )>,
            >("(user_id, currency)"))
            .do_update()
            .set(wallets::balance.eq(wallets::balance + amount))
            .returning(wallets::id)
            .get_result::<Uuid>(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn credit(conn: &mut PgConnection, wallet_id: Uuid, amount: i64) -> Result<(), ApiError> {
        diesel::update(wallets::table)
            .filter(wallets::id.eq(wallet_id))
            .set(wallets::balance.eq(wallets::balance + amount))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn credit_by_user_and_currency(
        conn: &mut PgConnection,
        user_id_val: Uuid,
        currency_val: CurrencyCode,
        amount: i64
    ) -> Result<(), ApiError> {
        diesel::update(wallets::table)
            .filter(wallets::user_id.eq(user_id_val))
            .filter(wallets::currency.eq(currency_val))
            .set(wallets::balance.eq(wallets::balance + amount))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }


    pub fn debit(conn: &mut PgConnection, wallet_id: Uuid, amount: i64) -> Result<(), ApiError> {
        diesel::update(wallets::table)
            .filter(wallets::id.eq(wallet_id))
            .set(wallets::balance.eq(wallets::balance - amount))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn add_ledger_entry(conn: &mut PgConnection, entry: NewWalletLedger) -> Result<(), ApiError> {
        diesel::insert_into(wallet_ledger::table)
            .values(entry)
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn create_if_not_exists(
        conn: &mut PgConnection, 
        user_id: Uuid, 
        currency: CurrencyCode
    ) -> Result<Wallet, ApiError> {
        // Try fetch with lock first
        if let Ok(wallet) = Self::find_by_user_and_currency_with_lock(conn, user_id, currency) {
            return Ok(wallet);
        }

        let new_wallet = NewWallet {
            user_id,
            currency,
        };

        diesel::insert_into(wallets::table)
            .values(&new_wallet)
            .on_conflict((wallets::user_id, wallets::currency))
            .do_nothing()
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        // Re-fetch with lock
        Self::find_by_user_and_currency_with_lock(conn, user_id, currency)
    }
}
