use eyre::Report;
use secrecy::SecretString;
use std::env;

#[derive(Clone, Debug)]
pub struct JWTInfo {
    pub jwt_secret: SecretString,
    pub jwt_expiration_hours: i64,
    pub jwt_issuer: String,
    pub jwt_audience: String,
}

impl JWTInfo {
    pub fn new() -> Result<JWTInfo, Report> {
        use eyre::eyre;
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| eyre!("JWT_SECRET must be set in environment variables"))?;

        if jwt_secret.len() < 32 {
            return Err(eyre!("JWT_SECRET must be at least 32 characters long"));
        }

        Ok(Self {
            jwt_secret: SecretString::new(jwt_secret.into()),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "2".into())
                .parse()
                .map_err(|e| eyre!("Invalid JWT expiration configuration: {}", e))?,

            jwt_issuer: env::var("ISSUER")
                .map_err(|_| eyre!("ISSUER environment variable not set"))?,

            jwt_audience: env::var("AUDIENCE")
                .map_err(|_| eyre!("AUDIENCE environment variable not set"))?,
        })
    }
}
