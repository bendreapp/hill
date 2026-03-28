use std::sync::Arc;

use chrono::{DateTime, Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;
use uuid::Uuid;

use crate::scheduling::domain::entity::*;
use crate::scheduling::domain::error::SchedulingError;
use crate::scheduling::domain::port::*;

// ─── Session Service ────────────────────────────────────────────────────────

pub struct SessionService {
    pub session_repo: Arc<dyn SessionRepository>,
}

impl SessionService {
    pub fn new(session_repo: Arc<dyn SessionRepository>) -> Self {
        Self { session_repo }
    }

    pub async fn get_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<Session, SchedulingError> {
        self.session_repo
            .find_by_id(therapist_id, id)
            .await?
            .ok_or(SchedulingError::SessionNotFound)
    }

    pub async fn list_by_date_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError> {
        self.session_repo
            .list_by_date_range(therapist_id, start, end)
            .await
    }

    pub async fn list_pending(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<Session>, SchedulingError> {
        self.session_repo.list_pending(therapist_id).await
    }

    pub async fn list_today(
        &self,
        therapist_id: Uuid,
        day_start: DateTime<Utc>,
        day_end: DateTime<Utc>,
    ) -> Result<Vec<Session>, SchedulingError> {
        self.session_repo
            .list_today(therapist_id, day_start, day_end)
            .await
    }

    pub async fn list_upcoming(
        &self,
        therapist_id: Uuid,
        limit: i64,
    ) -> Result<Vec<Session>, SchedulingError> {
        self.session_repo
            .list_upcoming(therapist_id, Utc::now(), limit)
            .await
    }

    pub async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<Session>, SchedulingError> {
        self.session_repo
            .list_by_client(therapist_id, client_id)
            .await
    }

    pub async fn approve(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
    ) -> Result<Session, SchedulingError> {
        let session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "pending_approval" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot approve session with status '{}'",
                session.status
            )));
        }
        self.session_repo
            .update_status(therapist_id, session_id, "scheduled", None, None, None)
            .await
    }

    pub async fn reject(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
        reason: Option<&str>,
    ) -> Result<Session, SchedulingError> {
        let session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "pending_approval" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot reject session with status '{}'",
                session.status
            )));
        }
        self.session_repo
            .update_status(
                therapist_id,
                session_id,
                "cancelled",
                reason,
                Some("therapist"),
                Some(false),
            )
            .await
    }

    pub async fn cancel(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
        reason: Option<&str>,
        cancelled_by: &str,
        cancellation_hours: i32,
    ) -> Result<Session, SchedulingError> {
        let session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "scheduled" && session.status != "pending_approval" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot cancel session with status '{}'",
                session.status
            )));
        }

        // Detect late cancellation: if less than cancellation_hours before session start
        let hours_until_session = (session.starts_at - Utc::now()).num_hours();
        let is_late = hours_until_session < cancellation_hours as i64;

        self.session_repo
            .update_status(
                therapist_id,
                session_id,
                "cancelled",
                reason,
                Some(cancelled_by),
                Some(is_late),
            )
            .await
    }

    pub async fn complete(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
    ) -> Result<Session, SchedulingError> {
        let session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "scheduled" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot complete session with status '{}'",
                session.status
            )));
        }
        self.session_repo
            .update_status(therapist_id, session_id, "completed", None, None, None)
            .await
    }

    pub async fn no_show(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
    ) -> Result<Session, SchedulingError> {
        let session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "scheduled" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot mark no-show for session with status '{}'",
                session.status
            )));
        }
        self.session_repo
            .update_status(therapist_id, session_id, "no_show", None, None, None)
            .await
    }

    pub async fn reschedule(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
        new_starts_at: DateTime<Utc>,
        new_ends_at: DateTime<Utc>,
    ) -> Result<Session, SchedulingError> {
        if new_starts_at >= new_ends_at {
            return Err(SchedulingError::InvalidTimeRange);
        }
        let mut session = self.get_by_id(therapist_id, session_id).await?;
        if session.status != "scheduled" && session.status != "pending_approval" {
            return Err(SchedulingError::InvalidStatus(format!(
                "Cannot reschedule session with status '{}'",
                session.status
            )));
        }
        session.starts_at = new_starts_at;
        session.ends_at = new_ends_at;
        session.duration_mins = (new_ends_at - new_starts_at).num_minutes() as i32;
        self.session_repo.update(&session).await
    }

    pub async fn create_manual(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        amount_inr: i32,
        session_type_name: Option<String>,
        recurring_reservation_id: Option<Uuid>,
    ) -> Result<Session, SchedulingError> {
        if starts_at >= ends_at {
            return Err(SchedulingError::InvalidTimeRange);
        }
        let duration_mins = (ends_at - starts_at).num_minutes() as i32;
        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4(),
            therapist_id,
            client_id,
            starts_at,
            ends_at,
            duration_mins,
            status: "scheduled".to_string(),
            zoom_meeting_id: None,
            zoom_join_url: None,
            zoom_start_url: None,
            google_event_id: None,
            payment_status: "pending".to_string(),
            amount_inr,
            razorpay_payment_id: None,
            reminder_24h_sent: false,
            reminder_1h_sent: false,
            session_number: None,
            cancellation_reason: None,
            cancelled_at: None,
            cancelled_by: None,
            is_late_cancellation: None,
            session_type_name,
            recurring_reservation_id,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };
        self.session_repo.create(&session).await
    }

    pub async fn update_session(
        &self,
        therapist_id: Uuid,
        session: &Session,
    ) -> Result<Session, SchedulingError> {
        // Verify the session belongs to this therapist
        self.get_by_id(therapist_id, session.id).await?;
        self.session_repo.update(session).await
    }

    pub async fn soft_delete(
        &self,
        therapist_id: Uuid,
        session_id: Uuid,
    ) -> Result<(), SchedulingError> {
        // Verify it exists
        self.get_by_id(therapist_id, session_id).await?;
        self.session_repo.soft_delete(therapist_id, session_id).await
    }
}

