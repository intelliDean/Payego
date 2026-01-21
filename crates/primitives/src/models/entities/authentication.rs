use chrono::{DateTime, Utc};
use diesel::{Associations, Identifiable, Insertable, Queryable};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Queryable, Identifiable)]
#[diesel(table_name = crate::schema::blacklisted_tokens)]
#[diesel(primary_key(jti))]
pub struct BlacklistedToken {
    pub jti: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::blacklisted_tokens)]
pub struct NewBlacklistedToken<'a> {
    pub jti: &'a str,
    pub expires_at: DateTime<Utc>,
}

#[derive(Queryable, Identifiable, Associations)]
#[diesel(table_name = crate::schema::refresh_tokens)]
#[diesel(belongs_to(crate::models::entities::user::User))]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = crate::schema::refresh_tokens)]
pub struct NewRefreshToken<'a> {
    pub user_id: Uuid,
    pub token_hash: &'a str,
    pub expires_at: DateTime<Utc>,
}
