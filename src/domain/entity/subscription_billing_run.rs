use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

use super::BillingRunStatus;
use super::AuditMetadata;

/// Strongly-typed ID for SubscriptionBillingRun
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionBillingRunId(pub Uuid);

impl SubscriptionBillingRunId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for SubscriptionBillingRunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SubscriptionBillingRunId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for SubscriptionBillingRunId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<SubscriptionBillingRunId> for Uuid {
    fn from(id: SubscriptionBillingRunId) -> Self { id.0 }
}

impl AsRef<Uuid> for SubscriptionBillingRunId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for SubscriptionBillingRunId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionBillingRun {
    pub id: Uuid,
    pub subscription_id: Uuid,
    pub company_id: Uuid,
    pub period_start: NaiveDate,
    pub period_end: NaiveDate,
    pub due_date: NaiveDate,
    pub grand_total: Decimal,
    pub status: BillingRunStatus,
    pub invoice_id: Option<Uuid>,
    pub idempotency_key: String,
    pub attempted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl SubscriptionBillingRun {
    /// Create a builder for SubscriptionBillingRun
    pub fn builder() -> SubscriptionBillingRunBuilder {
        SubscriptionBillingRunBuilder::default()
    }

    /// Create a new SubscriptionBillingRun with required fields
    pub fn new(subscription_id: Uuid, company_id: Uuid, period_start: NaiveDate, period_end: NaiveDate, due_date: NaiveDate, grand_total: Decimal, status: BillingRunStatus, idempotency_key: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            subscription_id,
            company_id,
            period_start,
            period_end,
            due_date,
            grand_total,
            status,
            invoice_id: None,
            idempotency_key,
            attempted_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> SubscriptionBillingRunId {
        SubscriptionBillingRunId(self.id)
    }

    /// Get when this entity was created
    pub fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.created_at.as_ref()
    }

    /// Get when this entity was last updated
    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.updated_at.as_ref()
    }

    /// Check if this entity is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.metadata.deleted_at.is_some()
    }

    /// Check if this entity is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.metadata.deleted_at.is_none()
    }

    /// Get when this entity was deleted
    pub fn deleted_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.deleted_at.as_ref()
    }

    /// Get who created this entity
    pub fn created_by(&self) -> Option<&Uuid> {
        self.metadata.created_by.as_ref()
    }

    /// Get who last updated this entity
    pub fn updated_by(&self) -> Option<&Uuid> {
        self.metadata.updated_by.as_ref()
    }

    /// Get who deleted this entity
    pub fn deleted_by(&self) -> Option<&Uuid> {
        self.metadata.deleted_by.as_ref()
    }

    /// Get the current status
    pub fn status(&self) -> &BillingRunStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the invoice_id field (chainable)
    pub fn with_invoice_id(mut self, value: Uuid) -> Self {
        self.invoice_id = Some(value);
        self
    }

    /// Set the attempted_at field (chainable)
    pub fn with_attempted_at(mut self, value: DateTime<Utc>) -> Self {
        self.attempted_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "subscription_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.subscription_id = v; }
                }
                "company_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.company_id = v; }
                }
                "period_start" => {
                    if let Ok(v) = serde_json::from_value(value) { self.period_start = v; }
                }
                "period_end" => {
                    if let Ok(v) = serde_json::from_value(value) { self.period_end = v; }
                }
                "due_date" => {
                    if let Ok(v) = serde_json::from_value(value) { self.due_date = v; }
                }
                "grand_total" => {
                    if let Ok(v) = serde_json::from_value(value) { self.grand_total = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "invoice_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.invoice_id = v; }
                }
                "idempotency_key" => {
                    if let Ok(v) = serde_json::from_value(value) { self.idempotency_key = v; }
                }
                "attempted_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.attempted_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for SubscriptionBillingRun {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "SubscriptionBillingRun"
    }
}

