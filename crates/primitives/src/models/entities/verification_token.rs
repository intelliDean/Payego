use crate::schema::verification_tokens;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(crate::models::entities::user::User))]
#[diesel(table_name = verification_tokens)]
pub struct VerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = verification_tokens)]
pub struct NewVerificationToken {
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
}
