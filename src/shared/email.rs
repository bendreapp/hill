/// Lightweight email utility using the Resend API.
/// Reads RESEND_API_KEY from env at call time (cached in AppConfig).

pub struct EmailParams<'a> {
    pub to: &'a str,
    pub reply_to: Option<&'a str>,
    pub from_name: &'a str,
    pub subject: &'a str,
    /// Plain text or HTML body. Will be auto-wrapped in Bendre branded shell.
    pub html: &'a str,
}

/// Wrap a body (plain text or HTML) in a branded HTML email shell.
/// Plain text: split on \n\n into paragraphs, \n becomes <br>.
/// Already-HTML content (starts with `<`): pass through.
/// Adds Bendre header + footer.
pub fn wrap_html(body: &str) -> String {
    let trimmed = body.trim();
    let body_html = if trimmed.starts_with('<') {
        // Already HTML — wrap in a paragraph-ish container but preserve content
        format!(r#"<div style="font-size: 15px; line-height: 1.6; color: #1C1C1E;">{}</div>"#, trimmed)
    } else {
        // Plain text — convert paragraphs
        let paragraphs: Vec<String> = trimmed
            .split("\n\n")
            .map(|p| {
                let escaped = escape_html(p.trim());
                format!(r#"<p style="margin: 0 0 16px 0; font-size: 15px; line-height: 1.6; color: #1C1C1E;">{}</p>"#, escaped.replace('\n', "<br>"))
            })
            .collect();
        paragraphs.join("")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
</head>
<body style="margin:0; padding:0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; background:#F4F1EC;">
  <table width="100%" cellpadding="0" cellspacing="0" role="presentation" style="background:#F4F1EC; padding:32px 16px;">
    <tr>
      <td align="center">
        <table width="560" cellpadding="0" cellspacing="0" role="presentation" style="max-width:560px; background:#FFFFFF; border-radius:12px; overflow:hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.06);">
          <tr>
            <td style="padding: 32px 40px 24px 40px; border-bottom: 1px solid #E5E0D8;">
              <div style="font-size: 20px; font-weight: 700; color:#1C1C1E; letter-spacing:-0.01em;">Bendre</div>
            </td>
          </tr>
          <tr>
            <td style="padding: 32px 40px;">
              {body_html}
            </td>
          </tr>
          <tr>
            <td style="padding: 20px 40px; background:#FAFAF8; border-top: 1px solid #E5E0D8;">
              <p style="margin:0; font-size:12px; color:#8A8480; line-height:1.5;">Sent via Bendre — practice management for therapists.</p>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"#,
        body_html = body_html
    )
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
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

    // Auto-wrap body in branded HTML shell
    let wrapped = wrap_html(params.html);

    let mut payload = serde_json::json!({
        "from": from,
        "to": [params.to],
        "subject": params.subject,
        "html": wrapped,
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
