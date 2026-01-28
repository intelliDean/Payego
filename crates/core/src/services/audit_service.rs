use crate::app_state::AppState;
use crate::repositories::audit_repository::AuditLogRepository;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::audit_log::NewAuditLog;
use uuid::Uuid;

pub struct AuditService;

impl AuditService {
    pub async fn log_event(
        state: &AppState,
        user_id: Option<Uuid>,
        event_type: &str,
        target_type: Option<&str>,
        target_id: Option<&str>,
        metadata: serde_json::Value,
        ip_address: Option<String>,
    ) -> Result<(), ApiError> {
        let mut conn = state
            .db
            .get()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))?;

        let new_log = NewAuditLog {
            id: Uuid::new_v4(),
            user_id,
            event_type: event_type.to_string(),
            target_type: target_type.map(|s| s.to_string()),
            target_id: target_id.map(|s| s.to_string()),
            metadata,
            ip_address,
        };

        AuditLogRepository::create(&mut conn, new_log)
    }
}
