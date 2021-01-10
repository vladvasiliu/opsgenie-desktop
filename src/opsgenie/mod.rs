use std::collections::HashMap;

use self::alert::Alert;
use crate::config::Config;
use chrono::{DateTime, Duration, Utc};
use color_eyre::Result;
use log::{debug, info};
use notify_rust::{Hint, Notification, Urgency};
use opsgenie_rs::apis::alert_api::list_alerts;
use opsgenie_rs::apis::configuration::{ApiKey, Configuration as OpsGenieConfig};
use tokio::signal::ctrl_c;
use tokio::time::{interval, sleep};

pub mod alert;
pub type AlertMap = HashMap<String, Alert>;

pub struct OpsGenieInterface {
    alerts: AlertMap,
    ops_genie_config: OpsGenieConfig,
    request_limit: u8,
    history_days: u8,
    update_interval: u8,
}

impl OpsGenieInterface {
    pub fn new_with_config(config: Config) -> Self {
        let api_key = Some(ApiKey {
            prefix: Some("GenieKey".to_string()),
            key: config.api_key,
        });

        let ops_genie_config = OpsGenieConfig {
            api_key,
            base_path: config.base_path.to_string(),
            ..Default::default()
        };

        Self {
            alerts: AlertMap::new(),
            ops_genie_config,
            request_limit: config.request_limit,
            history_days: config.history_days,
            update_interval: config.update_interval,
        }
    }

    fn get_query(&self) -> String {
        let latest_update = self
            .get_latest_update()
            .unwrap_or_else(|| Utc::now() - Duration::days(self.history_days as i64));
        format!("status: open OR updatedAt >= {}", latest_update.timestamp())
    }

    /// Returns the latest update time of all alerts or None if there are no alerts
    pub fn get_latest_update<'a>(&self) -> Option<DateTime<Utc>> {
        self.alerts
            .iter()
            .max_by_key(|(_, alert)| alert.updated_at)
            .and_then(|(_, alert)| alert.updated_at)
    }

    /// Fetches a new set of alerts from the API
    async fn retrieve_alerts(&self) -> Result<AlertMap> {
        let query = self.get_query();
        debug!("Retrieving alerts with query: \"{}\"", query);
        let mut new_alerts = AlertMap::new();
        let mut offset = 0;
        loop {
            let result = list_alerts(
                &self.ops_genie_config,
                Some(&query),
                None,
                None,
                Some(offset),
                Some(self.request_limit as i32),
                Some("created_at"),
                Some("asc"),
            )
            .await?;

            // TODO: Convert to stream
            if let Some(alert_vec) = result.data {
                for base_alert in alert_vec {
                    let alert_id = base_alert.id;
                    let alert = Alert {
                        tiny_id: base_alert.tiny_id,
                        alias: base_alert.alias,
                        message: base_alert.message,
                        status: base_alert.status,
                        acknowledged: base_alert.acknowledged,
                        tags: base_alert.tags,
                        created_at: base_alert.created_at,
                        updated_at: base_alert.updated_at,
                        priority: base_alert.priority,
                    };
                    new_alerts.insert(alert_id, alert);
                }
            }

            if let Some(page_details) = result.paging {
                if page_details.next.is_some() {
                    offset += self.request_limit as i32;
                    // Don't break the API rate limit
                    debug!("Sleeping for 1s between API calls...");
                    sleep(Duration::seconds(1).to_std().unwrap()).await;
                    continue;
                }
            }
            break;
        }
        debug!("Retrieved {} alerts.", new_alerts.len());
        Ok(new_alerts)
    }

    /// Updates the state with the new alerts and returns those alerts that are new
    pub async fn update_alerts(&mut self) -> Result<Vec<String>> {
        let mut new_alerts = vec![];
        let retrieved_alerts = self.retrieve_alerts().await?;

        for (new_alert_id, new_alert) in retrieved_alerts.into_iter() {
            if self
                .alerts
                .insert(new_alert_id.clone(), new_alert)
                .is_none()
            {
                new_alerts.push(new_alert_id)
            }
        }

        Ok(new_alerts)
    }

    fn handle_new_alerts(&self, new_alerts: Vec<String>) {
        for alert_id in new_alerts {
            if let Some(alert) = self.alerts.get(&alert_id).filter(|alert| {
                alert.acknowledged == Some(false) && alert.status == Some("open".to_string())
            }) {
                let urgency = match alert.priority.as_ref() {
                    Some(p) if p == "P1" || p == "P2" => Urgency::Critical,
                    Some(p) if p == "P3" || p == "P4" => Urgency::Normal,
                    _ => Urgency::Low,
                };
                Notification::new()
                    .summary(alert.message.as_ref().unwrap())
                    .hint(Hint::Urgency(urgency))
                    .show()
                    .unwrap();
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut timer = interval(
            Duration::seconds(self.update_interval as i64)
                .to_std()
                .unwrap(),
        );
        loop {
            tokio::select! {
                _ = timer.tick() => {
                    let new_alerts = self.update_alerts().await?;
                    self.handle_new_alerts(new_alerts);
                }
                _ = ctrl_c() => {
                    info!("Received ctrl-c. Shutting down.");
                    break;
                }
            }
        }
        Ok(())
    }
}
