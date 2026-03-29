use actix_web::{web, HttpResponse};
use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Asia::Kolkata;
use serde::Deserialize;
use uuid::Uuid;

use crate::clients::application::service::ClientService;
use crate::clients::domain::entity::CreateClientInput;
use crate::iam::application::service::TherapistService;
use crate::scheduling::application::service::{
    BlockedSlotService, BookingService, DayAvailability, RecurringReservationService,
    SessionService, SessionTypeService,
};
use crate::shared::error::AppError;
use crate::shared::types::AuthUser;

// Parse date string: accepts "2026-03-22" or "2026-03-22T18:30:00.000Z"
fn parse_date_to_utc_start(s: &str) -> Result<DateTime<Utc>, AppError> {
    // Try ISO datetime first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    // Try plain date
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Kolkata
            .from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap())
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| AppError::bad_request("Invalid date"));
    }
    Err(AppError::bad_request("Invalid date format"))
}

fn parse_date_to_utc_end(s: &str) -> Result<DateTime<Utc>, AppError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Ok(dt);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Kolkata
            .from_local_datetime(&d.and_hms_opt(23, 59, 59).unwrap())
            .earliest()
            .map(|dt| dt.with_timezone(&Utc))
            .ok_or_else(|| AppError::bad_request("Invalid date"));
    }
    Err(AppError::bad_request("Invalid date format"))
}

