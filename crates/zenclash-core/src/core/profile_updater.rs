use crate::config::ProfileItem;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum ProfileUpdaterError {
    #[error("Profile not found: {0}")]
    NotFound(String),

    #[error("Failed to parse cron expression: {0}")]
    CronParseError(String),

    #[error("Update failed: {0}")]
    UpdateFailed(String),
}

struct ScheduledUpdate {
    profile_id: String,
    interval_minutes: Option<u64>,
    cron_expression: Option<String>,
    last_update: Option<Instant>,
    next_update: Option<Instant>,
}

pub struct ProfileUpdater {
    schedules: Arc<RwLock<HashMap<String, ScheduledUpdate>>>,
    running: Arc<RwLock<bool>>,
}

impl ProfileUpdater {
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn add_interval_update(&self, profile_id: &str, interval_minutes: u64) {
        let mut schedules = self.schedules.write().await;

        let next_update = Instant::now() + Duration::from_secs(interval_minutes * 60);

        schedules.insert(
            profile_id.to_string(),
            ScheduledUpdate {
                profile_id: profile_id.to_string(),
                interval_minutes: Some(interval_minutes),
                cron_expression: None,
                last_update: None,
                next_update: Some(next_update),
            },
        );
    }

    pub async fn add_cron_update(
        &self,
        profile_id: &str,
        cron_expression: &str,
    ) -> Result<(), ProfileUpdaterError> {
        let next = Self::parse_cron_next(cron_expression)?;

        let mut schedules = self.schedules.write().await;

        schedules.insert(
            profile_id.to_string(),
            ScheduledUpdate {
                profile_id: profile_id.to_string(),
                interval_minutes: None,
                cron_expression: Some(cron_expression.to_string()),
                last_update: None,
                next_update: Some(next),
            },
        );

        Ok(())
    }

    pub async fn remove_update(&self, profile_id: &str) {
        let mut schedules = self.schedules.write().await;
        schedules.remove(profile_id);
    }

    pub async fn clear_all(&self) {
        let mut schedules = self.schedules.write().await;
        schedules.clear();
    }

    pub async fn get_next_update(&self, profile_id: &str) -> Option<Instant> {
        let schedules = self.schedules.read().await;
        schedules.get(profile_id).and_then(|s| s.next_update)
    }

    fn parse_cron_next(expression: &str) -> Result<Instant, ProfileUpdaterError> {
        let parts: Vec<&str> = expression.split_whitespace().collect();

        if parts.len() < 5 {
            return Err(ProfileUpdaterError::CronParseError(
                "Invalid cron expression. Expected 5 parts: minute hour day month weekday".into(),
            ));
        }

        let now = chrono::Local::now();
        let mut next = now;

        loop {
            next = next + chrono::Duration::minutes(1);

            let minute: i32 = next.minute() as i32;
            let hour: i32 = next.hour() as i32;
            let day: i32 = next.day() as i32;
            let month: i32 = next.month() as i32;
            let weekday: i32 = next.weekday().num_days_from_sunday() as i32;

            if !Self::cron_field_matches(parts[0], minute, 0, 59)? {
                continue;
            }
            if !Self::cron_field_matches(parts[1], hour, 0, 23)? {
                continue;
            }
            if !Self::cron_field_matches(parts[2], day, 1, 31)? {
                continue;
            }
            if !Self::cron_field_matches(parts[3], month, 1, 12)? {
                continue;
            }
            if !Self::cron_field_matches(parts[4], weekday, 0, 6)? {
                continue;
            }

            break;
        }

        let duration = (next - now).to_std().unwrap_or(Duration::ZERO);
        Ok(Instant::now() + duration)
    }