// ─── Booking Service (with inline slot calculator) ──────────────────────────

/// Availability window for a single day-of-week.
/// Passed in from the caller (fetched from IAM availability table).
#[derive(Debug, Clone)]
pub struct DayAvailability {
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
}

pub struct BookingService {
    pub session_repo: Arc<dyn SessionRepository>,
    pub blocked_slot_repo: Arc<dyn BlockedSlotRepository>,
    pub recurring_repo: Arc<dyn RecurringReservationRepository>,
}

impl BookingService {
    pub fn new(
        session_repo: Arc<dyn SessionRepository>,
        blocked_slot_repo: Arc<dyn BlockedSlotRepository>,
        recurring_repo: Arc<dyn RecurringReservationRepository>,
    ) -> Self {
        Self {
            session_repo,
            blocked_slot_repo,
            recurring_repo,
        }
    }

    /// Compute available time slots for a given therapist on a given date.
    ///
    /// Algorithm:
    /// 1. Get therapist availability for the day-of-week
    /// 2. Generate slots based on session duration + buffer
    /// 3. Remove slots overlapping with booked sessions
    /// 4. Remove slots overlapping with blocked slots
    /// 5. Remove slots overlapping with recurring reservations
    /// 6. Remove slots before the advance booking cutoff
    /// 7. All times in IST (Asia/Kolkata)
    pub async fn get_available_slots(
        &self,
        therapist_id: Uuid,
        date: NaiveDate,
        duration_mins: i32,
        buffer_mins: i32,
        min_booking_advance_hours: i32,
        availability: &[DayAvailability],
    ) -> Result<Vec<TimeSlot>, SchedulingError> {
        // Determine day-of-week (chrono: Mon=0..Sun=6 in weekday().num_days_from_monday(),
        // but our DB uses 0=Sun..6=Sat to match JS convention)
        let weekday = date.weekday().num_days_from_sunday() as i16;

        // Step 1: Find availability windows for this day
        let day_windows: Vec<&DayAvailability> = availability
            .iter()
            .filter(|a| a.day_of_week == weekday && a.is_active)
            .collect();

        if day_windows.is_empty() {
            return Ok(vec![]);
        }

        // Compute day boundaries in UTC for DB queries
        let day_start_utc = Kolkata
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| SchedulingError::InvalidTimeRange)?;

