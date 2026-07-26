use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "billing_run_status", rename_all = "snake_case")]
pub enum BillingRunStatus {
    Pending,
    Invoiced,
    Failed,
}

impl std::fmt::Display for BillingRunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Invoiced => write!(f, "invoiced"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl FromStr for BillingRunStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "invoiced" => Ok(Self::Invoiced),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown BillingRunStatus variant: {}", s)),
        }
    }
}

impl Default for BillingRunStatus {
    fn default() -> Self {
        Self::Pending
    }
}
