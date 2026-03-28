use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::billing::domain::entity::Invoice;
use crate::billing::domain::error::BillingError;
use crate::billing::domain::port::InvoiceRepository;

pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl PgInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InvoiceRepository for PgInvoiceRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Invoice>, BillingError> {
        sqlx::query_as::<_, Invoice>(
            "SELECT
                id, therapist_id, client_id, session_id,
                invoice_number,
                amount_inr, gst_amount_inr, total_inr,
                status::text as status,
                razorpay_payment_id, razorpay_order_id,
                paid_at, created_at
            FROM invoices
            WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))
    }

    async fn list(
        &self,
        therapist_ids: &[Uuid],
        status: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<Invoice>, i64), BillingError> {
        let rows = sqlx::query_as::<_, Invoice>(
            "SELECT
                id, therapist_id, client_id, session_id,
                invoice_number,
                amount_inr, gst_amount_inr, total_inr,
                status::text as status,
                razorpay_payment_id, razorpay_order_id,
                paid_at, created_at
            FROM invoices
            WHERE therapist_id = ANY($1)
              AND ($2::text IS NULL OR status::text = $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4"
        )
        .bind(therapist_ids)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))?;

        let total = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM invoices
            WHERE therapist_id = ANY($1)
              AND ($2::text IS NULL OR status::text = $2)"
        )
        .bind(therapist_ids)
        .bind(status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))?;

        Ok((rows, total))
    }

    async fn create(
        &self,
        therapist_id: Uuid,
        client_id: Uuid,
        session_id: Option<Uuid>,
        invoice_number: &str,
        amount_inr: i32,
        gst_amount_inr: i32,
        total_inr: i32,
    ) -> Result<Invoice, BillingError> {
        sqlx::query_as::<_, Invoice>(
            "INSERT INTO invoices (
                therapist_id, client_id, session_id,
                invoice_number, amount_inr, gst_amount_inr, total_inr
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING
                id, therapist_id, client_id, session_id,
                invoice_number,
                amount_inr, gst_amount_inr, total_inr,
                status::text as status,
                razorpay_payment_id, razorpay_order_id,
                paid_at, created_at"
        )
        .bind(therapist_id)
        .bind(client_id)
        .bind(session_id)
        .bind(invoice_number)
        .bind(amount_inr)
        .bind(gst_amount_inr)
        .bind(total_inr)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))
    }

    async fn mark_paid(
        &self,
        id: Uuid,
        razorpay_payment_id: &str,
        razorpay_order_id: Option<&str>,
    ) -> Result<Invoice, BillingError> {
        sqlx::query_as::<_, Invoice>(
            "UPDATE invoices SET
                status = 'paid'::invoice_status,
                razorpay_payment_id = $2,
                razorpay_order_id = COALESCE($3, razorpay_order_id),
                paid_at = now()
            WHERE id = $1
            RETURNING
                id, therapist_id, client_id, session_id,
                invoice_number,
                amount_inr, gst_amount_inr, total_inr,
                status::text as status,
                razorpay_payment_id, razorpay_order_id,
                paid_at, created_at"
        )
        .bind(id)
        .bind(razorpay_payment_id)
        .bind(razorpay_order_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))
    }

    async fn next_sequence_number(
        &self,
        therapist_id: Uuid,
        year_month: &str,
    ) -> Result<i32, BillingError> {
        let prefix = format!("MANO-{}-", year_month);

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
            FROM invoices
            WHERE therapist_id = $1 AND invoice_number LIKE $2"
        )
        .bind(therapist_id)
        .bind(format!("{}%", prefix))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| BillingError::Database(e.to_string()))?;

        Ok(count as i32 + 1)
    }
}
