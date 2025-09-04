use crate::models::{Event, EventType, Project};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

pub struct ProjectManager {
    projects: HashMap<Uuid, Project>,
    current_project_id: Option<Uuid>,
}

impl ProjectManager {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            current_project_id: None,
        }
    }

    /// 添加新项目
    pub fn add_project(&mut self, name: String, description: Option<String>) -> Uuid {
        let mut project = Project::new(name, description);
        let project_id = project.id;

        // 如果这是第一个项目，自动设置为当前项目
        if self.projects.is_empty() {
            project.set_active(true);
            self.current_project_id = Some(project_id);
        }

        self.projects.insert(project_id, project);
        project_id
    }

    /// 删除项目
    pub fn delete_project(&mut self, project_id: Uuid) -> Result<(), String> {
        if !self.projects.contains_key(&project_id) {
            return Err("项目不存在".to_string());
        }

        // 如果删除的是当前项目，清除当前项目ID
        if self.current_project_id == Some(project_id) {
            self.current_project_id = None;
        }

        self.projects.remove(&project_id);
        Ok(())
    }

    /// 切换当前项目
    pub fn switch_to_project(&mut self, project_id: Uuid) -> Result<(), String> {
        if !self.projects.contains_key(&project_id) {
            return Err("项目不存在".to_string());
        }

        // 取消所有项目的激活状态
        for project in self.projects.values_mut() {
            project.set_active(false);
        }

        // 激活选中的项目
        if let Some(project) = self.projects.get_mut(&project_id) {
            project.set_active(true);
            self.current_project_id = Some(project_id);
        }

        Ok(())
    }

    /// 获取当前项目
    pub fn get_current_project(&self) -> Option<&Project> {
        self.current_project_id
            .and_then(|id| self.projects.get(&id))
    }

    /// 获取所有项目
    pub fn get_all_projects(&self) -> Vec<&Project> {
        self.projects.values().collect()
    }

    /// 根据ID获取项目
    pub fn get_project(&self, project_id: Uuid) -> Option<&Project> {
        self.projects.get(&project_id)
    }

    /// 更新项目信息
    pub fn update_project(
        &mut self,
        project_id: Uuid,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        if let Some(project) = self.projects.get_mut(&project_id) {
            if let Some(name) = name {
                project.name = name;
            }
            if description.is_some() {
                project.description = description;
            }
            Ok(())
        } else {
            Err("项目不存在".to_string())
        }
    }

    /// 获取项目数量
    pub fn get_project_count(&self) -> usize {
        self.projects.len()
    }

    /// 检查项目是否存在
    pub fn project_exists(&self, project_id: Uuid) -> bool {
        self.projects.contains_key(&project_id)
    }

    /// 获取项目名称列表
    pub fn get_project_names(&self) -> Vec<String> {
        self.projects.values().map(|p| p.name.clone()).collect()
    }

    /// 创建项目相关事件
    pub fn create_project_event(
        &self,
        title: String,
        description: Option<String>,
    ) -> Result<Event, String> {
        if let Some(current_project_id) = self.current_project_id {
            let event = Event::new(
                title,
                description,
                EventType::ProjectRelated(current_project_id),
                Utc::now(),
            );
            Ok(event)
        } else {
            Err("没有当前活动项目".to_string())
        }
    }

    /// 创建项目外事件
    pub fn create_non_project_event(&self, title: String, description: Option<String>) -> Event {
        Event::new(title, description, EventType::NonProject, Utc::now())
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_project() {
        let mut manager = ProjectManager::new();
        let project_id = manager.add_project("测试项目".to_string(), Some("测试描述".to_string()));

        assert_eq!(manager.get_project_count(), 1);
        assert!(manager.project_exists(project_id));

        let project = manager.get_project(project_id).unwrap();
        assert_eq!(project.name, "测试项目");
        assert_eq!(project.description, Some("测试描述".to_string()));
        assert!(project.is_active);
    }

    #[test]
    fn test_switch_project() {
        let mut manager = ProjectManager::new();
        let id1 = manager.add_project("项目1".to_string(), None);
        let id2 = manager.add_project("项目2".to_string(), None);

        // 第一个项目应该是当前项目
        assert_eq!(manager.get_current_project().unwrap().id, id1);

        // 切换到第二个项目
        manager.switch_to_project(id2).unwrap();
        assert_eq!(manager.get_current_project().unwrap().id, id2);

        // 第一个项目应该不再是激活状态
        assert!(!manager.get_project(id1).unwrap().is_active);
        assert!(manager.get_project(id2).unwrap().is_active);
    }

    #[test]
    fn test_delete_project() {
        let mut manager = ProjectManager::new();
        let id1 = manager.add_project("项目1".to_string(), None);
        let id2 = manager.add_project("项目2".to_string(), None);

        manager.switch_to_project(id2).unwrap();
        manager.delete_project(id1).unwrap();

        assert_eq!(manager.get_project_count(), 1);
        assert!(!manager.project_exists(id1));
        assert!(manager.project_exists(id2));
    }
}
