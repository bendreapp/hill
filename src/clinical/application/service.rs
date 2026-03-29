use std::sync::Arc;
use uuid::Uuid;

use crate::clinical::domain::entity::*;
use crate::clinical::domain::error::ClinicalError;
use crate::clinical::domain::port::*;

// ─── Note Service ───────────────────────────────────────────────────────────

pub struct NoteService {
    pub note_repo: Arc<dyn NoteRepository>,
    pub encryption: Arc<dyn EncryptionPort>,
}

impl NoteService {
    pub fn new(
        note_repo: Arc<dyn NoteRepository>,
        encryption: Arc<dyn EncryptionPort>,
    ) -> Self {
        Self {
            note_repo,
            encryption,
        }
    }

    pub async fn get_note(&self, id: Uuid, therapist_id: Uuid) -> Result<SessionNote, ClinicalError> {
        let mut note = self
            .note_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::NoteNotFound)?;
        self.decrypt_note_fields(&mut note);
        Ok(note)
    }

    pub async fn get_note_by_session(&self, session_id: Uuid, therapist_id: Uuid) -> Result<Option<SessionNote>, ClinicalError> {
        let mut note = self.note_repo.find_by_session(session_id, therapist_id).await?;
        if let Some(ref mut n) = note {
            self.decrypt_note_fields(n);
        }
        Ok(note)
    }

    pub async fn list_notes(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<SessionNote>, i64), ClinicalError> {
        let (mut notes, total) = self.note_repo.list_by_therapist(therapist_ids, limit, offset).await?;
        for note in &mut notes {
            self.decrypt_note_fields(note);
        }
        Ok((notes, total))
    }

    pub async fn create_note(
        &self,
        therapist_id: Uuid,
        mut input: CreateNoteInput,
    ) -> Result<SessionNote, ClinicalError> {
        // Encrypt sensitive fields before insert
        if let Some(ref val) = input.techniques_used {
            input.techniques_used = Some(self.encryption.encrypt(val)?);
        }
        if let Some(ref val) = input.risk_flags {
            input.risk_flags = Some(self.encryption.encrypt(val)?);
        }

        let mut note = self.note_repo.create(therapist_id, &input).await?;
        self.decrypt_note_fields(&mut note);
        Ok(note)
    }

    pub async fn update_note(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        mut input: UpdateNoteInput,
    ) -> Result<SessionNote, ClinicalError> {
        self.note_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::NoteNotFound)?;

        // Encrypt sensitive fields before update
        if let Some(ref val) = input.techniques_used {
            input.techniques_used = Some(self.encryption.encrypt(val)?);
        }
        if let Some(ref val) = input.risk_flags {
            input.risk_flags = Some(self.encryption.encrypt(val)?);
        }

        let mut note = self.note_repo.update(id, therapist_id, &input).await?;
        self.decrypt_note_fields(&mut note);
        Ok(note)
    }

    pub async fn delete_note(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError> {
        self.note_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::NoteNotFound)?;
        self.note_repo.soft_delete(id, therapist_id).await
    }

    fn decrypt_note_fields(&self, note: &mut SessionNote) {
        if let Some(ref val) = note.techniques_used {
            note.techniques_used = self.encryption.decrypt(val).ok();
        }
        if let Some(ref val) = note.risk_flags {
            note.risk_flags = self.encryption.decrypt(val).ok();
        }
    }
}

// ─── Treatment Plan Service ─────────────────────────────────────────────────

pub struct TreatmentPlanService {
    pub plan_repo: Arc<dyn TreatmentPlanRepository>,
    pub encryption: Arc<dyn EncryptionPort>,
}

impl TreatmentPlanService {
    pub fn new(
        plan_repo: Arc<dyn TreatmentPlanRepository>,
        encryption: Arc<dyn EncryptionPort>,
    ) -> Self {
        Self {
            plan_repo,
            encryption,
        }
    }

