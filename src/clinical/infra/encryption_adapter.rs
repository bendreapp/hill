use crate::clinical::domain::error::ClinicalError;
use crate::clinical::domain::port::EncryptionPort;
use crate::shared::encryption::EncryptionService;

/// Adapts the shared EncryptionService to the clinical feature's EncryptionPort.
pub struct EncryptionAdapter {
    inner: EncryptionService,
}

impl EncryptionAdapter {
    pub fn new(inner: EncryptionService) -> Self {
        Self { inner }
    }
}

impl EncryptionPort for EncryptionAdapter {
    fn encrypt(&self, plaintext: &str) -> Result<String, ClinicalError> {
        self.inner
            .encrypt(plaintext)
            .map_err(|e| ClinicalError::EncryptionFailed(e.to_string()))
    }

    fn decrypt(&self, ciphertext: &str) -> Result<String, ClinicalError> {
        self.inner
            .decrypt(ciphertext)
            .map_err(|e| ClinicalError::EncryptionFailed(e.to_string()))
    }
}
