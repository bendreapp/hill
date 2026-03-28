use async_trait::async_trait;

use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::BroadcastPort;

/// HTTP-based broadcast adapter for WhatsApp (Gupshup) and Email (Resend).
pub struct HttpBroadcastAdapter {
    whatsapp_api_key: String,
    whatsapp_source: String,
    email_api_key: String,
    email_from: String,
    client: reqwest::Client,
}

impl HttpBroadcastAdapter {
    pub fn new(
        whatsapp_api_key: String,
        whatsapp_source: String,
        email_api_key: String,
        email_from: String,
    ) -> Self {
        Self {
            whatsapp_api_key,
            whatsapp_source,
            email_api_key,
            email_from,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl BroadcastPort for HttpBroadcastAdapter {
    async fn send_whatsapp(
        &self,
        phone: &str,
        body: &str,
    ) -> Result<(), EngagementError> {
        let params = [
            ("channel", "whatsapp"),
            ("source", &self.whatsapp_source),
            ("destination", phone),
            ("message", body),
            ("src.name", "Bendre"),
        ];

        let response = self
            .client
            .post("https://api.gupshup.io/wa/api/v1/msg")
            .header("apikey", &self.whatsapp_api_key)
            .form(&params)
            .send()
            .await
            .map_err(|e| {
                EngagementError::BroadcastFailed(format!("WhatsApp HTTP error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(EngagementError::BroadcastFailed(format!(
                "WhatsApp API returned {}: {}",
                status, text
            )));
        }

        Ok(())
    }

    async fn send_email(
        &self,
        to: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EngagementError> {
        let payload = serde_json::json!({
            "from": self.email_from,
            "to": [to],
            "subject": subject,
            "html": body,
        });

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.email_api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                EngagementError::BroadcastFailed(format!("Email HTTP error: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            return Err(EngagementError::BroadcastFailed(format!(
                "Resend API returned {}: {}",
                status, text
            )));
        }

        Ok(())
    }
}
