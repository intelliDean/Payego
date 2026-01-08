use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use secrecy::SecretString;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: SecretString,
    pub stripe_secret_key: SecretString,
    pub app_url: String,
    pub exchange_api_url: String,
    pub paypal_api_url: String,
    pub paystack_api_url: String,
    pub paystack_secret_key: SecretString,
}
