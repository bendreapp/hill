use async_trait::async_trait;
use uuid::Uuid;

use super::entity::*;
use super::error::ClinicalError;

// ─── Note Repository ────────────────────────────────────────────────────────

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<SessionNote>, ClinicalError>;

    async fn find_by_session(&self, session_id: Uuid, therapist_id: Uuid) -> Result<Option<SessionNote>, ClinicalError>;

    async fn list_by_therapist(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SessionNote>, i64), ClinicalError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateNoteInput,
    ) -> Result<SessionNote, ClinicalError>;

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateNoteInput,
    ) -> Result<SessionNote, ClinicalError>;

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError>;
}

// ─── Treatment Plan Repository ──────────────────────────────────────────────

#[async_trait]
pub trait TreatmentPlanRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid, therapist_id: Uuid) -> Result<Option<TreatmentPlan>, ClinicalError>;

    async fn list_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError>;

    async fn list_by_therapist(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError>;

    async fn update(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        input: &UpdateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError>;

    async fn soft_delete(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError>;
}

// ─── Message Repository ─────────────────────────────────────────────────────

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, ClinicalError>;

    async fn list_thread(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64), ClinicalError>;

    async fn list_unread_count(
        &self,
        therapist_id: Uuid,
    ) -> Result<i64, ClinicalError>;

    async fn create(
        &self,
        therapist_id: Uuid,
        input: &CreateMessageInput,
    ) -> Result<Message, ClinicalError>;

    async fn mark_read(&self, therapist_id: Uuid, message_ids: &[Uuid]) -> Result<(), ClinicalError>;

    async fn soft_delete(&self, id: Uuid) -> Result<(), ClinicalError>;
}

// ─── Encryption Port ────────────────────────────────────────────────────────

pub trait EncryptionPort: Send + Sync {
    fn encrypt(&self, plaintext: &str) -> Result<String, ClinicalError>;
    fn decrypt(&self, ciphertext: &str) -> Result<String, ClinicalError>;
}
