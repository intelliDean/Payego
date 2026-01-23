use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use payego_primitives::models::entities::enum_types::CurrencyCode;
use payego_primitives::models::entities::user::{NewUser, User};
use payego_primitives::models::entities::wallet::NewWallet;
use std::env;
use std::str::FromStr;
use uuid::Uuid;

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    dotenv().ok();
    println!("🌱 Seeding database...");

    let mut conn = establish_connection();

    // 1. Clean DB
    clean_db(&mut conn);

    // 2. Seed Users
    let user_id = seed_user(&mut conn, "test@payego.com", "Test User", "password123");
    let admin_id = seed_user(&mut conn, "admin@payego.com", "Admin User", "admin123");

    // 3. Seed Wallets
    seed_wallet(&mut conn, user_id, "USD", 100000); // $1000.00
    seed_wallet(&mut conn, user_id, "NGN", 5000000); // ₦50,000.00
                                                     // Admin gets rich
    seed_wallet(&mut conn, admin_id, "USD", 100000000); // $1M

    println!("✅ Database seeded successfully!");
}

fn clean_db(conn: &mut PgConnection) {
    use diesel::sql_query;
    println!("🧹 Cleaning database...");
    sql_query("TRUNCATE users, wallets, transactions, bank_accounts, blacklisted_tokens CASCADE")
        .execute(conn)
        .expect("Error truncating tables");
}

fn seed_user(conn: &mut PgConnection, u_email: &str, u_username: &str, u_password: &str) -> Uuid {
    use payego_primitives::schema::users;
    use payego_primitives::schema::users::dsl::*;

    // Check if user exists
    let existing = users
        .filter(email.eq(u_email))
        .first::<User>(conn)
        .optional()
        .unwrap();

    if let Some(user) = existing {
        println!("User {} already exists", u_email);
        return user.id;
    }

    let hashed = hash_password(u_password);

    let new_user = NewUser {
        email: u_email,
        password_hash: &hashed,
        username: Some(u_username),
    };

    let inserted_user: User = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(conn)
        .expect("Error inserting new user");

    println!("Created user: {}", u_email);
    inserted_user.id
}

fn seed_wallet(conn: &mut PgConnection, u_id: Uuid, curr: &str, amt: i64) {
    use payego_primitives::schema::wallets;

    let currency_enum = CurrencyCode::from_str(curr).expect("Invalid currency code");

    let new_wallet = NewWallet {
        user_id: u_id,
        currency: currency_enum,
    };

    // We want to set balance, but NewWallet might not have balance field?
    // Let's check NewWallet definition again.
    // It only has user_id and currency. Balance likely defaults to 0.
    // If I want to seed with balance, I might need to update it after insert OR add balance to NewWallet if possible (but I can't change primitives easily).
    // Or I can insert then update.

    let _ = diesel::insert_into(wallets::table)
        .values(&new_wallet)
        .execute(conn)
        .expect("Error inserting wallet");

    // Update balance
    // Need to find the wallet we just inserted or assume we can target it by user+currency
    use payego_primitives::schema::wallets::dsl::*;

    diesel::update(
        wallets
            .filter(user_id.eq(u_id))
            .filter(currency.eq(currency_enum)),
    )
    .set(balance.eq(amt))
    .execute(conn)
    .expect("Error updating wallet balance");

    println!("Created {} wallet for user {}", curr, u_id);
}
