use crate::event_manager::EventManager;
use crate::models::{Event, EventType, Project, TimeRecord};
use crate::project_manager::ProjectManager;
use crate::report_generator::ReportGenerator;
use crate::storage;
use crate::time_calculator::TimeCalculator;
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table},
    Frame, Terminal,
};
use std::collections::HashMap;
use std::io;
use uuid::Uuid;

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
    pub project_list_state: ListState,
    pub event_list_state: ListState,
    pub input: String,
    pub input_cursor: usize,
    pub message: String,
    pub selected_project_id: Option<Uuid>,
    pub event_type_selection: bool, // true for project event, false for non-project event
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            project_manager: ProjectManager::new(),
            event_manager: EventManager::new(),
            mode: AppMode::ProjectList,
            project_list_state: ListState::default(),
            event_list_state: ListState::default(),
            input: String::new(),
            input_cursor: 0,
            message: String::new(),
            selected_project_id: None,
            event_type_selection: false,
        };

        app.project_list_state.select(Some(0));
        app.event_list_state.select(Some(0));

        app
    }

    pub fn from_data(data: storage::AppData) -> Self {
        let mut app = Self {
            project_manager: ProjectManager::new(),
            event_manager: EventManager::new(),
            mode: AppMode::ProjectList,
            project_list_state: ListState::default(),
            event_list_state: ListState::default(),
            input: String::new(),
            input_cursor: 0,
            message: "已加载保存的数据".to_string(),
            selected_project_id: None,
            event_type_selection: false,
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

        // 恢复时间记录数据
        for _record in data.time_records {
            // 注意：这里需要通过EventManager的公共方法来添加时间记录
            // 由于时间记录通常是通过事件完成时自动创建的，这里暂时跳过
        }

        app.project_list_state.select(Some(0));
        app.event_list_state.select(Some(0));

        app
    }

    pub fn get_projects(&self) -> Vec<&Project> {
        self.project_manager.get_all_projects()
    }

    pub fn get_events(&self) -> Vec<&Event> {
        self.event_manager.get_all_events()
    }

    pub fn get_current_project(&self) -> Option<&Project> {
        self.project_manager.get_current_project()
    }

    pub fn add_project(&mut self, name: String, description: Option<String>) {
        let project_id = self.project_manager.add_project(name, description);
        self.message = format!("项目添加成功: ID {}", project_id);
    }

    pub fn switch_to_project(&mut self, project_id: Uuid) {
        if let Err(e) = self.project_manager.switch_to_project(project_id) {
            self.message = format!("切换项目失败: {}", e);
        } else {
            self.message = "项目切换成功".to_string();
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
}

#[derive(Default)]
pub struct ListState {
    selected: Option<usize>,
}

impl ListState {
    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }
}

pub fn run_app(mut app: &mut App) -> io::Result<()> {
    // 设置终端
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let CEvent::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Char('h') => {
                    app.mode = AppMode::Help;
                }
                KeyCode::Esc => {
                    app.mode = AppMode::ProjectList;
                    app.input.clear();
                    app.input_cursor = 0;
                }
                KeyCode::Char('r') => {
                    app.mode = AppMode::Reports;
                }
                _ => {
                    handle_input(&mut app, key);
                }
            }
        }
    }

    // 恢复终端
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

