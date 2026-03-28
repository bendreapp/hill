use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

use super::error::AppError;

/// AES-256-GCM encryption service.
/// Wire format is byte-compatible with the TypeScript implementation:
/// `Base64( [version: 1 byte] [iv: 12 bytes] [authTag: 16 bytes] [ciphertext: N bytes] )`
#[derive(Clone)]
pub struct EncryptionService {
    key: Key<Aes256Gcm>,
}

const VERSION_BYTE: u8 = 0x01;
const IV_LEN: usize = 12;
const AUTH_TAG_LEN: usize = 16;

impl EncryptionService {
    /// Create from a 32-byte base64-encoded key (matches ENCRYPTION_KEY env var).
    pub fn from_base64_key(key_b64: &str) -> Result<Self, AppError> {
        let key_bytes = BASE64
            .decode(key_b64)
            .map_err(|_| AppError::Encryption {
                message: "Invalid base64 encryption key".to_string(),
            })?;

        if key_bytes.len() != 32 {
            return Err(AppError::Encryption {
                message: format!("Encryption key must be 32 bytes, got {}", key_bytes.len()),
            });
        }

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes).clone();
        Ok(Self { key })
    }

    /// Encrypt plaintext → base64 string (versioned format).
    pub fn encrypt(&self, plaintext: &str) -> Result<String, AppError> {
        let cipher = Aes256Gcm::new(&self.key);

        // Generate random 12-byte IV
        let iv_bytes: [u8; IV_LEN] = rand::random();
        let nonce = Nonce::from_slice(&iv_bytes);

        // AES-GCM encrypt (ciphertext includes auth tag appended by aes-gcm crate)
        let ciphertext_with_tag = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Encryption {
                message: format!("Encryption failed: {}", e),
            })?;

        // aes-gcm appends the 16-byte auth tag to the ciphertext.
        // We need to split them to match the TS wire format: version || iv || authTag || ciphertext
        let ct_len = ciphertext_with_tag.len() - AUTH_TAG_LEN;
        let ciphertext = &ciphertext_with_tag[..ct_len];
        let auth_tag = &ciphertext_with_tag[ct_len..];

        // Wire format: [version(1)] [iv(12)] [authTag(16)] [ciphertext(N)]
        let mut output = Vec::with_capacity(1 + IV_LEN + AUTH_TAG_LEN + ciphertext.len());
        output.push(VERSION_BYTE);
        output.extend_from_slice(&iv_bytes);
        output.extend_from_slice(auth_tag);
        output.extend_from_slice(ciphertext);

        Ok(BASE64.encode(&output))
    }

    /// Decrypt base64 string → plaintext. Handles both versioned and legacy formats.
    pub fn decrypt(&self, data: &str) -> Result<String, AppError> {
        let bytes = BASE64.decode(data).map_err(|_| AppError::Encryption {
            message: "Invalid base64 in encrypted data".to_string(),
        })?;

        if bytes.is_empty() {
            return Ok(String::new());
        }

        // Determine format: versioned (starts with 0x01) or legacy
        let (iv, auth_tag, ciphertext) = if bytes[0] == VERSION_BYTE {
            // Versioned: [version(1)] [iv(12)] [authTag(16)] [ciphertext(N)]
            if bytes.len() < 1 + IV_LEN + AUTH_TAG_LEN {
                return Err(AppError::Encryption {
                    message: "Encrypted data too short (versioned)".to_string(),
                });
            }
            (
                &bytes[1..1 + IV_LEN],
                &bytes[1 + IV_LEN..1 + IV_LEN + AUTH_TAG_LEN],
                &bytes[1 + IV_LEN + AUTH_TAG_LEN..],
            )
        } else {
            // Legacy: [iv(12)] [authTag(16)] [ciphertext(N)]
            if bytes.len() < IV_LEN + AUTH_TAG_LEN {
                // Not encrypted data — return as-is (legacy unencrypted)
                return String::from_utf8(bytes).map_err(|_| AppError::Encryption {
                    message: "Data is neither encrypted nor valid UTF-8".to_string(),
                });
            }
            (
                &bytes[..IV_LEN],
                &bytes[IV_LEN..IV_LEN + AUTH_TAG_LEN],
                &bytes[IV_LEN + AUTH_TAG_LEN..],
            )
        };

        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Nonce::from_slice(iv);

        // aes-gcm expects ciphertext || authTag
        let mut ct_with_tag = Vec::with_capacity(ciphertext.len() + AUTH_TAG_LEN);
        ct_with_tag.extend_from_slice(ciphertext);
        ct_with_tag.extend_from_slice(auth_tag);

        let plaintext = cipher
            .decrypt(nonce, ct_with_tag.as_ref())
            .map_err(|_| AppError::Encryption {
                message: "Decryption failed — wrong key or corrupted data".to_string(),
            })?;

        String::from_utf8(plaintext).map_err(|_| AppError::Encryption {
            message: "Decrypted data is not valid UTF-8".to_string(),
        })
    }

    /// Encrypt a JSON-serializable value.
    pub fn encrypt_json<T: serde::Serialize>(&self, value: &T) -> Result<String, AppError> {
        let json = serde_json::to_string(value).map_err(|e| AppError::Encryption {
            message: format!("JSON serialization failed: {}", e),
        })?;
        self.encrypt(&json)
    }

    /// Decrypt to a JSON-deserializable value.
    pub fn decrypt_json<T: serde::de::DeserializeOwned>(&self, data: &str) -> Result<T, AppError> {
        let json = self.decrypt(data)?;
        serde_json::from_str(&json).map_err(|e| AppError::Encryption {
            message: format!("JSON deserialization failed: {}", e),
        })
    }
}
