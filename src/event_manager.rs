use crate::models::{Event, EventType, TimeRecord};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;

pub struct EventManager {
    events: HashMap<Uuid, Event>,
    time_records: HashMap<Uuid, TimeRecord>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
            time_records: HashMap::new(),
        }
    }

    /// 添加项目相关事件
    pub fn add_project_event(
        &mut self,
        title: String,
        description: Option<String>,
        project_id: Uuid,
        start_time: Option<DateTime<Utc>>,
    ) -> Uuid {
        let start_time = start_time.unwrap_or_else(Utc::now);
        let event = Event::new(
            title,
            description,
            EventType::ProjectRelated(project_id),
            start_time,
        );
        let event_id = event.id;
        self.events.insert(event_id, event);
        event_id
    }

    /// 添加项目外事件
    pub fn add_non_project_event(
        &mut self,
        title: String,
        description: Option<String>,
        start_time: Option<DateTime<Utc>>,
    ) -> Uuid {
        let start_time = start_time.unwrap_or_else(Utc::now);
        let event = Event::new(title, description, EventType::NonProject, start_time);
        let event_id = event.id;
        self.events.insert(event_id, event);
        event_id
    }

    /// 设置事件结束时间
    pub fn set_event_end_time(
        &mut self,
        event_id: Uuid,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        let end_time = end_time.unwrap_or_else(Utc::now);

        if let Some(event) = self.events.get_mut(&event_id) {
            if event.end_time.is_some() {
                return Err("事件已经结束".to_string());
            }

            if end_time <= event.start_time {
                return Err("结束时间必须晚于开始时间".to_string());
            }

            event.set_end_time(end_time);

            // 创建时间记录
            let project_id = match event.event_type {
                EventType::ProjectRelated(id) => Some(id),
                EventType::NonProject => None,
            };

            let time_record = TimeRecord::new(event_id, project_id, event.start_time, end_time);

            self.time_records.insert(time_record.id, time_record);
            Ok(())
        } else {
            Err("事件不存在".to_string())
        }
    }

    /// 获取事件
    pub fn get_event(&self, event_id: Uuid) -> Option<&Event> {
        self.events.get(&event_id)
    }

    /// 获取所有事件
    pub fn get_all_events(&self) -> Vec<&Event> {
        self.events.values().collect()
    }

    /// 获取进行中的事件
    pub fn get_active_events(&self) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| event.end_time.is_none())
            .collect()
    }

    /// 获取已完成的事件
    pub fn get_completed_events(&self) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| event.end_time.is_some())
            .collect()
    }

    /// 获取项目相关事件
    pub fn get_project_events(&self, project_id: Uuid) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| match event.event_type {
                EventType::ProjectRelated(id) => id == project_id,
                EventType::NonProject => false,
            })
            .collect()
    }

    /// 获取项目外事件
    pub fn get_non_project_events(&self) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| matches!(event.event_type, EventType::NonProject))
            .collect()
    }

    /// 删除事件
    pub fn delete_event(&mut self, event_id: Uuid) -> Result<(), String> {
        if self.events.remove(&event_id).is_none() {
            return Err("事件不存在".to_string());
        }

        // 同时删除相关的时间记录
        self.time_records
            .retain(|_, record| record.event_id != event_id);

        Ok(())
    }

    /// 更新事件信息
    pub fn update_event(
        &mut self,
        event_id: Uuid,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        if let Some(event) = self.events.get_mut(&event_id) {
            if let Some(title) = title {
                event.title = title;
            }
            if description.is_some() {
                event.description = description;
            }
            Ok(())
        } else {
            Err("事件不存在".to_string())
        }
    }

    /// 获取时间记录
    pub fn get_time_record(&self, record_id: Uuid) -> Option<&TimeRecord> {
        self.time_records.get(&record_id)
    }

    /// 获取所有时间记录
    pub fn get_all_time_records(&self) -> Vec<&TimeRecord> {
        self.time_records.values().collect()
    }

    /// 获取事件的时间记录
    pub fn get_event_time_record(&self, event_id: Uuid) -> Option<&TimeRecord> {
        self.time_records
            .values()
            .find(|record| record.event_id == event_id)
    }

    /// 获取项目的时间记录
    pub fn get_project_time_records(&self, project_id: Uuid) -> Vec<&TimeRecord> {
        self.time_records
            .values()
            .filter(|record| record.project_id == Some(project_id))
            .collect()
    }

    /// 获取项目外的时间记录
    pub fn get_non_project_time_records(&self) -> Vec<&TimeRecord> {
        self.time_records
            .values()
            .filter(|record| record.project_id.is_none())
            .collect()
    }

    /// 获取事件数量
    pub fn get_event_count(&self) -> usize {
        self.events.len()
    }

    /// 检查事件是否存在
    pub fn event_exists(&self, event_id: Uuid) -> bool {
        self.events.contains_key(&event_id)
    }

    /// 获取指定时间范围内的事件
    pub fn get_events_in_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<&Event> {
        self.events
            .values()
            .filter(|event| event.start_time >= start_time && event.start_time <= end_time)
            .collect()
    }

    /// 获取指定时间范围内的时间记录
    pub fn get_time_records_in_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Vec<&TimeRecord> {
        self.time_records
            .values()
            .filter(|record| record.start_time >= start_time && record.start_time <= end_time)
            .collect()
    }
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_add_project_event() {
        let mut manager = EventManager::new();
        let project_id = Uuid::new_v4();

        let event_id = manager.add_project_event(
            "测试事件".to_string(),
            Some("测试描述".to_string()),
            project_id,
            None,
        );

        assert_eq!(manager.get_event_count(), 1);
        assert!(manager.event_exists(event_id));

        let event = manager.get_event(event_id).unwrap();
        assert_eq!(event.title, "测试事件");
        assert_eq!(event.description, Some("测试描述".to_string()));
        assert!(matches!(event.event_type, EventType::ProjectRelated(id) if id == project_id));
    }

    #[test]
    fn test_add_non_project_event() {
        let mut manager = EventManager::new();

        let event_id = manager.add_non_project_event("非项目事件".to_string(), None, None);

        assert_eq!(manager.get_event_count(), 1);

        let event = manager.get_event(event_id).unwrap();
        assert_eq!(event.title, "非项目事件");
        assert!(matches!(event.event_type, EventType::NonProject));
    }

    #[test]
    fn test_set_event_end_time() {
        let mut manager = EventManager::new();
        let project_id = Uuid::new_v4();

        let event_id = manager.add_project_event("测试事件".to_string(), None, project_id, None);

        let end_time = Utc::now() + Duration::hours(1);
        manager
            .set_event_end_time(event_id, Some(end_time))
            .unwrap();

        let event = manager.get_event(event_id).unwrap();
        assert!(event.is_completed());
        assert_eq!(event.end_time, Some(end_time));

        // 检查是否创建了时间记录
        let time_record = manager.get_event_time_record(event_id).unwrap();
        assert_eq!(time_record.event_id, event_id);
        assert_eq!(time_record.project_id, Some(project_id));
    }

    #[test]
    fn test_get_project_events() {
        let mut manager = EventManager::new();
        let project_id1 = Uuid::new_v4();
        let project_id2 = Uuid::new_v4();

        manager.add_project_event("项目1事件".to_string(), None, project_id1, None);
        manager.add_project_event("项目2事件".to_string(), None, project_id2, None);
        manager.add_non_project_event("非项目事件".to_string(), None, None);

        let project1_events = manager.get_project_events(project_id1);
        assert_eq!(project1_events.len(), 1);
        assert_eq!(project1_events[0].title, "项目1事件");

        let project2_events = manager.get_project_events(project_id2);
        assert_eq!(project2_events.len(), 1);
        assert_eq!(project2_events[0].title, "项目2事件");

        let non_project_events = manager.get_non_project_events();
        assert_eq!(non_project_events.len(), 1);
        assert_eq!(non_project_events[0].title, "非项目事件");
    }
}
