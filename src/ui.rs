use crate::event_manager::EventManager;
use crate::models::{Event, EventType, Project, TimeRecord};
use crate::project_manager::ProjectManager;
use crate::report_generator::ReportGenerator;
use crate::storage;
use chrono::Utc;
use eframe::egui;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    ProjectList,
    EventList,
    AddProject,
    AddEvent,
    Reports,
    Help,
}

pub struct App {
    pub project_manager: ProjectManager,
    pub event_manager: EventManager,
    pub mode: AppMode,
    pub selected_project_index: usize,
    pub selected_event_index: usize,
    pub input: String,
    pub message: String,
    pub selected_project_id: Option<Uuid>,
    pub event_type_selection: bool, // true for project event, false for non-project event
    pub new_project_name: String,
    pub new_project_description: String,
    pub new_event_title: String,
    pub new_event_description: String,
    pub show_completed_events: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            project_manager: ProjectManager::new(),
            event_manager: EventManager::new(),
            mode: AppMode::ProjectList,
            selected_project_index: 0,
            selected_event_index: 0,
            input: String::new(),
            message: "欢迎使用项目管理系统".to_string(),
            selected_project_id: None,
            event_type_selection: false,
            new_project_name: String::new(),
            new_project_description: String::new(),
            new_event_title: String::new(),
            new_event_description: String::new(),
            show_completed_events: false,
        }
    }

    pub fn from_data(data: storage::AppData) -> Self {
        let mut app = Self {
            project_manager: ProjectManager::new(),
            event_manager: EventManager::new(),
            mode: AppMode::ProjectList,
            selected_project_index: 0,
            selected_event_index: 0,
            input: String::new(),
            message: "已加载保存的数据".to_string(),
            selected_project_id: None,
            event_type_selection: false,
            new_project_name: String::new(),
            new_project_description: String::new(),
            new_event_title: String::new(),
            new_event_description: String::new(),
            show_completed_events: false,
        };

        // 恢复项目数据
        for project in data.projects {
            let project_id = app
                .project_manager
                .add_project(project.name, project.description);
            if project.is_active {
                app.project_manager.switch_to_project(project_id).unwrap();
            }
        }

        // 恢复事件数据
        for event in data.events {
            match event.event_type {
                EventType::ProjectRelated(project_id) => {
                    app.event_manager.add_project_event(
                        event.title,
                        event.description,
                        project_id,
                        Some(event.start_time),
                    );
                }
                EventType::NonProject => {
                    app.event_manager.add_non_project_event(
                        event.title,
                        event.description,
                        Some(event.start_time),
                    );
                }
            }
        }

        app
    }

    pub fn get_projects(&self) -> Vec<&Project> {
        self.project_manager.get_all_projects()
    }

    pub fn get_events(&self) -> Vec<&Event> {
        if self.show_completed_events {
            self.event_manager.get_all_events()
        } else {
            self.event_manager.get_active_events()
        }
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.project_manager.get_current_project()
    }

    pub fn add_project(&mut self, name: String, description: Option<String>) {
        let project_id = self.project_manager.add_project(name, description);
        self.message = format!("项目添加成功: ID {}", project_id);
        self.new_project_name.clear();
        self.new_project_description.clear();
    }

    pub fn switch_to_project(&mut self, project_id: Uuid) {
        if let Err(e) = self.project_manager.switch_to_project(project_id) {
            self.message = format!("切换项目失败: {}", e);
        } else {
            self.message = "项目切换成功".to_string();
            self.selected_project_id = Some(project_id);
        }
    }

    pub fn add_event(
        &mut self,
        title: String,
        description: Option<String>,
        is_project_event: bool,
    ) {
        if is_project_event {
            if let Some(current_project) = self.get_current_project() {
                let event_id = self.event_manager.add_project_event(
                    title,
                    description,
                    current_project.id,
                    None,
                );
                self.message = format!("项目事件添加成功: ID {}", event_id);
            } else {
                self.message = "没有当前活动项目，请先选择项目".to_string();
            }
        } else {
            let event_id = self
                .event_manager
                .add_non_project_event(title, description, None);
            self.message = format!("项目外事件添加成功: ID {}", event_id);
        }
        self.new_event_title.clear();
        self.new_event_description.clear();
    }

    pub fn complete_event(&mut self, event_id: Uuid) {
        if let Err(e) = self.event_manager.set_event_end_time(event_id, None) {
            self.message = format!("完成事件失败: {}", e);
        } else {
            self.message = "事件已完成".to_string();
        }
    }

    pub fn get_weekly_report(&self) -> String {
        let time_records = self.event_manager.get_all_time_records();
        let time_records_refs: Vec<&TimeRecord> = time_records.iter().map(|&r| r).collect();

        let mut project_names = HashMap::new();
        for project in self.get_projects() {
            project_names.insert(project.id, project.name.clone());
        }

        let now = Utc::now();
        let weekly_report =
            ReportGenerator::generate_weekly_report(&time_records_refs, &project_names, now);
        ReportGenerator::generate_report_summary(&weekly_report)
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("项目管理系统");
                
                if let Some(current_project) = self.get_current_project() {
                    ui.label(format!("当前项目: {}", current_project.name));
                } else {
                    ui.label("没有当前项目");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("帮助").clicked() {
                        self.mode = AppMode::Help;
                    }
                    if ui.button("报表").clicked() {
                        self.mode = AppMode::Reports;
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mode_text = match self.mode {
                    AppMode::ProjectList => "项目列表",
                    AppMode::EventList => "事件列表",
                    AppMode::AddProject => "添加项目",
                    AppMode::AddEvent => "添加事件",
                    AppMode::Reports => "报表",
                    AppMode::Help => "帮助",
                };
                ui.label(format!("模式: {}", mode_text));
                ui.label(&self.message);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.mode {
                AppMode::ProjectList => self.show_project_list(ui),
                AppMode::EventList => self.show_event_list(ui),
                AppMode::AddProject => self.show_add_project(ui),
                AppMode::AddEvent => self.show_add_event(ui),
                AppMode::Reports => self.show_reports(ui),
                AppMode::Help => self.show_help(ui),
            }
        });
    }

    fn show_project_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("添加项目").clicked() {
                self.mode = AppMode::AddProject;
            }
            if ui.button("查看事件").clicked() {
                self.mode = AppMode::EventList;
            }
        });

        ui.separator();

        let projects: Vec<_> = self.get_projects().into_iter().cloned().collect();
        if projects.is_empty() {
            ui.label("没有项目，点击\"添加项目\"创建新项目");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut project_to_switch = None;
                
                for (index, project) in projects.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let mut selected = self.selected_project_index == index;
                        if ui.checkbox(&mut selected, "").clicked() {
                            project_to_switch = Some((index, project.id));
                        }
                        
                        ui.vertical(|ui| {
                            ui.heading(&project.name);
                            if let Some(desc) = &project.description {
                                ui.label(desc);
                            }
                            ui.label(format!("创建时间: {}", project.created_at.format("%Y-%m-%d %H:%M")));
                            if project.is_active {
                                ui.label("（当前项目）");
                            }
                        });
                    });
                    ui.separator();
                }
                
                // 在闭包外切换项目
                if let Some((index, project_id)) = project_to_switch {
                    self.selected_project_index = index;
                    self.switch_to_project(project_id);
                }
            });
        }
    }

    fn show_event_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("返回项目").clicked() {
                self.mode = AppMode::ProjectList;
            }
            if ui.button("添加项目事件").clicked() {
                self.mode = AppMode::AddEvent;
                self.event_type_selection = true;
            }
            if ui.button("添加非项目事件").clicked() {
                self.mode = AppMode::AddEvent;
                self.event_type_selection = false;
            }
            
            ui.checkbox(&mut self.show_completed_events, "显示已完成事件");
        });

        ui.separator();

        let events: Vec<_> = self.get_events().into_iter().cloned().collect();
        if events.is_empty() {
            ui.label("没有事件");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut events_to_complete = Vec::new();
                
                for event in events.iter() {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.heading(&event.title);
                            if let Some(desc) = &event.description {
                                ui.label(desc);
                            }
                            
                            let event_type = match &event.event_type {
                                EventType::ProjectRelated(project_id) => {
                                    if let Some(project) = self.project_manager.get_project(*project_id) {
                                        format!("项目: {}", project.name)
                                    } else {
                                        "项目: (未知)".to_string()
                                    }
                                }
                                EventType::NonProject => "非项目事件".to_string(),
                            };
                            ui.label(event_type);
                            
                            ui.label(format!("开始时间: {}", event.start_time.format("%Y-%m-%d %H:%M")));
                            
                            if let Some(end_time) = event.end_time {
                                ui.label(format!("结束时间: {}", end_time.format("%Y-%m-%d %H:%M")));
                                if let Some(duration) = event.duration() {
                                    ui.label(format!("持续时间: {}分钟", duration.num_minutes()));
                                }
                            } else {
                                if ui.button("完成").clicked() {
                                    events_to_complete.push(event.id);
                                }
                            }
                        });
                    });
                    ui.separator();
                }
                
                // 在闭包外完成事件
                for event_id in events_to_complete {
                    self.complete_event(event_id);
                }
            });
        }
    }

    fn show_add_project(&mut self, ui: &mut egui::Ui) {
        ui.heading("添加新项目");
        
        ui.horizontal(|ui| {
            ui.label("项目名称:");
            ui.text_edit_singleline(&mut self.new_project_name);
        });
        
        ui.horizontal(|ui| {
            ui.label("项目描述:");
            ui.text_edit_multiline(&mut self.new_project_description);
        });
        
        ui.horizontal(|ui| {
            if ui.button("添加").clicked() {
                if !self.new_project_name.is_empty() {
                    self.add_project(
                        self.new_project_name.clone(),
                        if self.new_project_description.is_empty() {
                            None
                        } else {
                            Some(self.new_project_description.clone())
                        },
                    );
                    self.mode = AppMode::ProjectList;
                } else {
                    self.message = "项目名称不能为空".to_string();
                }
            }
            
            if ui.button("取消").clicked() {
                self.new_project_name.clear();
                self.new_project_description.clear();
                self.mode = AppMode::ProjectList;
            }
        });
    }

    fn show_add_event(&mut self, ui: &mut egui::Ui) {
        ui.heading("添加新事件");
        
        ui.horizontal(|ui| {
            ui.label("事件标题:");
            ui.text_edit_singleline(&mut self.new_event_title);
        });
        
        ui.horizontal(|ui| {
            ui.label("事件描述:");
            ui.text_edit_multiline(&mut self.new_event_description);
        });
        
        ui.horizontal(|ui| {
            ui.label("事件类型:");
            ui.radio_value(&mut self.event_type_selection, true, "项目事件");
            ui.radio_value(&mut self.event_type_selection, false, "非项目事件");
        });
        
        ui.horizontal(|ui| {
            if ui.button("添加").clicked() {
                if !self.new_event_title.is_empty() {
                    self.add_event(
                        self.new_event_title.clone(),
                        if self.new_event_description.is_empty() {
                            None
                        } else {
                            Some(self.new_event_description.clone())
                        },
                        self.event_type_selection,
                    );
                    self.mode = AppMode::EventList;
                } else {
                    self.message = "事件标题不能为空".to_string();
                }
            }
            
            if ui.button("取消").clicked() {
                self.new_event_title.clear();
                self.new_event_description.clear();
                self.mode = AppMode::EventList;
            }
        });
    }

    fn show_reports(&mut self, ui: &mut egui::Ui) {
        ui.heading("周报");
        
        if ui.button("返回").clicked() {
            self.mode = AppMode::ProjectList;
        }
        
        ui.separator();
        
        let report = self.get_weekly_report();
        ui.label(&report);
    }

    fn show_help(&mut self, ui: &mut egui::Ui) {
        ui.heading("帮助");
        
        if ui.button("返回").clicked() {
            self.mode = AppMode::ProjectList;
        }
        
        ui.separator();
        
        ui.label("项目管理系统使用说明：");
        ui.label("");
        ui.label("1. 项目列表：查看所有项目，选择当前项目");
        ui.label("2. 事件列表：查看所有事件，完成进行中的事件");
        ui.label("3. 添加项目：创建新项目");
        ui.label("4. 添加事件：创建新事件（项目事件或非项目事件）");
        ui.label("5. 报表：查看周报统计");
        ui.label("");
        ui.label("操作说明：");
        ui.label("- 点击项目名称切换当前项目");
        ui.label("- 点击\"完成\"按钮结束事件");
        ui.label("- 使用复选框选择项目或事件");
    }
}