fn handle_input(app: &mut App, key: KeyEvent) {
    match app.mode {
        AppMode::ProjectList => match key.code {
            KeyCode::Down => {
                let projects = app.get_projects();
                if let Some(selected) = app.project_list_state.selected() {
                    let new_selected = if selected >= projects.len().saturating_sub(1) {
                        0
                    } else {
                        selected + 1
                    };
                    app.project_list_state.select(Some(new_selected));
                }
            }
            KeyCode::Up => {
                let projects = app.get_projects();
                if let Some(selected) = app.project_list_state.selected() {
                    let new_selected = if selected == 0 {
                        projects.len().saturating_sub(1)
                    } else {
                        selected - 1
                    };
                    app.project_list_state.select(Some(new_selected));
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = app.project_list_state.selected() {
                    let projects = app.get_projects();
                    if selected < projects.len() {
                        let project_id = projects[selected].id;
                        app.switch_to_project(project_id);
                    }
                }
            }
            KeyCode::Char('a') => {
                app.mode = AppMode::AddProject;
                app.input.clear();
                app.input_cursor = 0;
            }
            KeyCode::Char('e') => {
                app.mode = AppMode::EventList;
                app.event_list_state.select(Some(0));
            }
            _ => {}
        },
        AppMode::EventList => match key.code {
            KeyCode::Down => {
                let events = app.get_events();
                if let Some(selected) = app.event_list_state.selected() {
                    let new_selected = if selected >= events.len().saturating_sub(1) {
                        0
                    } else {
                        selected + 1
                    };
                    app.event_list_state.select(Some(new_selected));
                }
            }
            KeyCode::Up => {
                let events = app.get_events();
                if let Some(selected) = app.event_list_state.selected() {
                    let new_selected = if selected == 0 {
                        events.len().saturating_sub(1)
                    } else {
                        selected - 1
                    };
                    app.event_list_state.select(Some(new_selected));
                }
            }
            KeyCode::Enter => {
                if let Some(selected) = app.event_list_state.selected() {
                    let events = app.get_events();
                    if selected < events.len() {
                        let event_id = events[selected].id;
                        if !events[selected].is_completed() {
                            app.complete_event(event_id);
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                app.mode = AppMode::AddEvent;
                app.input.clear();
                app.input_cursor = 0;
                app.event_type_selection = false;
            }
            KeyCode::Char('p') => {
                app.mode = AppMode::AddEvent;
                app.input.clear();
                app.input_cursor = 0;
                app.event_type_selection = true;
            }
            KeyCode::Esc => {
                app.mode = AppMode::ProjectList;
            }
            _ => {}
        },
        AppMode::AddProject => match key.code {
            KeyCode::Enter => {
                if !app.input.is_empty() {
                    app.add_project(app.input.clone(), None);
                    app.input.clear();
                    app.input_cursor = 0;
                    app.mode = AppMode::ProjectList;
                }
            }
            KeyCode::Char(c) => {
                app.input.insert(app.input_cursor, c);
                app.input_cursor += 1;
            }
            KeyCode::Backspace => {
                if app.input_cursor > 0 {
                    app.input.remove(app.input_cursor - 1);
                    app.input_cursor -= 1;
                }
            }
            KeyCode::Left => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if app.input_cursor < app.input.len() {
                    app.input_cursor += 1;
                }
            }
            KeyCode::Esc => {
                app.input.clear();
                app.input_cursor = 0;
                app.mode = AppMode::ProjectList;
            }
            _ => {}
        },
        AppMode::AddEvent => match key.code {
            KeyCode::Enter => {
                if !app.input.is_empty() {
                    app.add_event(app.input.clone(), None, app.event_type_selection);
                    app.input.clear();
                    app.input_cursor = 0;
                    app.mode = AppMode::EventList;
                }
            }
            KeyCode::Char(c) => {
                app.input.insert(app.input_cursor, c);
                app.input_cursor += 1;
            }
            KeyCode::Backspace => {
                if app.input_cursor > 0 {
                    app.input.remove(app.input_cursor - 1);
                    app.input_cursor -= 1;
                }
            }
            KeyCode::Left => {
                if app.input_cursor > 0 {
                    app.input_cursor -= 1;
                }
            }
            KeyCode::Right => {
                if app.input_cursor < app.input.len() {
                    app.input_cursor += 1;
                }
            }
            KeyCode::Esc => {
                app.input.clear();
                app.input_cursor = 0;
                app.mode = AppMode::EventList;
            }
            _ => {}
        },
        AppMode::Reports => match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::ProjectList;
            }
            _ => {}
        },
        AppMode::Help => match key.code {
            KeyCode::Esc => {
                app.mode = AppMode::ProjectList;
            }
            _ => {}
        },
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.size());

    // 标题栏
    let title = render_title_bar(app);
    f.render_widget(title, chunks[0]);

    // 主要内容区域
    match app.mode {
        AppMode::ProjectList => render_project_list(f, app, chunks[1]),
        AppMode::EventList => render_event_list(f, app, chunks[1]),
        AppMode::AddProject => render_add_project(f, app, chunks[1]),
        AppMode::AddEvent => render_add_event(f, app, chunks[1]),
        AppMode::Reports => render_reports(f, app, chunks[1]),
        AppMode::Help => render_help(f, app, chunks[1]),
    }

    // 状态栏
    let status = render_status_bar(app);
    f.render_widget(status, chunks[2]);
}

fn render_title_bar(app: &App) -> Paragraph {
    let current_project = app
        .get_current_project()
        .map(|p| format!("当前项目: {}", p.name))
        .unwrap_or_else(|| "没有当前项目".to_string());

    let title = Line::from(vec![
        Span::styled(
            "项目管理系统",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::raw(current_project),
    ]);

    Paragraph::new(title)
        .style(Style::default().bg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Center)
}

fn render_status_bar(app: &App) -> Paragraph {
    let mode_text = match app.mode {
        AppMode::ProjectList => "项目列表",
        AppMode::EventList => "事件列表",
        AppMode::AddProject => "添加项目",
        AppMode::AddEvent => "添加事件",
        AppMode::Reports => "报表",
        AppMode::Help => "帮助",
    };

    let status = Line::from(vec![
        Span::styled(mode_text, Style::default().fg(Color::Cyan)),
        Span::raw(" | "),
        Span::raw(&app.message),
        Span::raw(" | "),
        Span::raw("按 H 查看帮助，Q 退出"),
    ]);

    Paragraph::new(status)
        .style(Style::default().bg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Left)
}

fn render_project_list(f: &mut Frame, app: &App, area: Rect) {
    let projects = app.get_projects();

    let items: Vec<ListItem> = projects
        .iter()
        .map(|p| {
            let status = if p.is_active { "[当前]" } else { "" };
            ListItem::new(format!(
                "{} {} {}",
                status,
                p.name,
                p.description.as_deref().unwrap_or("")
            ))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("项目列表").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default()
            .with_selected(app.project_list_state.selected()),
    );
}

fn render_event_list(f: &mut Frame, app: &App, area: Rect) {
    let events = app.get_events();

    let items: Vec<ListItem> = events
        .iter()
        .map(|e| {
            let status = if e.is_completed() {
                "[已完成]"
            } else {
                "[进行中]"
            };
            let project_info = match &e.event_type {
                crate::models::EventType::ProjectRelated(id) => {
                    if let Some(project) = app.project_manager.get_project(*id) {
                        format!("(项目: {})", project.name)
                    } else {
                        "(未知项目)".to_string()
                    }
                }
                crate::models::EventType::NonProject => "(项目外)".to_string(),
            };
            ListItem::new(format!(
                "{} {} {} {}",
                status,
                e.title,
                project_info,
                e.description.as_deref().unwrap_or("")
            ))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("事件列表").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(
        list,
        area,
        &mut ratatui::widgets::ListState::default().with_selected(app.event_list_state.selected()),
    );
}

fn render_add_project(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new("添加新项目")
        .block(Block::default().title("添加项目").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title("项目名称").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);

    let help = Paragraph::new("输入项目名称后按 Enter 确认，按 Esc 取消")
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(title, chunks[0]);
    f.render_widget(input, chunks[1]);
    f.render_widget(help, chunks[2]);
}

fn render_add_event(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let event_type = if app.event_type_selection {
        "项目事件"
    } else {
        "项目外事件"
    };
    let title = Paragraph::new(format!("添加{}", event_type))
        .block(Block::default().title("添加事件").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    let input = Paragraph::new(app.input.as_str())
        .block(Block::default().title("事件标题").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);

    let help = Paragraph::new("输入事件标题后按 Enter 确认，按 Esc 取消")
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(title, chunks[0]);
    f.render_widget(input, chunks[1]);
    f.render_widget(help, chunks[2]);
}

fn render_reports(f: &mut Frame, app: &App, area: Rect) {
    let report_text = app.get_weekly_report();
    let text = Text::from(report_text);

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("每周报表").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_help(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = r#"
项目管理系统 - 帮助

全局快捷键:
  Q - 退出程序
  H - 显示帮助
  Esc - 返回上一级
  R - 查看报表

项目列表:
  ↑/↓ - 选择项目
  Enter - 切换到选中项目
  A - 添加新项目
  E - 查看事件列表

事件列表:
  ↑/↓ - 选择事件
  Enter - 完成选中事件
  A - 添加项目外事件
  P - 添加项目事件

添加项目/事件:
  输入文本
  Enter - 确认
  Esc - 取消
  Backspace - 删除字符
  ←/→ - 移动光标
"#;

    let text = Text::from(help_text);

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("帮助").borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Left);

    f.render_widget(paragraph, area);
}
