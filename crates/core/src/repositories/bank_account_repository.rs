use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::bank::{BankAccount, NewBankAccount};
use payego_primitives::schema::bank_accounts;
use uuid::Uuid;

pub struct BankAccountRepository;

impl BankAccountRepository {
    pub fn find_all_by_user(
        conn: &mut PgConnection,
        user_id: Uuid,
    ) -> Result<Vec<BankAccount>, ApiError> {
        bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .load::<BankAccount>(conn)
            .map_err(|e: diesel::result::Error| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_id_and_user(
        conn: &mut PgConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<BankAccount>, ApiError> {
        bank_accounts::table
            .filter(bank_accounts::id.eq(id))
            .filter(bank_accounts::user_id.eq(user_id))
            .first::<BankAccount>(conn)
            .optional()
            .map_err(|e: diesel::result::Error| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_verified_by_id_and_user(
        conn: &mut PgConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<BankAccount, ApiError> {
        bank_accounts::table
            .filter(bank_accounts::id.eq(id))
            .filter(bank_accounts::user_id.eq(user_id))
            .filter(bank_accounts::is_verified.eq(true))
            .first::<BankAccount>(conn)
            .map_err(|_| ApiError::Internal("Verified bank account not found".into()))
    }

    pub fn create(
        conn: &mut PgConnection,
        new_bank: NewBankAccount,
    ) -> Result<BankAccount, ApiError> {
        diesel::insert_into(bank_accounts::table)
            .values(&new_bank)
            .get_result(conn)
            .map_err(|e: diesel::result::Error| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn delete_by_id_and_user(
        conn: &mut PgConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<(), ApiError> {
        let deleted_rows = diesel::delete(
            bank_accounts::table
                .filter(bank_accounts::id.eq(id))
                .filter(bank_accounts::user_id.eq(user_id)),
        )
        .execute(conn)
        .map_err(|e: diesel::result::Error| ApiError::DatabaseConnection(e.to_string()))?;

        if deleted_rows == 0 {
            return Err(ApiError::Internal(
                "Bank account not found or access denied".into(),
            ));
        }

        Ok(())
    }

    pub fn find_active_by_details(
        conn: &mut PgConnection,
        user_id: Uuid,
        bank_code: &str,
        account_number: &str,
    ) -> Result<Option<BankAccount>, ApiError> {
        bank_accounts::table
            .filter(bank_accounts::user_id.eq(user_id))
            .filter(bank_accounts::bank_code.eq(bank_code))
            .filter(bank_accounts::account_number.eq(account_number))
            .first::<BankAccount>(conn)
            .optional()
            .map_err(|e: diesel::result::Error| ApiError::DatabaseConnection(e.to_string()))
    }
}
