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
        // Encrypt all clinical fields before insert
        self.encrypt_note_input_fields(&mut input)?;

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

        // Encrypt all clinical fields before update
        self.encrypt_update_note_fields(&mut input)?;

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

    fn encrypt_optional(&self, val: &Option<String>) -> Result<Option<String>, ClinicalError> {
        match val {
            Some(v) => Ok(Some(self.encryption.encrypt(v)?)),
            None => Ok(None),
        }
    }

    fn decrypt_optional(&self, val: &Option<String>) -> Option<String> {
        val.as_ref().and_then(|v| self.encryption.decrypt(v).ok())
    }

    fn encrypt_note_input_fields(&self, input: &mut CreateNoteInput) -> Result<(), ClinicalError> {
        input.subjective = self.encrypt_optional(&input.subjective)?;
        input.objective = self.encrypt_optional(&input.objective)?;
        input.assessment = self.encrypt_optional(&input.assessment)?;
        input.plan = self.encrypt_optional(&input.plan)?;
        input.freeform_content = self.encrypt_optional(&input.freeform_content)?;
        input.homework = self.encrypt_optional(&input.homework)?;
        input.techniques_used = self.encrypt_optional(&input.techniques_used)?;
        input.risk_flags = self.encrypt_optional(&input.risk_flags)?;
        Ok(())
    }

    fn encrypt_update_note_fields(&self, input: &mut UpdateNoteInput) -> Result<(), ClinicalError> {
        input.subjective = self.encrypt_optional(&input.subjective)?;
        input.objective = self.encrypt_optional(&input.objective)?;
        input.assessment = self.encrypt_optional(&input.assessment)?;
        input.plan = self.encrypt_optional(&input.plan)?;
        input.freeform_content = self.encrypt_optional(&input.freeform_content)?;
        input.homework = self.encrypt_optional(&input.homework)?;
        input.techniques_used = self.encrypt_optional(&input.techniques_used)?;
        input.risk_flags = self.encrypt_optional(&input.risk_flags)?;
        Ok(())
    }

    fn decrypt_note_fields(&self, note: &mut SessionNote) {
        note.subjective = self.decrypt_optional(&note.subjective);
        note.objective = self.decrypt_optional(&note.objective);
        note.assessment = self.decrypt_optional(&note.assessment);
        note.plan = self.decrypt_optional(&note.plan);
        note.freeform_content = self.decrypt_optional(&note.freeform_content);
        note.homework = self.decrypt_optional(&note.homework);
        note.techniques_used = self.decrypt_optional(&note.techniques_used);
        note.risk_flags = self.decrypt_optional(&note.risk_flags);
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
        self.encrypt_plan_input_fields(&mut input)?;
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

        self.encrypt_update_plan_fields(&mut input)?;
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

    fn encrypt_optional(&self, val: &Option<String>) -> Result<Option<String>, ClinicalError> {
        match val {
            Some(v) => Ok(Some(self.encryption.encrypt(v)?)),
            None => Ok(None),
        }
    }

    fn decrypt_optional(&self, val: &Option<String>) -> Option<String> {
        val.as_ref().and_then(|v| self.encryption.decrypt(v).ok())
    }

    fn encrypt_plan_input_fields(&self, input: &mut CreateTreatmentPlanInput) -> Result<(), ClinicalError> {
        input.presenting_concerns = self.encrypt_optional(&input.presenting_concerns)?;
        input.diagnosis = self.encrypt_optional(&input.diagnosis)?;
        input.goals = self.encrypt_optional(&input.goals)?;
        input.notes = self.encrypt_optional(&input.notes)?;
        Ok(())
    }

    fn encrypt_update_plan_fields(&self, input: &mut UpdateTreatmentPlanInput) -> Result<(), ClinicalError> {
        input.presenting_concerns = self.encrypt_optional(&input.presenting_concerns)?;
        input.diagnosis = self.encrypt_optional(&input.diagnosis)?;
        input.goals = self.encrypt_optional(&input.goals)?;
        input.notes = self.encrypt_optional(&input.notes)?;
        Ok(())
    }

    fn decrypt_plan_fields(&self, plan: &mut TreatmentPlan) {
        plan.presenting_concerns = self.decrypt_optional(&plan.presenting_concerns);
        plan.diagnosis = self.decrypt_optional(&plan.diagnosis);
        plan.goals = self.decrypt_optional(&plan.goals);
        plan.notes = self.decrypt_optional(&plan.notes);
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
