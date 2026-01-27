use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::bank::{Bank, NewBank};
use payego_primitives::schema::banks;

pub struct BankRepository;

impl BankRepository {
    pub fn count(conn: &mut PgConnection) -> Result<i64, ApiError> {
        banks::table
            .count()
            .get_result(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn create_many(conn: &mut PgConnection, new_banks: Vec<NewBank>) -> Result<usize, ApiError> {
        diesel::insert_into(banks::table)
            .values(&new_banks)
            .on_conflict(banks::code)
            .do_nothing()
            .execute(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn list_active_by_country(
        conn: &mut PgConnection,
        country_code: &str,
    ) -> Result<Vec<Bank>, ApiError> {
        banks::table
            .filter(banks::country.eq(country_code))
            .filter(banks::is_active.eq(true))
            .order(banks::name.asc())
            .load::<Bank>(conn)
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }
}