        let day_end_utc = Kolkata
            .from_local_datetime(&date.and_hms_opt(23, 59, 59).unwrap())
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| SchedulingError::InvalidTimeRange)?;

        // Step 2: Generate candidate slots from each availability window
        let slot_step = duration_mins + buffer_mins;
        let mut candidates: Vec<TimeSlot> = Vec::new();

        for window in &day_windows {
            let mut current_time = window.start_time;
            loop {
                let slot_end = current_time + Duration::minutes(duration_mins as i64);
                if slot_end > window.end_time {
                    break;
                }

                let start_utc = Kolkata
                    .from_local_datetime(&date.and_time(current_time))
                    .earliest()
                    .map(|dt| dt.with_timezone(&Utc));

                let end_utc = Kolkata
                    .from_local_datetime(&date.and_time(slot_end))
                    .earliest()
                    .map(|dt| dt.with_timezone(&Utc));

                if let (Some(s), Some(e)) = (start_utc, end_utc) {
                    candidates.push(TimeSlot {
                        start: s,
                        end: e,
                        duration_mins,
                    });
                }

                current_time = current_time + Duration::minutes(slot_step as i64);
            }
        }

        if candidates.is_empty() {
            return Ok(vec![]);
        }

        // Step 3: Fetch booked sessions for the day
        let sessions = self
            .session_repo
            .list_by_date_range(therapist_id, day_start_utc, day_end_utc)
            .await?;

        let booked: Vec<(DateTime<Utc>, DateTime<Utc>)> = sessions
            .iter()
            .filter(|s| s.status == "scheduled" || s.status == "pending_approval")
            .map(|s| (s.starts_at, s.ends_at))
            .collect();

        // Step 4: Fetch blocked slots for the day
        let blocked = self
            .blocked_slot_repo
            .list_by_range(therapist_id, day_start_utc, day_end_utc)
            .await?;

        let blocked_ranges: Vec<(DateTime<Utc>, DateTime<Utc>)> =
            blocked.iter().map(|b| (b.start_at, b.end_at)).collect();

        // Step 5: Fetch recurring reservations for this day-of-week
        let recurring = self.recurring_repo.list_active(therapist_id).await?;

        let recurring_ranges: Vec<(DateTime<Utc>, DateTime<Utc>)> = recurring
            .iter()
            .filter(|r| r.day_of_week == weekday as i32)
            .filter_map(|r| {
                let s = Kolkata
                    .from_local_datetime(&date.and_time(r.start_time))
                    .earliest()
                    .map(|dt| dt.with_timezone(&Utc));
                let e = Kolkata
                    .from_local_datetime(&date.and_time(r.end_time))
                    .earliest()
                    .map(|dt| dt.with_timezone(&Utc));
                match (s, e) {
                    (Some(s), Some(e)) => Some((s, e)),
                    _ => None,
                }
            })
            .collect();

        // Step 6: Compute the advance booking cutoff
        let cutoff = Utc::now() + Duration::hours(min_booking_advance_hours as i64);

        // Filter candidates
        let available: Vec<TimeSlot> = candidates
            .into_iter()
            .filter(|slot| {
                // Must be after cutoff
                if slot.start < cutoff {
                    return false;
                }
                // Must not overlap with booked sessions
                if booked
                    .iter()
                    .any(|(bs, be)| slot.start < *be && slot.end > *bs)
                {
                    return false;
                }
                // Must not overlap with blocked slots
                if blocked_ranges
                    .iter()
                    .any(|(bs, be)| slot.start < *be && slot.end > *bs)
                {
                    return false;
                }
                // Must not overlap with recurring reservations
                if recurring_ranges
                    .iter()
                    .any(|(rs, re)| slot.start < *re && slot.end > *rs)
                {
                    return false;
                }
                true
            })
            .collect();

        Ok(available)
    }

    /// Book a single session (from booking page, status = pending_approval).
    pub async fn book(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        amount_inr: i32,
        session_type_name: Option<String>,
    ) -> Result<Session, SchedulingError> {
        if starts_at >= ends_at {
            return Err(SchedulingError::InvalidTimeRange);
        }

        // Check for time conflicts with existing sessions
        let existing = self
            .session_repo
            .list_by_date_range(therapist_id, starts_at, ends_at)
            .await?;

        let has_conflict = existing.iter().any(|s| {
            (s.status == "scheduled" || s.status == "pending_approval")
                && s.starts_at < ends_at
                && s.ends_at > starts_at
        });

        if has_conflict {
            return Err(SchedulingError::TimeConflict);
        }

        let duration_mins = (ends_at - starts_at).num_minutes() as i32;
        let now = Utc::now();
        let session = Session {
            id: Uuid::new_v4(),
            therapist_id,
            client_id,
            starts_at,
            ends_at,
            duration_mins,
            status: "pending_approval".to_string(),
            zoom_meeting_id: None,
            zoom_join_url: None,
            zoom_start_url: None,
            google_event_id: None,
            payment_status: "pending".to_string(),
            amount_inr,
            razorpay_payment_id: None,
            reminder_24h_sent: false,
            reminder_1h_sent: false,
            session_number: None,
            cancellation_reason: None,
            cancelled_at: None,
            cancelled_by: None,
            is_late_cancellation: None,
            session_type_name,
            recurring_reservation_id: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.session_repo.create(&session).await
    }

    /// Book multiple sessions at once (e.g. batch booking from recurring reservations).
    pub async fn book_multiple(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        slots: Vec<(DateTime<Utc>, DateTime<Utc>)>,
        amount_inr: i32,
        session_type_name: Option<String>,
        recurring_reservation_id: Option<Uuid>,
    ) -> Result<Vec<Session>, SchedulingError> {
        let mut sessions = Vec::with_capacity(slots.len());
        let now = Utc::now();

        for (starts_at, ends_at) in slots {
            if starts_at >= ends_at {
                return Err(SchedulingError::InvalidTimeRange);
            }
            let duration_mins = (ends_at - starts_at).num_minutes() as i32;
            let session = Session {
                id: Uuid::new_v4(),
                therapist_id,
                client_id,
                starts_at,
                ends_at,
                duration_mins,
                status: "scheduled".to_string(),
                zoom_meeting_id: None,
                zoom_join_url: None,
                zoom_start_url: None,
                google_event_id: None,
                payment_status: "pending".to_string(),
                amount_inr,
                razorpay_payment_id: None,
                reminder_24h_sent: false,
                reminder_1h_sent: false,
                session_number: None,
                cancellation_reason: None,
                cancelled_at: None,
                cancelled_by: None,
                is_late_cancellation: None,
                session_type_name: session_type_name.clone(),
                recurring_reservation_id,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };
            let created = self.session_repo.create(&session).await?;
            sessions.push(created);
        }

        Ok(sessions)
    }
}