    fn cron_field_matches(
        field: &str,
        value: i32,
        min: i32,
        max: i32,
    ) -> Result<bool, ProfileUpdaterError> {
        if field == "*" {
            return Ok(true);
        }

        if field.contains(',') {
            for part in field.split(',') {
                if Self::cron_field_matches(part, value, min, max)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        if field.contains('/') {
            let parts: Vec<&str> = field.split('/').collect();
            let base = if parts[0] == "*" {
                min
            } else {
                parts[0].parse::<i32>().map_err(|_| {
                    ProfileUpdaterError::CronParseError(format!("Invalid cron field: {}", field))
                })?
            };
            let step = parts[1].parse::<i32>().map_err(|_| {
                ProfileUpdaterError::CronParseError(format!("Invalid cron step: {}", parts[1]))
            })?;

            return Ok((value - base) % step == 0);
        }

        if field.contains('-') {
            let parts: Vec<&str> = field.split('-').collect();
            let start = parts[0].parse::<i32>().map_err(|_| {
                ProfileUpdaterError::CronParseError(format!("Invalid cron range: {}", field))
            })?;
            let end = parts[1].parse::<i32>().map_err(|_| {
                ProfileUpdaterError::CronParseError(format!("Invalid cron range: {}", field))
            })?;

            return Ok(value >= start && value <= end);
        }

        let field_value = field.parse::<i32>().map_err(|_| {
            ProfileUpdaterError::CronParseError(format!("Invalid cron field: {}", field))
        })?;

        Ok(value == field_value)
    }

    fn calculate_next_update(schedule: &ScheduledUpdate) -> Option<Instant> {
        if let Some(interval) = schedule.interval_minutes {
            Some(Instant::now() + Duration::from_secs(interval * 60))
        } else if let Some(ref cron) = schedule.cron_expression {
            Self::parse_cron_next(cron).ok()
        } else {
            None
        }
    }

    pub async fn start<F>(&self, mut on_update: F)
    where
        F: FnMut(String) + Send + 'static,
    {
        let mut running = self.running.write().await;
        *running = true;
        drop(running);

        let schedules = self.schedules.clone();
        let running_flag = self.running.clone();

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(60));

            loop {
                check_interval.tick().await;

                if !*running_flag.read().await {
                    break;
                }

                let now = Instant::now();
                let mut schedules_guard = schedules.write().await;

                let due_updates: Vec<String> = schedules_guard
                    .iter()
                    .filter(|(_, schedule)| {
                        schedule
                            .next_update
                            .map(|next| next <= now)
                            .unwrap_or(false)
                    })
                    .map(|(id, _)| id.clone())
                    .collect();

                for profile_id in due_updates {
                    if let Some(schedule) = schedules_guard.get_mut(&profile_id) {
                        schedule.last_update = Some(now);
                        schedule.next_update = Self::calculate_next_update(schedule);

                        let id = profile_id.clone();
                        drop(schedules_guard);
                        on_update(id);
                        schedules_guard = schedules.write().await;
                    }
                }
            }
        });
    }

    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
    }

    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    pub async fn get_scheduled_profiles(&self) -> Vec<String> {
        let schedules = self.schedules.read().await;
        schedules.keys().cloned().collect()
    }
}

impl Default for ProfileUpdater {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn update_profile(profile: &ProfileItem) -> Result<(), ProfileUpdaterError> {
    let url = profile
        .url
        .as_ref()
        .ok_or_else(|| ProfileUpdaterError::UpdateFailed("Profile has no URL".into()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| ProfileUpdaterError::UpdateFailed(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ProfileUpdaterError::UpdateFailed(e.to_string()))?;

    if !response.status().is_success() {
        return Err(ProfileUpdaterError::UpdateFailed(format!(
            "HTTP error: {}",
            response.status()
        )));
    }

    let content = response
        .text()
        .await
        .map_err(|e| ProfileUpdaterError::UpdateFailed(e.to_string()))?;

    let file_path = profile
        .file_path()
        .ok_or_else(|| ProfileUpdaterError::UpdateFailed("No file path for profile".into()))?;

    std::fs::write(&file_path, content)
        .map_err(|e| ProfileUpdaterError::UpdateFailed(e.to_string()))?;

    Ok(())
}
