use chrono::{DateTime, Utc};
use core::convert::From;
use core::option::Option;
use opsgenie_rs::models::base_alert::BaseAlert;

#[derive(Eq, PartialEq, Hash)]
pub struct Alert {
    pub(crate) tiny_id: Option<String>,
    pub(crate) alias: Option<String>,
    pub(crate) message: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) acknowledged: Option<bool>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) created_at: Option<DateTime<Utc>>,
    pub(crate) updated_at: Option<DateTime<Utc>>,
    pub(crate) priority: Option<String>,
}

impl From<BaseAlert> for Alert {
    fn from(og_alert: BaseAlert) -> Self {
        Self {
            tiny_id: og_alert.tiny_id,
            alias: og_alert.alias,
            message: og_alert.message,
            status: og_alert.status,
            acknowledged: og_alert.acknowledged,
            tags: og_alert.tags,
            created_at: og_alert.created_at,
            updated_at: og_alert.updated_at,
            priority: og_alert.priority,
        }
    }
}
