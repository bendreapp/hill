use crate::engagement::domain::error::EngagementError;
use crate::engagement::domain::port::EngagementEncryptionPort;
use crate::shared::encryption::EncryptionService;

/// Adapts the shared EncryptionService to the engagement feature's EncryptionPort.
pub struct EngagementEncryptionAdapter {
    inner: EncryptionService,
}

impl EngagementEncryptionAdapter {
    pub fn new(inner: EncryptionService) -> Self {
        Self { inner }
    }
}

impl EngagementEncryptionPort for EngagementEncryptionAdapter {
    fn encrypt(&self, plaintext: &str) -> Result<String, EngagementError> {
        self.inner
            .encrypt(plaintext)
            .map_err(|e| EngagementError::EncryptionFailed(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, EngagementError> {
        self.inner
            .decrypt(ciphertext)
            .map_err(|e| EngagementError::EncryptionFailed(e.to_string()))
    }
}