// ─── Blocked Slot Service ───────────────────────────────────────────────────

pub struct BlockedSlotService {
    pub blocked_slot_repo: Arc<dyn BlockedSlotRepository>,
}

impl BlockedSlotService {
    pub fn new(blocked_slot_repo: Arc<dyn BlockedSlotRepository>) -> Self {
        Self { blocked_slot_repo }
    }

    pub async fn get_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<BlockedSlot, SchedulingError> {
        self.blocked_slot_repo
            .find_by_id(therapist_id, id)
            .await?
            .ok_or(SchedulingError::BlockedSlotNotFound)
    }

    pub async fn list_by_range(
        &self,
        therapist_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<BlockedSlot>, SchedulingError> {
        self.blocked_slot_repo
            .list_by_range(therapist_id, start, end)
            .await
    }

    pub async fn create(
        &self,
        therapist_id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError> {
        if start_at >= end_at {
            return Err(SchedulingError::InvalidTimeRange);
        }
        self.blocked_slot_repo
            .create(therapist_id, start_at, end_at, reason)
            .await
    }

    pub async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        start_at: DateTime<Utc>,
        end_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<BlockedSlot, SchedulingError> {
        if start_at >= end_at {
            return Err(SchedulingError::InvalidTimeRange);
        }
        // Verify it exists
        self.get_by_id(therapist_id, id).await?;
        self.blocked_slot_repo
            .update(therapist_id, id, start_at, end_at, reason)
            .await
    }

    pub async fn delete(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<(), SchedulingError> {
        self.get_by_id(therapist_id, id).await?;
        self.blocked_slot_repo.delete(therapist_id, id).await
    }
}

// ─── Recurring Reservation Service ──────────────────────────────────────────

pub struct RecurringReservationService {
    pub recurring_repo: Arc<dyn RecurringReservationRepository>,
    pub session_repo: Arc<dyn SessionRepository>,
}

impl RecurringReservationService {
    pub fn new(
        recurring_repo: Arc<dyn RecurringReservationRepository>,
        session_repo: Arc<dyn SessionRepository>,
    ) -> Self {
        Self {
            recurring_repo,
            session_repo,
        }
    }

    pub async fn get_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<RecurringReservation, SchedulingError> {
        self.recurring_repo
            .find_by_id(therapist_id, id)
            .await?
            .ok_or(SchedulingError::RecurringReservationNotFound)
    }

    pub async fn list_active(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError> {
        self.recurring_repo.list_active(therapist_id).await
    }

    pub async fn list_by_client(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<RecurringReservation>, SchedulingError> {
        self.recurring_repo
            .list_by_client(therapist_id, client_id)
            .await
    }

    pub async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError> {
        if start_time >= end_time {
            return Err(SchedulingError::InvalidTimeRange);
        }
        self.recurring_repo
            .create(
                therapist_id,
                client_id,
                day_of_week,
                start_time,
                end_time,
                session_type_name,
                amount_inr,
            )
            .await
    }

    pub async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        day_of_week: i32,
        start_time: NaiveTime,
        end_time: NaiveTime,
        session_type_name: Option<&str>,
        amount_inr: i32,
    ) -> Result<RecurringReservation, SchedulingError> {
        if start_time >= end_time {
            return Err(SchedulingError::InvalidTimeRange);
        }
        self.get_by_id(therapist_id, id).await?;
        self.recurring_repo
            .update(
                therapist_id,
                id,
                day_of_week,
                start_time,
                end_time,
                session_type_name,
                amount_inr,
            )
            .await
    }

    pub async fn deactivate(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<RecurringReservation, SchedulingError> {
        self.get_by_id(therapist_id, id).await?;
        self.recurring_repo.deactivate(therapist_id, id).await
    }

    /// Create a session from a recurring reservation for a specific date.
    pub async fn create_session_from_reservation(
        &self,
        therapist_id: Uuid,
        reservation_id: Uuid,
        date: NaiveDate,
    ) -> Result<Session, SchedulingError> {
        let reservation = self.get_by_id(therapist_id, reservation_id).await?;

        if !reservation.is_active {
            return Err(SchedulingError::InvalidStatus(
                "Recurring reservation is inactive".to_string(),
            ));
        }

        let starts_at = Kolkata
            .from_local_datetime(&date.and_time(reservation.start_time))
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or(SchedulingError::InvalidTimeRange)?;

        let ends_at = Kolkata
            .from_local_datetime(&date.and_time(reservation.end_time))
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or(SchedulingError::InvalidTimeRange)?;

        let duration_mins = (ends_at - starts_at).num_minutes() as i32;
        let now = Utc::now();

        let session = Session {
            id: Uuid::new_v4(),
            therapist_id,
            client_id: reservation.client_id,
            starts_at,
            ends_at,
            duration_mins,
            status: "scheduled".to_string(),
            zoom_meeting_id: None,
            zoom_join_url: None,
            zoom_start_url: None,
            google_event_id: None,
            payment_status: "pending".to_string(),
            amount_inr: reservation.amount_inr,
            razorpay_payment_id: None,
            reminder_24h_sent: false,
            reminder_1h_sent: false,
            session_number: None,
            cancellation_reason: None,
            cancelled_at: None,
            cancelled_by: None,
            is_late_cancellation: None,
            session_type_name: reservation.session_type_name.clone(),
            recurring_reservation_id: Some(reservation.id),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        self.session_repo.create(&session).await
    }
}

// ─── Session Type Service ───────────────────────────────────────────────────

pub struct SessionTypeService {
    pub session_type_repo: Arc<dyn SessionTypeRepository>,
}

impl SessionTypeService {
    pub fn new(session_type_repo: Arc<dyn SessionTypeRepository>) -> Self {
        Self { session_type_repo }
    }

    pub async fn get_by_id(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<SessionType, SchedulingError> {
        self.session_type_repo
            .find_by_id(therapist_id, id)
            .await?
            .ok_or(SchedulingError::SessionTypeNotFound)
    }

    pub async fn list_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError> {
        self.session_type_repo.list_by_therapist(therapist_id).await
    }

    pub async fn list_active_by_therapist(
        &self,
        therapist_id: Uuid,
    ) -> Result<Vec<SessionType>, SchedulingError> {
        self.session_type_repo
            .list_active_by_therapist(therapist_id)
            .await
    }

    pub async fn create(
        &self,
        therapist_id: Uuid,
        name: &str,
        duration_mins: i32,
        rate_inr: i32,
        description: Option<&str>,
        is_active: bool,
        sort_order: i32,
        intake_form_id: Option<Uuid>,
    ) -> Result<SessionType, SchedulingError> {
        self.session_type_repo
            .create(
                therapist_id,
                name,
                duration_mins,
                rate_inr,
                description,
                is_active,
                sort_order,
                intake_form_id,
            )
            .await
    }

    pub async fn update(
        &self,
        therapist_id: Uuid,
        id: Uuid,
        name: &str,
        duration_mins: i32,
        rate_inr: i32,
        description: Option<&str>,
        is_active: bool,
        sort_order: i32,
        intake_form_id: Option<Uuid>,
    ) -> Result<SessionType, SchedulingError> {
        self.get_by_id(therapist_id, id).await?;
        self.session_type_repo
            .update(
                therapist_id,
                id,
                name,
                duration_mins,
                rate_inr,
                description,
                is_active,
                sort_order,
                intake_form_id,
            )
            .await
    }

    pub async fn delete(
        &self,
        therapist_id: Uuid,
        id: Uuid,
    ) -> Result<(), SchedulingError> {
        self.get_by_id(therapist_id, id).await?;
        self.session_type_repo.delete(therapist_id, id).await
    }

    pub async fn reorder(
        &self,
        therapist_id: Uuid,
        ordered_ids: &[Uuid],
    ) -> Result<(), SchedulingError> {
        self.session_type_repo
            .reorder(therapist_id, ordered_ids)
            .await
    }

    pub async fn get_rates(
        &self,
        therapist_id: Uuid,
        session_type_id: Uuid,
    ) -> Result<Vec<SessionTypeRate>, SchedulingError> {
        // Verify the session type belongs to this therapist
        self.get_by_id(therapist_id, session_type_id).await?;
        self.session_type_repo.find_rates(session_type_id).await
    }
}