impl backbone_core::PersistentEntity for SubscriptionBillingRun {
    fn entity_id(&self) -> String {
        self.id.to_string()
    }
    fn set_entity_id(&mut self, id: String) {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            self.id = uuid;
        }
    }
    fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.created_at
    }
    fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.created_at = Some(ts);
    }
    fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.updated_at
    }
    fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.updated_at = Some(ts);
    }
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.deleted_at
    }
    fn set_deleted_at(&mut self, ts: Option<chrono::DateTime<chrono::Utc>>) {
        self.metadata.deleted_at = ts;
    }
}

impl backbone_orm::EntityRepoMeta for SubscriptionBillingRun {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("subscription_id".to_string(), "uuid".to_string());
        m.insert("company_id".to_string(), "uuid".to_string());
        m.insert("invoice_id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "billing_run_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["idempotency_key"]
    }
    fn company_field() -> Option<&'static str> {
        Some("company_id")
    }
    fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
        &[("subscription", "subscriptions", "subscriptionId")]
    }
}

/// Builder for SubscriptionBillingRun entity
///
/// Provides a fluent API for constructing SubscriptionBillingRun instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionBillingRunBuilder {
    subscription_id: Option<Uuid>,
    company_id: Option<Uuid>,
    period_start: Option<NaiveDate>,
    period_end: Option<NaiveDate>,
    due_date: Option<NaiveDate>,
    grand_total: Option<Decimal>,
    status: Option<BillingRunStatus>,
    invoice_id: Option<Uuid>,
    idempotency_key: Option<String>,
    attempted_at: Option<DateTime<Utc>>,
}

impl SubscriptionBillingRunBuilder {
    /// Set the subscription_id field (required)
    pub fn subscription_id(mut self, value: Uuid) -> Self {
        self.subscription_id = Some(value);
        self
    }

    /// Set the company_id field (required)
    pub fn company_id(mut self, value: Uuid) -> Self {
        self.company_id = Some(value);
        self
    }

    /// Set the period_start field (required)
    pub fn period_start(mut self, value: NaiveDate) -> Self {
        self.period_start = Some(value);
        self
    }

    /// Set the period_end field (required)
    pub fn period_end(mut self, value: NaiveDate) -> Self {
        self.period_end = Some(value);
        self
    }

    /// Set the due_date field (required)
    pub fn due_date(mut self, value: NaiveDate) -> Self {
        self.due_date = Some(value);
        self
    }

    /// Set the grand_total field (required)
    pub fn grand_total(mut self, value: Decimal) -> Self {
        self.grand_total = Some(value);
        self
    }

    /// Set the status field (default: `BillingRunStatus::default()`)
    pub fn status(mut self, value: BillingRunStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the invoice_id field (optional)
    pub fn invoice_id(mut self, value: Uuid) -> Self {
        self.invoice_id = Some(value);
        self
    }

    /// Set the idempotency_key field (required)
    pub fn idempotency_key(mut self, value: String) -> Self {
        self.idempotency_key = Some(value);
        self
    }

    /// Set the attempted_at field (optional)
    pub fn attempted_at(mut self, value: DateTime<Utc>) -> Self {
        self.attempted_at = Some(value);
        self
    }

    /// Build the SubscriptionBillingRun entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<SubscriptionBillingRun, String> {
        let subscription_id = self.subscription_id.ok_or_else(|| "subscription_id is required".to_string())?;
        let company_id = self.company_id.ok_or_else(|| "company_id is required".to_string())?;
        let period_start = self.period_start.ok_or_else(|| "period_start is required".to_string())?;
        let period_end = self.period_end.ok_or_else(|| "period_end is required".to_string())?;
        let due_date = self.due_date.ok_or_else(|| "due_date is required".to_string())?;
        let grand_total = self.grand_total.ok_or_else(|| "grand_total is required".to_string())?;
        let idempotency_key = self.idempotency_key.ok_or_else(|| "idempotency_key is required".to_string())?;

        Ok(SubscriptionBillingRun {
            id: Uuid::new_v4(),
            subscription_id,
            company_id,
            period_start,
            period_end,
            due_date,
            grand_total,
            status: self.status.unwrap_or(BillingRunStatus::default()),
            invoice_id: self.invoice_id,
            idempotency_key,
            attempted_at: self.attempted_at,
            metadata: AuditMetadata::default(),
        })
    }
}
