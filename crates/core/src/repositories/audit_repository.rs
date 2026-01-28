use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::audit_log::NewAuditLog;
use payego_primitives::schema::audit_logs;

pub struct AuditLogRepository;

impl AuditLogRepository {
    pub fn create(conn: &mut PgConnection, new_log: NewAuditLog) -> Result<(), ApiError> {
        diesel::insert_into(audit_logs::table)
            .values(&new_log)
            .execute(conn)
            .map_err(ApiError::Database)?;
        Ok(())
    }
}
