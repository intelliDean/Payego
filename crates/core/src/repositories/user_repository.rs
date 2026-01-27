use diesel::prelude::*;
use payego_primitives::error::ApiError;
use payego_primitives::models::entities::user::{NewUser, User};
use payego_primitives::schema::users;
use uuid::Uuid;

pub struct UserRepository;

impl UserRepository {
    pub fn find_by_id(conn: &mut PgConnection, user_id: Uuid) -> Result<Option<User>, ApiError> {
        users::table
            .find(user_id)
            .first::<User>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_email(
        conn: &mut PgConnection,
        user_email: &str,
    ) -> Result<Option<User>, ApiError> {
        users::table
            .filter(users::email.eq(user_email))
            .first::<User>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn find_by_username(
        conn: &mut PgConnection,
        user_name: &str,
    ) -> Result<Option<User>, ApiError> {
        users::table
            .filter(users::username.eq(user_name))
            .first::<User>(conn)
            .optional()
            .map_err(|e| ApiError::DatabaseConnection(e.to_string()))
    }

    pub fn create(conn: &mut PgConnection, new_user: NewUser) -> Result<User, ApiError> {
        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result::<User>(conn)
            .map_err(|e| {
                if matches!(
                    e,
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::UniqueViolation,
                        _
                    )
                ) {
                    ApiError::Auth(payego_primitives::error::AuthError::DuplicateEmail)
                } else {
                    ApiError::DatabaseConnection(e.to_string())
                }
            })
    }
}
