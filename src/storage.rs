use crate::event_manager::EventManager;
use crate::models::{Event, Project, TimeRecord, WeeklyReport};
use crate::project_manager::ProjectManager;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppData {
    pub projects: Vec<Project>,
    pub events: Vec<Event>,
    pub time_records: Vec<TimeRecord>,
    pub weekly_reports: Vec<WeeklyReport>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            events: Vec::new(),
            time_records: Vec::new(),
            weekly_reports: Vec::new(),
        }
    }

    pub fn from_managers(project_manager: &ProjectManager, event_manager: &EventManager) -> Self {
        Self {
            projects: project_manager
                .get_all_projects()
                .into_iter()
                .cloned()
                .collect(),
            events: event_manager
                .get_all_events()
                .into_iter()
                .cloned()
                .collect(),
            time_records: event_manager
                .get_all_time_records()
                .into_iter()
                .cloned()
                .collect(),
            weekly_reports: Vec::new(), // 暂时不保存报表，因为可以重新生成
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Storage {
    data_dir: String,
}

impl Storage {
    pub fn new(data_dir: String) -> Self {
        // 确保数据目录存在
        if !Path::new(&data_dir).exists() {
            fs::create_dir_all(&data_dir).unwrap_or_else(|e| {
                eprintln!("无法创建数据目录 {}: {}", data_dir, e);
            });
        }

        Self { data_dir }
    }

    pub fn get_data_file_path(&self) -> String {
        format!("{}/app_data.json", self.data_dir)
    }

    pub fn get_backup_file_path(&self, timestamp: &str) -> String {
        format!("{}/backup_{}.json", self.data_dir, timestamp)
    }

    /// 保存应用数据到文件
    pub fn save_data(
        &self,
        project_manager: &ProjectManager,
        event_manager: &EventManager,
    ) -> io::Result<()> {
        let app_data = AppData::from_managers(project_manager, event_manager);
        let json_data = serde_json::to_string_pretty(&app_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let file_path = self.get_data_file_path();
        let mut file = fs::File::create(&file_path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(())
    }

    /// 从文件加载应用数据
    pub fn load_data(&self) -> io::Result<AppData> {
        let file_path = self.get_data_file_path();

        if !Path::new(&file_path).exists() {
            return Ok(AppData::new());
        }

        let mut file = fs::File::open(&file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let app_data: AppData =
            serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(app_data)
    }

    /// 创建数据备份
    pub fn create_backup(
        &self,
        project_manager: &ProjectManager,
        event_manager: &EventManager,
    ) -> io::Result<String> {
        let app_data = AppData::from_managers(project_manager, event_manager);
        let json_data = serde_json::to_string_pretty(&app_data)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_path = self.get_backup_file_path(&timestamp);

        let mut file = fs::File::create(&backup_path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(backup_path)
    }

    /// 从备份恢复数据
    pub fn restore_from_backup(&self, backup_path: &str) -> io::Result<AppData> {
        if !Path::new(backup_path).exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "备份文件不存在"));
        }

        let mut file = fs::File::open(backup_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let app_data: AppData =
            serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(app_data)
    }

    /// 列出所有备份文件
    pub fn list_backups(&self) -> io::Result<Vec<String>> {
        let mut backups = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("backup_") && file_name.ends_with(".json") {
                        backups.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // 按文件名排序（最新的在前）
        backups.sort_by(|a, b| b.cmp(a));

        Ok(backups)
    }

    /// 删除备份文件
    pub fn delete_backup(&self, backup_path: &str) -> io::Result<()> {
        fs::remove_file(backup_path)
    }

    /// 导出数据到CSV格式
    pub fn export_to_csv(
        &self,
        project_manager: &ProjectManager,
        event_manager: &EventManager,
    ) -> io::Result<String> {
        let mut csv_content = String::new();

        // CSV头部
        csv_content.push_str("类型,名称,描述,项目,开始时间,结束时间,持续时间(分钟)\n");

        // 导出项目
        for project in project_manager.get_all_projects() {
            csv_content.push_str(&format!(
                "项目,\"{}\",\"{}\",N/A,N/A,N/A,N/A\n",
                project.name,
                project.description.as_deref().unwrap_or("")
            ));
        }

        // 导出事件
        for event in event_manager.get_all_events() {
            let project_name = match &event.event_type {
                crate::models::EventType::ProjectRelated(project_id) => project_manager
                    .get_project(*project_id)
                    .map(|p| p.name.as_str())
                    .unwrap_or("未知项目"),
                crate::models::EventType::NonProject => "项目外",
            };

            let duration = if let Some(end_time) = event.end_time {
                end_time
                    .signed_duration_since(event.start_time)
                    .num_minutes()
                    .to_string()
            } else {
                "进行中".to_string()
            };

            csv_content.push_str(&format!(
                "事件,\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",{}\n",
                event.title,
                event.description.as_deref().unwrap_or(""),
                project_name,
                event.start_time.format("%Y-%m-%d %H:%M:%S"),
                event
                    .end_time
                    .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "N/A".to_string()),
                duration
            ));
        }

        // 导出时间记录
        for record in event_manager.get_all_time_records() {
            let project_name = record
                .project_id
                .and_then(|id| project_manager.get_project(id))
                .map(|p| p.name.as_str())
                .unwrap_or("项目外");

            csv_content.push_str(&format!(
                "时间记录,N/A,N/A,\"{}\",\"{}\",\"{}\",{}\n",
                project_name,
                record.start_time.format("%Y-%m-%d %H:%M:%S"),
                record.end_time.format("%Y-%m-%d %H:%M:%S"),
                record.duration_minutes
            ));
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let csv_path = format!("{}/export_{}.csv", self.data_dir, timestamp);

        let mut file = fs::File::create(&csv_path)?;
        file.write_all(csv_content.as_bytes())?;

        Ok(csv_path)
    }

    /// 获取数据目录大小
    pub fn get_data_dir_size(&self) -> io::Result<u64> {
        let mut total_size = 0;

        if let Ok(entries) = fs::read_dir(&self.data_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// 清理旧备份文件（保留最近N个）
    pub fn cleanup_old_backups(&self, keep_count: usize) -> io::Result<usize> {
        let mut backups = self.list_backups()?;

        if backups.len() > keep_count {
            let to_delete = backups.split_off(keep_count);
            let mut deleted_count = 0;

            for backup_path in to_delete {
                if let Err(e) = self.delete_backup(&backup_path) {
                    eprintln!("删除备份文件失败 {}: {}", backup_path, e);
                } else {
                    deleted_count += 1;
                }
            }

            Ok(deleted_count)
        } else {
            Ok(0)
        }
    }

    /// 检查数据完整性
    pub fn check_data_integrity(&self, app_data: &AppData) -> Vec<String> {
        let mut issues = Vec::new();

        // 检查项目ID重复
        let mut project_ids = std::collections::HashSet::new();
        for project in &app_data.projects {
            if project_ids.contains(&project.id) {
                issues.push(format!("项目ID重复: {}", project.id));
            }
            project_ids.insert(project.id);
        }

        // 检查事件ID重复
        let mut event_ids = std::collections::HashSet::new();
        for event in &app_data.events {
            if event_ids.contains(&event.id) {
                issues.push(format!("事件ID重复: {}", event.id));
            }
            event_ids.insert(event.id);

            // 检查事件引用的项目是否存在
            if let crate::models::EventType::ProjectRelated(project_id) = &event.event_type {
                if !project_ids.contains(project_id) {
                    issues.push(format!(
                        "事件引用的项目不存在: 事件ID {}, 项目ID {}",
                        event.id, project_id
                    ));
                }
            }
        }

        // 检查时间记录ID重复
        let mut record_ids = std::collections::HashSet::new();
        for record in &app_data.time_records {
            if record_ids.contains(&record.id) {
                issues.push(format!("时间记录ID重复: {}", record.id));
            }
            record_ids.insert(record.id);

            // 检查时间记录引用的事件是否存在
            if !event_ids.contains(&record.event_id) {
                issues.push(format!(
                    "时间记录引用的事件不存在: 记录ID {}, 事件ID {}",
                    record.id, record.event_id
                ));
            }

            // 检查时间记录引用的项目是否存在
            if let Some(project_id) = record.project_id {
                if !project_ids.contains(&project_id) {
                    issues.push(format!(
                        "时间记录引用的项目不存在: 记录ID {}, 项目ID {}",
                        record.id, project_id
                    ));
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // 注意：tempfile crate需要添加到依赖中，这里暂时注释掉测试
    // use tempfile::tempdir;

    #[test]
    fn test_storage_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_string_lossy().to_string();

        let storage = Storage::new(data_dir.clone());
        assert_eq!(storage.data_dir, data_dir);
    }

    #[test]
    fn test_save_and_load_data() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_string_lossy().to_string();

        let storage = Storage::new(data_dir);
        let mut project_manager = ProjectManager::new();
        let mut event_manager = EventManager::new();

        // 添加测试数据
        let project_id = project_manager.add_project("测试项目".to_string(), None);
        project_manager.switch_to_project(project_id).unwrap();

        let _event_id =
            event_manager.add_project_event("测试事件".to_string(), None, project_id, None);

        // 保存数据
        storage.save_data(&project_manager, &event_manager).unwrap();

        // 加载数据
        let loaded_data = storage.load_data().unwrap();

        assert_eq!(loaded_data.projects.len(), 1);
        assert_eq!(loaded_data.events.len(), 1);
        assert_eq!(loaded_data.projects[0].name, "测试项目");
        assert_eq!(loaded_data.events[0].title, "测试事件");
    }

    #[test]
    fn test_backup_and_restore() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_string_lossy().to_string();

        let storage = Storage::new(data_dir);
        let mut project_manager = ProjectManager::new();
        let event_manager = EventManager::new();

        // 添加测试数据
        project_manager.add_project("测试项目".to_string(), None);

        // 创建备份
        let backup_path = storage
            .create_backup(&project_manager, &event_manager)
            .unwrap();
        assert!(Path::new(&backup_path).exists());

        // 从备份恢复
        let restored_data = storage.restore_from_backup(&backup_path).unwrap();
        assert_eq!(restored_data.projects.len(), 1);
        assert_eq!(restored_data.projects[0].name, "测试项目");
    }

    #[test]
    fn test_data_integrity_check() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_string_lossy().to_string();

        let storage = Storage::new(data_dir);
        let mut app_data = AppData::new();

        // 添加正常数据
        app_data
            .projects
            .push(Project::new("测试项目".to_string(), None));
        let project_id = app_data.projects[0].id;

        app_data.events.push(Event::new(
            "测试事件".to_string(),
            None,
            crate::models::EventType::ProjectRelated(project_id),
            chrono::Utc::now(),
        ));

        // 检查完整性（应该没有问题）
        let issues = storage.check_data_integrity(&app_data);
        assert!(issues.is_empty());

        // 添加重复ID
        app_data
            .projects
            .push(Project::new("重复项目".to_string(), None));
        app_data.projects[1].id = project_id; // 设置重复ID

        // 再次检查完整性（应该发现问题）
        let issues = storage.check_data_integrity(&app_data);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("项目ID重复")));
    }
}
