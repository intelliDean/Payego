use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use payego_primitives::error::ApiError;
use std::env;

#[derive(Clone)]
pub struct EmailClient {
    transport: Option<SmtpTransport>,
    from_email: String,
}

impl Default for EmailClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailClient {
    pub fn new() -> Self {
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(587);
        let smtp_user = env::var("SMTP_USER").ok();
        let smtp_pass = env::var("SMTP_PASS").ok();
        let from_email = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@payego.com".to_string());

        let transport =
            if let (Some(host), Some(user), Some(pass)) = (smtp_host, smtp_user, smtp_pass) {
                let creds = Credentials::new(user, pass);
                match SmtpTransport::starttls_relay(&host) {
                    Ok(builder) => Some(builder.credentials(creds).port(smtp_port).build()),
                    Err(e) => {
                        tracing::error!(
                            "Failed to initialize STARTTLS relay for host {}: {}",
                            host,
                            e
                        );
                        None
                    }
                }
            } else {
                tracing::warn!("SMTP configuration missing, email client running in mock mode");
                None
            };

        Self {
            transport,
            from_email,
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), ApiError> {
        if let Some(ref transport) = self.transport {
            let email = Message::builder()
                .from(
                    self.from_email
                        .parse()
                        .map_err(|e| ApiError::Internal(format!("Invalid from email: {}", e)))?,
                )
                .to(to
                    .parse()
                    .map_err(|e| ApiError::Internal(format!("Invalid recipient email: {}", e)))?)
                .subject(subject)
                .header(lettre::message::header::ContentType::TEXT_HTML)
                .body(body.to_string())
                .map_err(|e| ApiError::Internal(format!("Failed to build email: {}", e)))?;

            transport.send(&email).map_err(|e| {
                tracing::error!("Failed to send email: {}", e);
                ApiError::Internal("Failed to send email".to_string())
            })?;

            tracing::info!("Email sent successfully to: {}", to);
        } else {
            tracing::info!(
                "[MOCK EMAIL] To: {}, Subject: {}, Body: {}",
                to,
                subject,
                body
            );
        }

        Ok(())
    }
}