// ─── Request DTOs ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Deserialize)]
pub struct UpcomingQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CancelSessionInput {
    pub reason: Option<String>,
    pub cancelled_by: Option<String>,
    pub cancellation_hours: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RejectSessionInput {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RescheduleSessionInput {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionInput {
    pub client_id: Uuid,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub amount_inr: i32,
    pub session_type_name: Option<String>,
    pub recurring_reservation_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionInput {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub amount_inr: i32,
    pub status: String,
    pub payment_status: String,
    pub session_type_name: Option<String>,
    pub recurring_reservation_id: Option<Uuid>,
    pub zoom_meeting_id: Option<String>,
    pub zoom_join_url: Option<String>,
    pub zoom_start_url: Option<String>,
    pub google_event_id: Option<String>,
    pub razorpay_payment_id: Option<String>,
    pub session_number: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AvailableSlotsQuery {
    pub date: NaiveDate,
    pub duration_mins: i32,
    pub buffer_mins: Option<i32>,
    pub min_booking_advance_hours: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct AvailabilityInput {
    pub availability: Vec<DayAvailabilityInput>,
}

#[derive(Debug, Deserialize)]
pub struct DayAvailabilityInput {
    pub day_of_week: i16,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub is_active: bool,
}

#[derive(Debug, Deserialize)]
pub struct BookSessionInput {
    pub client_name: String,
    pub client_email: String,
    pub client_phone: Option<String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub amount_inr: Option<i32>,
    pub session_type_name: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookMultipleInput {
    pub client_name: String,
    pub client_email: String,
    pub client_phone: Option<String>,
    pub slots: Vec<SlotInput>,
    pub amount_inr: Option<i32>,
    pub session_type_name: Option<String>,
    pub recurring_reservation_id: Option<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SlotInput {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct BlockedSlotInput {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecurringReservationInput {
    pub client_id: Uuid,
    pub day_of_week: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub session_type_name: Option<String>,
    pub amount_inr: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecurringReservationInput {
    pub day_of_week: i32,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub session_type_name: Option<String>,
    pub amount_inr: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionFromReservationInput {
    pub date: NaiveDate,
}

#[derive(Debug, Deserialize)]
pub struct SessionTypeInput {
    pub name: String,
    pub duration_mins: i32,
    pub rate_inr: i32,
    pub description: Option<String>,
    pub is_active: bool,
    pub sort_order: i32,
    pub intake_form_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderInput {
    pub ordered_ids: Vec<Uuid>,
}

// ─── Session Handlers ───────────────────────────────────────────────────────

/// GET /api/v1/sessions/{id}
pub async fn get_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.get_by_id(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(session))
}

/// GET /api/v1/sessions?start=YYYY-MM-DD&end=YYYY-MM-DD
pub async fn list_sessions(
    user: AuthUser,
    query: web::Query<DateRangeQuery>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let start_utc = parse_date_to_utc_start(&query.start)?;
    let end_utc = parse_date_to_utc_end(&query.end)?;

    let sessions = session_svc
        .list_by_date_range(user.id, start_utc, end_utc)
        .await?;
    Ok(HttpResponse::Ok().json(sessions))
}

/// GET /api/v1/sessions/pending
pub async fn list_pending_sessions(
    user: AuthUser,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let sessions = session_svc.list_pending(user.id).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

/// GET /api/v1/sessions/today
pub async fn list_today_sessions(
    user: AuthUser,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let now_ist = Utc::now().with_timezone(&Kolkata);
    let today = now_ist.date_naive();

    let day_start = Kolkata
        .from_local_datetime(&today.and_hms_opt(0, 0, 0).unwrap())
        .earliest()
        .map(|dt| dt.with_timezone(&Utc))
        .ok_or_else(|| AppError::internal("Failed to compute day boundaries"))?;

    let day_end = Kolkata
        .from_local_datetime(&today.and_hms_opt(23, 59, 59).unwrap())
        .earliest()
        .map(|dt| dt.with_timezone(&Utc))
        .ok_or_else(|| AppError::internal("Failed to compute day boundaries"))?;

    let sessions = session_svc
        .list_today(user.id, day_start, day_end)
        .await?;
    Ok(HttpResponse::Ok().json(sessions))
}

/// GET /api/v1/sessions/upcoming?limit=10
pub async fn list_upcoming_sessions(
    user: AuthUser,
    query: web::Query<UpcomingQuery>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let limit = query.limit.unwrap_or(10).min(100);
    let sessions = session_svc.list_upcoming(user.id, limit).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

/// GET /api/v1/sessions/by-client/{client_id}
pub async fn list_client_sessions(
    user: AuthUser,
    client_id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let sessions = session_svc.list_by_client(user.id, *client_id).await?;
    Ok(HttpResponse::Ok().json(sessions))
}

/// POST /api/v1/sessions
pub async fn create_session(
    user: AuthUser,
    input: web::Json<CreateSessionInput>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc
        .create_manual(
            user.id,
            input.client_id,
            input.starts_at,
            input.ends_at,
            input.amount_inr,
            input.session_type_name.clone(),
            input.recurring_reservation_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(session))
}

/// PUT /api/v1/sessions/{id}
pub async fn update_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<UpdateSessionInput>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let mut session = session_svc.get_by_id(user.id, *id).await?;
    session.starts_at = input.starts_at;
    session.ends_at = input.ends_at;
    session.duration_mins = (input.ends_at - input.starts_at).num_minutes() as i32;
    session.amount_inr = input.amount_inr;
    session.status = input.status.clone();
    session.payment_status = input.payment_status.clone();
    session.session_type_name = input.session_type_name.clone();
    session.recurring_reservation_id = input.recurring_reservation_id;
    session.zoom_meeting_id = input.zoom_meeting_id.clone();
    session.zoom_join_url = input.zoom_join_url.clone();
    session.zoom_start_url = input.zoom_start_url.clone();
    session.google_event_id = input.google_event_id.clone();
    session.razorpay_payment_id = input.razorpay_payment_id.clone();
    session.session_number = input.session_number;
    let updated = session_svc.update_session(user.id, &session).await?;
    Ok(HttpResponse::Ok().json(updated))
}

/// POST /api/v1/sessions/{id}/approve
pub async fn approve_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.approve(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(session))
}

/// POST /api/v1/sessions/{id}/reject
pub async fn reject_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<RejectSessionInput>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc
        .reject(user.id, *id, input.reason.as_deref())
        .await?;
    Ok(HttpResponse::Ok().json(session))
}

/// POST /api/v1/sessions/{id}/cancel
pub async fn cancel_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<CancelSessionInput>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let cancelled_by = input.cancelled_by.as_deref().unwrap_or("therapist");
    let cancellation_hours = input.cancellation_hours.unwrap_or(24);
    let session = session_svc
        .cancel(
            user.id,
            *id,
            input.reason.as_deref(),
            cancelled_by,
            cancellation_hours,
        )
        .await?;
    Ok(HttpResponse::Ok().json(session))
}

/// POST /api/v1/sessions/{id}/complete
pub async fn complete_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.complete(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(session))
}

/// POST /api/v1/sessions/{id}/no-show
pub async fn no_show_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc.no_show(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(session))
}

/// POST /api/v1/sessions/{id}/reschedule
pub async fn reschedule_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<RescheduleSessionInput>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    let session = session_svc
        .reschedule(user.id, *id, input.starts_at, input.ends_at)
        .await?;
    Ok(HttpResponse::Ok().json(session))
}

/// DELETE /api/v1/sessions/{id}
pub async fn delete_session(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_svc: web::Data<SessionService>,
) -> Result<HttpResponse, AppError> {
    session_svc.soft_delete(user.id, *id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ─── Booking Handlers ───────────────────────────────────────────────────────

/// POST /api/v1/booking/{slug}/available-slots?date=YYYY-MM-DD&duration_mins=50
pub async fn get_available_slots(
    slug: web::Path<String>,
    query: web::Query<AvailableSlotsQuery>,
    input: web::Json<AvailabilityInput>,
    booking_svc: web::Data<BookingService>,
    therapist_svc: web::Data<TherapistService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_by_slug(&slug).await?;

    let availability: Vec<DayAvailability> = input
        .availability
        .iter()
        .map(|a| DayAvailability {
            day_of_week: a.day_of_week,
            start_time: a.start_time,
            end_time: a.end_time,
            is_active: a.is_active,
        })
        .collect();

    let slots = booking_svc
        .get_available_slots(
            therapist.id,
            query.date,
            query.duration_mins,
            query.buffer_mins.unwrap_or(10),
            query.min_booking_advance_hours.unwrap_or(24),
            &availability,
        )
        .await?;
    Ok(HttpResponse::Ok().json(slots))
}

/// POST /api/v1/booking/{slug}/book
pub async fn book_session(
    slug: web::Path<String>,
    input: web::Json<BookSessionInput>,
    booking_svc: web::Data<BookingService>,
    therapist_svc: web::Data<TherapistService>,
    client_svc: web::Data<ClientService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_by_slug(&slug).await?;

    // Look up existing client by email, or create a new one
    let client = client_svc
        .client_repo
        .find_by_email(therapist.id, &input.client_email)
        .await
        .map_err(AppError::from)?;

    let client_id = match client {
        Some(c) => c.id,
        None => {
            let new_client = client_svc
                .create_client(
                    therapist.id,
                    &CreateClientInput {
                        full_name: input.client_name.clone(),
                        email: Some(input.client_email.clone()),
                        phone: input.client_phone.clone(),
                        date_of_birth: None,
                        emergency_contact: None,
                        notes_private: input.reason.clone(),
                        client_type: None,
                        category: None,
                    },
                    None, // no plan limit check for public bookings
                )
                .await
                .map_err(AppError::from)?;
            new_client.id
        }
    };

    let amount = input.amount_inr.unwrap_or(0);
    let session = booking_svc
        .book(
            therapist.id,
            client_id,
            input.starts_at,
            input.ends_at,
            amount,
            input.session_type_name.clone(),
        )
        .await?;
    Ok(HttpResponse::Created().json(session))
}

/// POST /api/v1/booking/{slug}/book-multiple
pub async fn book_multiple_sessions(
    slug: web::Path<String>,
    input: web::Json<BookMultipleInput>,
    booking_svc: web::Data<BookingService>,
    therapist_svc: web::Data<TherapistService>,
    client_svc: web::Data<ClientService>,
) -> Result<HttpResponse, AppError> {
    let therapist = therapist_svc.get_by_slug(&slug).await?;

    // Look up existing client by email, or create a new one
    let client = client_svc
        .client_repo
        .find_by_email(therapist.id, &input.client_email)
        .await
        .map_err(AppError::from)?;

    let client_id = match client {
        Some(c) => c.id,
        None => {
            let new_client = client_svc
                .create_client(
                    therapist.id,
                    &CreateClientInput {
                        full_name: input.client_name.clone(),
                        email: Some(input.client_email.clone()),
                        phone: input.client_phone.clone(),
                        date_of_birth: None,
                        emergency_contact: None,
                        notes_private: input.reason.clone(),
                        client_type: None,
                        category: None,
                    },
                    None,
                )
                .await
                .map_err(AppError::from)?;
            new_client.id
        }
    };

    let slots: Vec<(DateTime<Utc>, DateTime<Utc>)> = input
        .slots
        .iter()
        .map(|s| (s.starts_at, s.ends_at))
        .collect();

    let amount = input.amount_inr.unwrap_or(0);
    let sessions = booking_svc
        .book_multiple(
            therapist.id,
            client_id,
            slots,
            amount,
            input.session_type_name.clone(),
            input.recurring_reservation_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(sessions))
}

// ─── Blocked Slot Handlers ──────────────────────────────────────────────────

/// GET /api/v1/blocked-slots?start=YYYY-MM-DD&end=YYYY-MM-DD
pub async fn list_blocked_slots(
    user: AuthUser,
    query: web::Query<DateRangeQuery>,
    blocked_svc: web::Data<BlockedSlotService>,
) -> Result<HttpResponse, AppError> {
    let start_utc = parse_date_to_utc_start(&query.start)?;
    let end_utc = parse_date_to_utc_end(&query.end)?;

    let slots = blocked_svc
        .list_by_range(user.id, start_utc, end_utc)
        .await?;
    Ok(HttpResponse::Ok().json(slots))
}

/// GET /api/v1/blocked-slots/{id}
pub async fn get_blocked_slot(
    user: AuthUser,
    id: web::Path<Uuid>,
    blocked_svc: web::Data<BlockedSlotService>,
) -> Result<HttpResponse, AppError> {
    let slot = blocked_svc.get_by_id(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(slot))
}

/// POST /api/v1/blocked-slots
pub async fn create_blocked_slot(
    user: AuthUser,
    input: web::Json<BlockedSlotInput>,
    blocked_svc: web::Data<BlockedSlotService>,
) -> Result<HttpResponse, AppError> {
    let slot = blocked_svc
        .create(user.id, input.start_at, input.end_at, input.reason.as_deref())
        .await?;
    Ok(HttpResponse::Created().json(slot))
}

/// PUT /api/v1/blocked-slots/{id}
pub async fn update_blocked_slot(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<BlockedSlotInput>,
    blocked_svc: web::Data<BlockedSlotService>,
) -> Result<HttpResponse, AppError> {
    let slot = blocked_svc
        .update(user.id, *id, input.start_at, input.end_at, input.reason.as_deref())
        .await?;
    Ok(HttpResponse::Ok().json(slot))
}

/// DELETE /api/v1/blocked-slots/{id}
pub async fn delete_blocked_slot(
    user: AuthUser,
    id: web::Path<Uuid>,
    blocked_svc: web::Data<BlockedSlotService>,
) -> Result<HttpResponse, AppError> {
    blocked_svc.delete(user.id, *id).await?;
    Ok(HttpResponse::NoContent().finish())
}

// ─── Recurring Reservation Handlers ─────────────────────────────────────────

/// GET /api/v1/recurring-reservations
pub async fn list_recurring_reservations(
    user: AuthUser,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservations = recurring_svc.list_active(user.id).await?;
    Ok(HttpResponse::Ok().json(reservations))
}

/// GET /api/v1/recurring-reservations/{id}
pub async fn get_recurring_reservation(
    user: AuthUser,
    id: web::Path<Uuid>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservation = recurring_svc.get_by_id(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(reservation))
}

/// GET /api/v1/recurring-reservations/by-client/{client_id}
pub async fn list_client_recurring_reservations(
    user: AuthUser,
    client_id: web::Path<Uuid>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservations = recurring_svc.list_by_client(user.id, *client_id).await?;
    Ok(HttpResponse::Ok().json(reservations))
}

/// POST /api/v1/recurring-reservations
pub async fn create_recurring_reservation(
    user: AuthUser,
    input: web::Json<RecurringReservationInput>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservation = recurring_svc
        .create(
            user.id,
            input.client_id,
            input.day_of_week,
            input.start_time,
            input.end_time,
            input.session_type_name.as_deref(),
            input.amount_inr,
        )
        .await?;
    Ok(HttpResponse::Created().json(reservation))
}

/// PUT /api/v1/recurring-reservations/{id}
pub async fn update_recurring_reservation(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<UpdateRecurringReservationInput>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservation = recurring_svc
        .update(
            user.id,
            *id,
            input.day_of_week,
            input.start_time,
            input.end_time,
            input.session_type_name.as_deref(),
            input.amount_inr,
        )
        .await?;
    Ok(HttpResponse::Ok().json(reservation))
}

/// POST /api/v1/recurring-reservations/{id}/deactivate
pub async fn deactivate_recurring_reservation(
    user: AuthUser,
    id: web::Path<Uuid>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let reservation = recurring_svc.deactivate(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(reservation))
}

/// POST /api/v1/recurring-reservations/{id}/create-session
pub async fn create_session_from_reservation(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<CreateSessionFromReservationInput>,
    recurring_svc: web::Data<RecurringReservationService>,
) -> Result<HttpResponse, AppError> {
    let session = recurring_svc
        .create_session_from_reservation(user.id, *id, input.date)
        .await?;
    Ok(HttpResponse::Created().json(session))
}

// ─── Session Type Handlers ──────────────────────────────────────────────────

/// GET /api/v1/session-types
pub async fn list_session_types(
    user: AuthUser,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let types = session_type_svc.list_by_therapist(user.id).await?;
    Ok(HttpResponse::Ok().json(types))
}

/// GET /api/v1/session-types/active
pub async fn list_active_session_types(
    user: AuthUser,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let types = session_type_svc.list_active_by_therapist(user.id).await?;
    Ok(HttpResponse::Ok().json(types))
}

/// GET /api/v1/session-types/by-therapist/{therapist_id}
pub async fn list_session_types_by_therapist(
    therapist_id: web::Path<Uuid>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let types = session_type_svc
        .list_active_by_therapist(*therapist_id)
        .await?;
    Ok(HttpResponse::Ok().json(types))
}

/// GET /api/v1/session-types/{id}
pub async fn get_session_type(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let st = session_type_svc.get_by_id(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(st))
}

/// POST /api/v1/session-types
pub async fn create_session_type(
    user: AuthUser,
    input: web::Json<SessionTypeInput>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let st = session_type_svc
        .create(
            user.id,
            &input.name,
            input.duration_mins,
            input.rate_inr,
            input.description.as_deref(),
            input.is_active,
            input.sort_order,
            input.intake_form_id,
        )
        .await?;
    Ok(HttpResponse::Created().json(st))
}

/// PUT /api/v1/session-types/{id}
pub async fn update_session_type(
    user: AuthUser,
    id: web::Path<Uuid>,
    input: web::Json<SessionTypeInput>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let st = session_type_svc
        .update(
            user.id,
            *id,
            &input.name,
            input.duration_mins,
            input.rate_inr,
            input.description.as_deref(),
            input.is_active,
            input.sort_order,
            input.intake_form_id,
        )
        .await?;
    Ok(HttpResponse::Ok().json(st))
}

/// DELETE /api/v1/session-types/{id}
pub async fn delete_session_type(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    session_type_svc.delete(user.id, *id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/v1/session-types/reorder
pub async fn reorder_session_types(
    user: AuthUser,
    input: web::Json<ReorderInput>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    session_type_svc
        .reorder(user.id, &input.ordered_ids)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/v1/session-types/{id}/rates
pub async fn get_session_type_rates(
    user: AuthUser,
    id: web::Path<Uuid>,
    session_type_svc: web::Data<SessionTypeService>,
) -> Result<HttpResponse, AppError> {
    let rates = session_type_svc.get_rates(user.id, *id).await?;
    Ok(HttpResponse::Ok().json(rates))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
            // Sessions — queries
            .route("/api/v1/sessions", web::get().to(list_sessions))
            .route("/api/v1/sessions/pending", web::get().to(list_pending_sessions))
            .route("/api/v1/sessions/today", web::get().to(list_today_sessions))
            .route("/api/v1/sessions/upcoming", web::get().to(list_upcoming_sessions))
            .route(
                "/api/v1/sessions/by-client/{client_id}",
                web::get().to(list_client_sessions),
            )
            .route("/api/v1/sessions/{id}", web::get().to(get_session))
            // Sessions — mutations
            .route("/api/v1/sessions", web::post().to(create_session))
            .route("/api/v1/sessions/{id}", web::put().to(update_session))
            .route("/api/v1/sessions/{id}", web::delete().to(delete_session))
            .route("/api/v1/sessions/{id}/approve", web::post().to(approve_session))
            .route("/api/v1/sessions/{id}/reject", web::post().to(reject_session))
            .route("/api/v1/sessions/{id}/cancel", web::post().to(cancel_session))
            .route("/api/v1/sessions/{id}/complete", web::post().to(complete_session))
            .route("/api/v1/sessions/{id}/no-show", web::post().to(no_show_session))
            .route(
                "/api/v1/sessions/{id}/reschedule",
                web::post().to(reschedule_session),
            )
            // Booking (public, slug in path)
            .route(
                "/api/v1/booking/{slug}/available-slots",
                web::post().to(get_available_slots),
            )
            .route(
                "/api/v1/booking/{slug}/book",
                web::post().to(book_session),
            )
            .route(
                "/api/v1/booking/{slug}/book-multiple",
                web::post().to(book_multiple_sessions),
            )
            // Blocked slots
            .route("/api/v1/blocked-slots", web::get().to(list_blocked_slots))
            .route("/api/v1/blocked-slots/{id}", web::get().to(get_blocked_slot))
            .route("/api/v1/blocked-slots", web::post().to(create_blocked_slot))
            .route("/api/v1/blocked-slots/{id}", web::put().to(update_blocked_slot))
            .route(
                "/api/v1/blocked-slots/{id}",
                web::delete().to(delete_blocked_slot),
            )
            // Recurring reservations
            .route(
                "/api/v1/recurring-reservations",
                web::get().to(list_recurring_reservations),
            )
            .route(
                "/api/v1/recurring-reservations/{id}",
                web::get().to(get_recurring_reservation),
            )
            .route(
                "/api/v1/recurring-reservations/by-client/{client_id}",
                web::get().to(list_client_recurring_reservations),
            )
            .route(
                "/api/v1/recurring-reservations",
                web::post().to(create_recurring_reservation),
            )
            .route(
                "/api/v1/recurring-reservations/{id}",
                web::put().to(update_recurring_reservation),
            )
            .route(
                "/api/v1/recurring-reservations/{id}/deactivate",
                web::post().to(deactivate_recurring_reservation),
            )
            .route(
                "/api/v1/recurring-reservations/{id}/create-session",
                web::post().to(create_session_from_reservation),
            )
            // Session types
            .route("/api/v1/session-types", web::get().to(list_session_types))
            .route(
                "/api/v1/session-types/active",
                web::get().to(list_active_session_types),
            )
            .route(
                "/api/v1/session-types/by-therapist/{therapist_id}",
                web::get().to(list_session_types_by_therapist),
            )
            .route("/api/v1/session-types/{id}", web::get().to(get_session_type))
            .route("/api/v1/session-types", web::post().to(create_session_type))
            .route("/api/v1/session-types/{id}", web::put().to(update_session_type))
            .route(
                "/api/v1/session-types/{id}",
                web::delete().to(delete_session_type),
            )
            .route(
                "/api/v1/session-types/reorder",
                web::post().to(reorder_session_types),
            )
            .route(
                "/api/v1/session-types/{id}/rates",
                web::get().to(get_session_type_rates),
            );
}
