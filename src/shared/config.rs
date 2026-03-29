/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,

    // Database
    pub database_url: String,

    // Supabase Auth
    pub supabase_jwt_secret: String, // Legacy HS256 fallback, can be empty
    pub supabase_url: String,
    pub supabase_anon_key: String,

    // Encryption
    pub encryption_key: String,

    // PostHog
    pub posthog_api_key: String,
    pub posthog_host: String,

    // Integrations (optional — only needed when working on those features)
    pub zoom_client_id: Option<String>,
    pub zoom_client_secret: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub razorpay_key_id: Option<String>,
    pub razorpay_key_secret: Option<String>,
    pub gupshup_api_key: Option<String>,
    pub gupshup_app_name: Option<String>,
    pub resend_api_key: Option<String>,

    // Frontend URL (for CORS)
    pub frontend_url: String,

    // Admin
    pub admin_emails: Vec<String>,
    pub hq_frontend_url: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),

            database_url: std::env::var("DATABASE_URL")?,
            supabase_jwt_secret: std::env::var("SUPABASE_JWT_SECRET").unwrap_or_default(),
            supabase_url: std::env::var("SUPABASE_URL")?,
            supabase_anon_key: std::env::var("SUPABASE_ANON_KEY")?,
            encryption_key: std::env::var("ENCRYPTION_KEY")?,

            posthog_api_key: std::env::var("POSTHOG_API_KEY").unwrap_or_default(),
            posthog_host: std::env::var("POSTHOG_HOST")
                .unwrap_or_else(|_| "https://us.i.posthog.com".to_string()),

            zoom_client_id: std::env::var("ZOOM_CLIENT_ID").ok(),
            zoom_client_secret: std::env::var("ZOOM_CLIENT_SECRET").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            razorpay_key_id: std::env::var("RAZORPAY_KEY_ID").ok(),
            razorpay_key_secret: std::env::var("RAZORPAY_KEY_SECRET").ok(),
            gupshup_api_key: std::env::var("GUPSHUP_API_KEY").ok(),
            gupshup_app_name: std::env::var("GUPSHUP_APP_NAME").ok(),
            resend_api_key: std::env::var("RESEND_API_KEY").ok(),

            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            admin_emails: std::env::var("ADMIN_EMAILS")
                .unwrap_or_default()
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect(),
            hq_frontend_url: std::env::var("HQ_FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
        })
    }
}