    pub async fn get_plan(&self, id: Uuid, therapist_id: Uuid) -> Result<TreatmentPlan, ClinicalError> {
        let mut plan = self
            .plan_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::PlanNotFound)?;
        self.decrypt_plan_fields(&mut plan);
        Ok(plan)
    }

    pub async fn list_by_client(
        &self,
        client_id: Uuid,
        therapist_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError> {
        let (mut plans, total) = self.plan_repo.list_by_client(client_id, therapist_id, limit, offset).await?;
        for plan in &mut plans {
            self.decrypt_plan_fields(plan);
        }
        Ok((plans, total))
    }

    pub async fn list_by_therapist(
        &self,
        therapist_ids: &[Uuid],
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<TreatmentPlan>, i64), ClinicalError> {
        let (mut plans, total) = self.plan_repo.list_by_therapist(therapist_ids, limit, offset).await?;
        for plan in &mut plans {
            self.decrypt_plan_fields(plan);
        }
        Ok((plans, total))
    }

    pub async fn create_plan(
        &self,
        therapist_id: Uuid,
        mut input: CreateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError> {
        if let Some(ref val) = input.goals {
            input.goals = Some(self.encryption.encrypt(val)?);
        }
        let mut plan = self.plan_repo.create(therapist_id, &input).await?;
        self.decrypt_plan_fields(&mut plan);
        Ok(plan)
    }

    pub async fn update_plan(
        &self,
        id: Uuid,
        therapist_id: Uuid,
        mut input: UpdateTreatmentPlanInput,
    ) -> Result<TreatmentPlan, ClinicalError> {
        self.plan_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::PlanNotFound)?;

        if let Some(ref val) = input.goals {
            input.goals = Some(self.encryption.encrypt(val)?);
        }
        let mut plan = self.plan_repo.update(id, therapist_id, &input).await?;
        self.decrypt_plan_fields(&mut plan);
        Ok(plan)
    }

    pub async fn delete_plan(&self, id: Uuid, therapist_id: Uuid) -> Result<(), ClinicalError> {
        self.plan_repo
            .find_by_id(id, therapist_id)
            .await?
            .ok_or(ClinicalError::PlanNotFound)?;
        self.plan_repo.soft_delete(id, therapist_id).await
    }

    fn decrypt_plan_fields(&self, plan: &mut TreatmentPlan) {
        if let Some(ref val) = plan.goals {
            plan.goals = self.encryption.decrypt(val).ok();
        }
    }
}

// ─── Message Service ────────────────────────────────────────────────────────

pub struct MessageService {
    pub message_repo: Arc<dyn MessageRepository>,
    pub encryption: Arc<dyn EncryptionPort>,
}

impl MessageService {
    pub fn new(
        message_repo: Arc<dyn MessageRepository>,
        encryption: Arc<dyn EncryptionPort>,
    ) -> Self {
        Self {
            message_repo,
            encryption,
        }
    }

    pub async fn get_message(&self, id: Uuid) -> Result<Message, ClinicalError> {
        let mut msg = self
            .message_repo
            .find_by_id(id)
            .await?
            .ok_or(ClinicalError::MessageNotFound)?;
        self.decrypt_message(&mut msg);
        Ok(msg)
    }

    pub async fn list_thread(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Message>, i64), ClinicalError> {
        let (mut msgs, total) = self
            .message_repo
            .list_thread(therapist_id, client_id, limit, offset)
            .await?;
        for msg in &mut msgs {
            self.decrypt_message(msg);
        }
        Ok((msgs, total))
    }

    pub async fn unread_count(&self, therapist_id: Uuid) -> Result<i64, ClinicalError> {
        self.message_repo.list_unread_count(therapist_id).await
    }

    pub async fn send_message(
        &self,
        therapist_id: Uuid,
        mut input: CreateMessageInput,
    ) -> Result<Message, ClinicalError> {
        input.content = self.encryption.encrypt(&input.content)?;
        let mut msg = self.message_repo.create(therapist_id, &input).await?;
        self.decrypt_message(&mut msg);
        Ok(msg)
    }

    pub async fn mark_read(&self, therapist_id: Uuid, message_ids: &[Uuid]) -> Result<(), ClinicalError> {
        self.message_repo.mark_read(therapist_id, message_ids).await
    }

    pub async fn delete_message(&self, id: Uuid) -> Result<(), ClinicalError> {
        self.message_repo
            .find_by_id(id)
            .await?
            .ok_or(ClinicalError::MessageNotFound)?;
        self.message_repo.soft_delete(id).await
    }

    fn decrypt_message(&self, msg: &mut Message) {
        if let Ok(decrypted) = self.encryption.decrypt(&msg.content) {
            msg.content = decrypted;
        }
    }
}
