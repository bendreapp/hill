use actix_web::{dev::Payload, FromRequest, HttpRequest};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use super::error::AppError;
use super::types::AuthUser;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    email: Option<String>,
    role: Option<String>,
}

/// Holds the JWT decoding keys (supports both ES256 and legacy HS256).
#[derive(Clone)]
pub struct JwtKeys {
    /// ES256 keys from JWKS endpoint, keyed by kid
    pub es256_keys: Vec<(String, DecodingKey)>,
    /// Legacy HS256 shared secret (if still in use)
    pub hs256_secret: Option<DecodingKey>,
}

/// JWKS response format from Supabase
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    alg: Option<String>,
    kid: Option<String>,
    kty: String,
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
}

impl JwtKeys {
    /// Fetch JWKS from Supabase and build decoding keys.
    pub async fn from_supabase(supabase_url: &str, jwt_secret: &str) -> Result<Self, AppError> {
        let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);

        let mut es256_keys = Vec::new();

        // Fetch JWKS
        match reqwest::get(&jwks_url).await {
            Ok(resp) => {
                if let Ok(jwks) = resp.json::<JwksResponse>().await {
                    for key in jwks.keys {
                        if key.kty == "EC" && key.crv.as_deref() == Some("P-256") {
                            if let (Some(x), Some(y)) = (&key.x, &key.y) {
                                if let Ok(decoding_key) =
                                    DecodingKey::from_ec_components(x, y)
                                {
                                    let kid = key.kid.unwrap_or_default();
                                    tracing::info!("Loaded ES256 key: kid={}", kid);
                                    es256_keys.push((kid, decoding_key));
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch JWKS: {}, falling back to HS256", e);
            }
        }

        // Also keep HS256 fallback for legacy tokens
        let hs256_secret = if !jwt_secret.is_empty() {
            Some(DecodingKey::from_secret(jwt_secret.as_bytes()))
        } else {
            None
        };

        if es256_keys.is_empty() && hs256_secret.is_none() {
            return Err(AppError::internal("No JWT keys available"));
        }

        Ok(Self {
            es256_keys,
            hs256_secret,
        })
    }

    /// Verify a JWT token — tries ES256 first (matching kid), then HS256 fallback.
    pub fn verify(&self, token: &str) -> Result<AuthUser, AppError> {
        // Decode header to check algorithm and kid
        let header = decode_header(token).map_err(|e| {
            tracing::warn!("JWT header decode failed: {:?}", e);
            AppError::unauthorized("Invalid token header")
        })?;

        // Try ES256 if header says so
        if header.alg == Algorithm::ES256 {
            let mut validation = Validation::new(Algorithm::ES256);
            validation.set_audience(&["authenticated"]);
            validation.validate_exp = true;

            for (kid, key) in &self.es256_keys {
                // Match by kid if present
                if let Some(ref token_kid) = header.kid {
                    if token_kid != kid {
                        continue;
                    }
                }

                if let Ok(token_data) = decode::<Claims>(token, key, &validation) {
                    return Self::claims_to_user(token_data.claims);
                }
            }
        }

        // Try HS256 fallback (legacy tokens)
        if let Some(ref secret) = self.hs256_secret {
            let mut validation = Validation::new(Algorithm::HS256);
            validation.set_audience(&["authenticated"]);
            validation.validate_exp = true;

            if let Ok(token_data) = decode::<Claims>(token, secret, &validation) {
                return Self::claims_to_user(token_data.claims);
            }
        }

        tracing::warn!("JWT validation failed for all keys");
        Err(AppError::unauthorized("Invalid or expired token"))
    }

    fn claims_to_user(claims: Claims) -> Result<AuthUser, AppError> {
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::unauthorized("Invalid user ID in token"))?;

        Ok(AuthUser {
            id: user_id,
            email: claims.email.unwrap_or_default(),
            role: claims.role.unwrap_or_else(|| "authenticated".to_string()),
        })
    }
}

/// Actix `FromRequest` extractor that validates the Supabase JWT
/// and produces an `AuthUser`.
impl FromRequest for AuthUser {
    type Error = AppError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let jwt_keys = req
            .app_data::<actix_web::web::Data<Arc<JwtKeys>>>()
            .cloned();

        Box::pin(async move {
            let header = auth_header
                .ok_or_else(|| AppError::unauthorized("Missing Authorization header"))?;

            let token = header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::unauthorized("Invalid Authorization header format"))?;

            let keys = jwt_keys
                .ok_or_else(|| AppError::internal("JWT keys not configured"))?;

            keys.verify(token)
        })
    }
}
