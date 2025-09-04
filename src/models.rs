use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

impl Project {
    pub fn new(name: String, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            created_at: Utc::now(),
            is_active: false,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    ProjectRelated(Uuid), // 关联到特定项目
    NonProject,           // 项目外事件
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_type: EventType,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Event {
    pub fn new(
        title: String,
        description: Option<String>,
        event_type: EventType,
        start_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            event_type,
            start_time,
            end_time: None,
            created_at: Utc::now(),
        }
    }

    pub fn set_end_time(&mut self, end_time: DateTime<Utc>) {
        self.end_time = Some(end_time);
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match self.end_time {
            Some(end) => Some(end.signed_duration_since(self.start_time)),
            None => None,
        }
    }

    pub fn is_completed(&self) -> bool {
        self.end_time.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRecord {
    pub id: Uuid,
    pub event_id: Uuid,
    pub project_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration_minutes: i64,
    pub created_at: DateTime<Utc>,
}

impl TimeRecord {
    pub fn new(
        event_id: Uuid,
        project_id: Option<Uuid>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Self {
        let duration = end_time.signed_duration_since(start_time);
        Self {
            id: Uuid::new_v4(),
            event_id,
            project_id,
            start_time,
            end_time,
            duration_minutes: duration.num_minutes(),
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReport {
    pub id: Uuid,
    pub week_start: DateTime<Utc>,
    pub week_end: DateTime<Utc>,
    pub total_project_time_minutes: i64,
    pub total_non_project_time_minutes: i64,
    pub project_breakdown: Vec<ProjectTimeBreakdown>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectTimeBreakdown {
    pub project_id: Uuid,
    pub project_name: String,
    pub total_time_minutes: i64,
    pub event_count: i32,
}

impl WeeklyReport {
    pub fn new(week_start: DateTime<Utc>, week_end: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            week_start,
            week_end,
            total_project_time_minutes: 0,
            total_non_project_time_minutes: 0,
            project_breakdown: Vec::new(),
            generated_at: Utc::now(),
        }
    }
}
