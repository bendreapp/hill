/// Lightweight email utility using the Resend API.
/// Reads RESEND_API_KEY from env at call time (cached in AppConfig).

pub struct EmailParams<'a> {
    pub to: &'a str,
    pub reply_to: Option<&'a str>,
    pub from_name: &'a str,
    pub subject: &'a str,
    pub html: &'a str,
}

/// Send an email via Resend. Best-effort: returns Ok(()) on success, Err on
/// permanent failure. Callers should log errors and continue — do not let email
/// failures bubble up as HTTP 5xx.
pub async fn send_email(api_key: &str, params: EmailParams<'_>) -> Result<(), String> {
    if api_key.is_empty() {
        return Err("RESEND_API_KEY not configured".to_string());
    }

    let from_domain = std::env::var("EMAIL_FROM_DOMAIN")
        .unwrap_or_else(|_| "onboarding@resend.dev".to_string());
    let from = format!("{} <{}>", params.from_name, from_domain);

    let mut payload = serde_json::json!({
        "from": from,
        "to": [params.to],
        "subject": params.subject,
        "html": params.html,
    });

    if let Some(reply_to) = params.reply_to {
        payload["reply_to"] = serde_json::json!(reply_to);
    }

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.resend.com/emails")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Resend HTTP error: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_else(|_| "unknown".to_string());
        return Err(format!("Resend API returned {}: {}", status, text));
    }

    Ok(())
}
