use payego_primitives::error::ApiError;
// use lettre::{SmtpTransport, Transport, Message};
// use secrecy::ExposeSecret;

#[derive(Clone)]
pub struct EmailClient {
    // transport: SmtpTransport,
}

impl Default for EmailClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EmailClient {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn send_email(&self, _to: &str, _subject: &str, _body: &str) -> Result<(), ApiError> {
        // Placeholder for real email sending logic
        tracing::info!("Sending email to: {}, subject: {}", _to, _subject);
        Ok(())
    }
}
