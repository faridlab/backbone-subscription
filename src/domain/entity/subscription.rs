use chrono::{DateTime, Utc, NaiveDate};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::SubscriptionStatus;
use super::AuditMetadata;

/// Strongly-typed ID for Subscription
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionId(pub Uuid);

impl SubscriptionId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SubscriptionId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for SubscriptionId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<SubscriptionId> for Uuid {
    fn from(id: SubscriptionId) -> Self { id.0 }
}

impl AsRef<Uuid> for SubscriptionId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for SubscriptionId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub subscription_number: String,
    pub customer_id: Uuid,
    pub plan_id: Uuid,
    pub branch_id: Option<Uuid>,
    pub status: SubscriptionStatus,
    pub started_at: NaiveDate,
    pub current_period_start: NaiveDate,
    pub current_period_end: NaiveDate,
    pub next_billing_date: NaiveDate,
    pub currency: String,
    pub cancelled_at: Option<NaiveDate>,
    pub ended_at: Option<NaiveDate>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl Subscription {
    /// Create a builder for Subscription
    pub fn builder() -> SubscriptionBuilder {
        <SubscriptionBuilder as Default>::default()
    }

    /// Create a new Subscription with required fields
    pub fn new(subscription_number: String, customer_id: Uuid, plan_id: Uuid, status: SubscriptionStatus, started_at: NaiveDate, current_period_start: NaiveDate, current_period_end: NaiveDate, next_billing_date: NaiveDate, currency: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            subscription_number,
            customer_id,
            plan_id,
            branch_id: None,
            status,
            started_at,
            current_period_start,
            current_period_end,
            next_billing_date,
            currency,
            cancelled_at: None,
            ended_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> SubscriptionId {
        SubscriptionId(self.id)
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
    pub fn status(&self) -> &SubscriptionStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the branch_id field (chainable)
    pub fn with_branch_id(mut self, value: Uuid) -> Self {
        self.branch_id = Some(value);
        self
    }

    /// Set the cancelled_at field (chainable)
    pub fn with_cancelled_at(mut self, value: NaiveDate) -> Self {
        self.cancelled_at = Some(value);
        self
    }

    /// Set the ended_at field (chainable)
    pub fn with_ended_at(mut self, value: NaiveDate) -> Self {
        self.ended_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "subscription_number" => {
                    if let Ok(v) = serde_json::from_value(value) { self.subscription_number = v; }
                }
                "customer_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.customer_id = v; }
                }
                "plan_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.plan_id = v; }
                }
                "branch_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.branch_id = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "started_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.started_at = v; }
                }
                "current_period_start" => {
                    if let Ok(v) = serde_json::from_value(value) { self.current_period_start = v; }
                }
                "current_period_end" => {
                    if let Ok(v) = serde_json::from_value(value) { self.current_period_end = v; }
                }
                "next_billing_date" => {
                    if let Ok(v) = serde_json::from_value(value) { self.next_billing_date = v; }
                }
                "currency" => {
                    if let Ok(v) = serde_json::from_value(value) { self.currency = v; }
                }
                "cancelled_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.cancelled_at = v; }
                }
                "ended_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.ended_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for Subscription {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "Subscription"
    }
}

impl backbone_core::PersistentEntity for Subscription {
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

impl backbone_orm::EntityRepoMeta for Subscription {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("customer_id".to_string(), "uuid".to_string());
        m.insert("plan_id".to_string(), "uuid".to_string());
        m.insert("branch_id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "subscription_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["subscription_number", "currency"]
    }
    fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
        &[("plan", "subscription_plans", "planId")]
    }
}

/// Builder for Subscription entity
///
/// Provides a fluent API for constructing Subscription instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionBuilder {
    subscription_number: Option<String>,
    customer_id: Option<Uuid>,
    plan_id: Option<Uuid>,
    branch_id: Option<Uuid>,
    status: Option<SubscriptionStatus>,
    started_at: Option<NaiveDate>,
    current_period_start: Option<NaiveDate>,
    current_period_end: Option<NaiveDate>,
    next_billing_date: Option<NaiveDate>,
    currency: Option<String>,
    cancelled_at: Option<NaiveDate>,
    ended_at: Option<NaiveDate>,
}

impl SubscriptionBuilder {
    /// Set the subscription_number field (required)
    pub fn subscription_number(mut self, value: String) -> Self {
        self.subscription_number = Some(value);
        self
    }

    /// Set the customer_id field (required)
    pub fn customer_id(mut self, value: Uuid) -> Self {
        self.customer_id = Some(value);
        self
    }

    /// Set the plan_id field (required)
    pub fn plan_id(mut self, value: Uuid) -> Self {
        self.plan_id = Some(value);
        self
    }

    /// Set the branch_id field (optional)
    pub fn branch_id(mut self, value: Uuid) -> Self {
        self.branch_id = Some(value);
        self
    }

    /// Set the status field (default: `SubscriptionStatus::default()`)
    pub fn status(mut self, value: SubscriptionStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the started_at field (required)
    pub fn started_at(mut self, value: NaiveDate) -> Self {
        self.started_at = Some(value);
        self
    }

    /// Set the current_period_start field (required)
    pub fn current_period_start(mut self, value: NaiveDate) -> Self {
        self.current_period_start = Some(value);
        self
    }

    /// Set the current_period_end field (required)
    pub fn current_period_end(mut self, value: NaiveDate) -> Self {
        self.current_period_end = Some(value);
        self
    }

    /// Set the next_billing_date field (required)
    pub fn next_billing_date(mut self, value: NaiveDate) -> Self {
        self.next_billing_date = Some(value);
        self
    }

    /// Set the currency field (default: `"IDR".to_string()`)
    pub fn currency(mut self, value: String) -> Self {
        self.currency = Some(value);
        self
    }

    /// Set the cancelled_at field (optional)
    pub fn cancelled_at(mut self, value: NaiveDate) -> Self {
        self.cancelled_at = Some(value);
        self
    }

    /// Set the ended_at field (optional)
    pub fn ended_at(mut self, value: NaiveDate) -> Self {
        self.ended_at = Some(value);
        self
    }

    /// Build the Subscription entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<Subscription, String> {
        let subscription_number = self.subscription_number.ok_or_else(|| "subscription_number is required".to_string())?;
        let customer_id = self.customer_id.ok_or_else(|| "customer_id is required".to_string())?;
        let plan_id = self.plan_id.ok_or_else(|| "plan_id is required".to_string())?;
        let started_at = self.started_at.ok_or_else(|| "started_at is required".to_string())?;
        let current_period_start = self.current_period_start.ok_or_else(|| "current_period_start is required".to_string())?;
        let current_period_end = self.current_period_end.ok_or_else(|| "current_period_end is required".to_string())?;
        let next_billing_date = self.next_billing_date.ok_or_else(|| "next_billing_date is required".to_string())?;

        Ok(Subscription {
            id: Uuid::new_v4(),
            subscription_number,
            customer_id,
            plan_id,
            branch_id: self.branch_id,
            status: self.status.unwrap_or_default(),
            started_at,
            current_period_start,
            current_period_end,
            next_billing_date,
            currency: self.currency.unwrap_or("IDR".to_string()),
            cancelled_at: self.cancelled_at,
            ended_at: self.ended_at,
            metadata: AuditMetadata::default(),
        })
    }
}
