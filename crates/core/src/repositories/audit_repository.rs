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

    pub fn find_by_user_paginated(
        conn: &mut PgConnection,
        user_id: uuid::Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<payego_primitives::models::entities::audit_log::AuditLog>, ApiError> {
        audit_logs::table
            .filter(audit_logs::user_id.eq(user_id))
            .order(audit_logs::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<payego_primitives::models::entities::audit_log::AuditLog>(conn)
            .map_err(ApiError::Database)
    }
}
