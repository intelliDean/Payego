use chrono::Utc;
use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::enum_types::PaymentState;
use payego_primitives::models::transaction::{NewTransaction, Transaction};
use payego_primitives::models::transaction_dto::TransactionSummaryDto;
use payego_primitives::schema::transactions;
use uuid::Uuid;

pub struct TransactionRepository;

impl TransactionRepository {
    pub fn find_by_id_or_reference(
        conn: &mut PgConnection,
        id_or_ref: Uuid,
    ) -> Result<Option<Transaction>, ApiError> {
        transactions::table
            .filter(
                transactions::id
                    .eq(id_or_ref)
                    .or(transactions::reference.eq(id_or_ref)),
            )
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_id_and_user(
        conn: &mut PgConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Transaction>, ApiError> {
        transactions::table
            .filter(transactions::id.eq(id))
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_id_or_ref_and_user(
        conn: &mut PgConnection,
        id_or_ref: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Transaction>, ApiError> {
        transactions::table
            .filter(
                transactions::id
                    .eq(id_or_ref)
                    .or(transactions::reference.eq(id_or_ref)),
            )
            .filter(transactions::user_id.eq(user_id))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_reference_for_update(
        conn: &mut PgConnection,
        reference: Uuid,
    ) -> Result<Option<Transaction>, ApiError> {
        transactions::table
            .filter(transactions::reference.eq(reference))
            .for_update()
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_idempotency_key(
        conn: &mut PgConnection,
        user_id: Uuid,
        key: &str,
    ) -> Result<Option<Transaction>, ApiError> {
        transactions::table
            .filter(transactions::user_id.eq(user_id))
            .filter(transactions::idempotency_key.eq(key))
            .first::<Transaction>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn create(
        conn: &mut PgConnection,
        new_tx: NewTransaction,
    ) -> Result<Transaction, ApiError> {
        let inserted_id = diesel::insert_into(transactions::table)
            .values(&new_tx)
            .on_conflict((transactions::user_id, transactions::idempotency_key))
            .do_nothing()
            .returning(transactions::id)
            .get_result::<Uuid>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        match inserted_id {
            Some(id) => transactions::table
                .find(id)
                .first::<Transaction>(conn)
                .map_err(|e| ApiError::DatabaseConnection(e.to_string())),
            None => transactions::table
                .filter(transactions::user_id.eq(new_tx.user_id))
                .filter(transactions::idempotency_key.eq(new_tx.idempotency_key))
                .first::<Transaction>(conn)
                .map_err(|e| ApiError::DatabaseConnection(e.to_string())),
        }
    }

    pub fn update_status_and_provider_ref(
        conn: &mut PgConnection,
        id: Uuid,
        status: PaymentState,
        provider_ref: Option<String>,
    ) -> Result<(), ApiError> {
        diesel::update(transactions::table.find(id))
            .set((
                transactions::txn_state.eq(status),
                transactions::provider_reference.eq(provider_ref),
                transactions::updated_at.eq(Utc::now()),
            ))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn update_status_by_reference(
        conn: &mut PgConnection,
        reference: Uuid,
        status: PaymentState,
    ) -> Result<(), ApiError> {
        diesel::update(transactions::table)
            .filter(transactions::reference.eq(reference))
            .set(transactions::txn_state.eq(status))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn update_state(
        conn: &mut PgConnection,
        id: Uuid,
        status: PaymentState,
    ) -> Result<(), ApiError> {
        diesel::update(transactions::table.find(id))
            .set(transactions::txn_state.eq(status))
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;
        Ok(())
    }

    pub fn find_recent_by_user(
        conn: &mut PgConnection,
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<TransactionSummaryDto>, ApiError> {
        transactions::table
            .filter(transactions::user_id.eq(user_id))
            .order(transactions::created_at.desc())
            .limit(limit)
            .select((
                transactions::id,
                transactions::intent,
                transactions::amount,
                transactions::currency,
                transactions::created_at,
                transactions::txn_state,
                transactions::reference,
            ))
            .load::<TransactionSummaryDto>(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }
}
