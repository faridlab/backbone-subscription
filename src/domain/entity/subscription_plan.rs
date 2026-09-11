use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

use super::BillingCycle;
use super::SubscriptionPlanStatus;
use super::AuditMetadata;

/// Strongly-typed ID for SubscriptionPlan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SubscriptionPlanId(pub Uuid);

impl SubscriptionPlanId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for SubscriptionPlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for SubscriptionPlanId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for SubscriptionPlanId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<SubscriptionPlanId> for Uuid {
    fn from(id: SubscriptionPlanId) -> Self { id.0 }
}

impl AsRef<Uuid> for SubscriptionPlanId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for SubscriptionPlanId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SubscriptionPlan {
    pub id: Uuid,
    pub plan_code: String,
    pub name: String,
    pub description: Option<String>,
    pub billing_cycle: BillingCycle,
    pub billing_day: i32,
    pub currency: String,
    pub receivable_account_id: Uuid,
    pub price: Decimal,
    pub trial_days: Option<i32>,
    pub status: SubscriptionPlanStatus,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl SubscriptionPlan {
    /// Create a builder for SubscriptionPlan
    pub fn builder() -> SubscriptionPlanBuilder {
        <SubscriptionPlanBuilder as Default>::default()
    }

    /// Create a new SubscriptionPlan with required fields
    pub fn new(plan_code: String, name: String, billing_cycle: BillingCycle, billing_day: i32, currency: String, receivable_account_id: Uuid, price: Decimal, status: SubscriptionPlanStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            plan_code,
            name,
            description: None,
            billing_cycle,
            billing_day,
            currency,
            receivable_account_id,
            price,
            trial_days: None,
            status,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> SubscriptionPlanId {
        SubscriptionPlanId(self.id)
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
    pub fn status(&self) -> &SubscriptionPlanStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the description field (chainable)
    pub fn with_description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the trial_days field (chainable)
    pub fn with_trial_days(mut self, value: i32) -> Self {
        self.trial_days = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "plan_code" => {
                    if let Ok(v) = serde_json::from_value(value) { self.plan_code = v; }
                }
                "name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.name = v; }
                }
                "description" => {
                    if let Ok(v) = serde_json::from_value(value) { self.description = v; }
                }
                "billing_cycle" => {
                    if let Ok(v) = serde_json::from_value(value) { self.billing_cycle = v; }
                }
                "billing_day" => {
                    if let Ok(v) = serde_json::from_value(value) { self.billing_day = v; }
                }
                "currency" => {
                    if let Ok(v) = serde_json::from_value(value) { self.currency = v; }
                }
                "receivable_account_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.receivable_account_id = v; }
                }
                "price" => {
                    if let Ok(v) = serde_json::from_value(value) { self.price = v; }
                }
                "trial_days" => {
                    if let Ok(v) = serde_json::from_value(value) { self.trial_days = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for SubscriptionPlan {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "SubscriptionPlan"
    }
}

impl backbone_core::PersistentEntity for SubscriptionPlan {
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

impl backbone_orm::EntityRepoMeta for SubscriptionPlan {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("receivable_account_id".to_string(), "uuid".to_string());
        m.insert("billing_cycle".to_string(), "billing_cycle".to_string());
        m.insert("status".to_string(), "subscription_plan_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["plan_code", "name", "currency"]
    }
}

/// Builder for SubscriptionPlan entity
///
/// Provides a fluent API for constructing SubscriptionPlan instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct SubscriptionPlanBuilder {
    plan_code: Option<String>,
    name: Option<String>,
    description: Option<String>,
    billing_cycle: Option<BillingCycle>,
    billing_day: Option<i32>,
    currency: Option<String>,
    receivable_account_id: Option<Uuid>,
    price: Option<Decimal>,
    trial_days: Option<i32>,
    status: Option<SubscriptionPlanStatus>,
}

impl SubscriptionPlanBuilder {
    /// Set the plan_code field (required)
    pub fn plan_code(mut self, value: String) -> Self {
        self.plan_code = Some(value);
        self
    }

    /// Set the name field (required)
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Set the description field (optional)
    pub fn description(mut self, value: String) -> Self {
        self.description = Some(value);
        self
    }

    /// Set the billing_cycle field (default: `BillingCycle::default()`)
    pub fn billing_cycle(mut self, value: BillingCycle) -> Self {
        self.billing_cycle = Some(value);
        self
    }

    /// Set the billing_day field (default: `1`)
    pub fn billing_day(mut self, value: i32) -> Self {
        self.billing_day = Some(value);
        self
    }

    /// Set the currency field (default: `"IDR".to_string()`)
    pub fn currency(mut self, value: String) -> Self {
        self.currency = Some(value);
        self
    }

    /// Set the receivable_account_id field (required)
    pub fn receivable_account_id(mut self, value: Uuid) -> Self {
        self.receivable_account_id = Some(value);
        self
    }

    /// Set the price field (required)
    pub fn price(mut self, value: Decimal) -> Self {
        self.price = Some(value);
        self
    }

    /// Set the trial_days field (optional)
    pub fn trial_days(mut self, value: i32) -> Self {
        self.trial_days = Some(value);
        self
    }

    /// Set the status field (default: `SubscriptionPlanStatus::default()`)
    pub fn status(mut self, value: SubscriptionPlanStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Build the SubscriptionPlan entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<SubscriptionPlan, String> {
        let plan_code = self.plan_code.ok_or_else(|| "plan_code is required".to_string())?;
        let name = self.name.ok_or_else(|| "name is required".to_string())?;
        let receivable_account_id = self.receivable_account_id.ok_or_else(|| "receivable_account_id is required".to_string())?;
        let price = self.price.ok_or_else(|| "price is required".to_string())?;

        Ok(SubscriptionPlan {
            id: Uuid::new_v4(),
            plan_code,
            name,
            description: self.description,
            billing_cycle: self.billing_cycle.unwrap_or_default(),
            billing_day: self.billing_day.unwrap_or(1),
            currency: self.currency.unwrap_or("IDR".to_string()),
            receivable_account_id,
            price,
            trial_days: self.trial_days,
            status: self.status.unwrap_or_default(),
            metadata: AuditMetadata::default(),
        })
    }
}
