use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::str::FromStr;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "subscription_status", rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Trial,
    Active,
    Paused,
    PastDue,
    Cancelled,
    Expired,
}

impl std::fmt::Display for SubscriptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trial => write!(f, "trial"),
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::PastDue => write!(f, "past_due"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Expired => write!(f, "expired"),
        }
    }
}

impl FromStr for SubscriptionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "trial" => Ok(Self::Trial),
            "active" => Ok(Self::Active),
            "paused" => Ok(Self::Paused),
            "past_due" => Ok(Self::PastDue),
            "cancelled" => Ok(Self::Cancelled),
            "expired" => Ok(Self::Expired),
            _ => Err(format!("Unknown SubscriptionStatus variant: {}", s)),
        }
    }
}

impl Default for SubscriptionStatus {
    fn default() -> Self {
        Self::Active
    }
}
