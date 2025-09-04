mod event_manager;
mod models;
mod project_manager;
mod report_generator;
mod storage;
mod time_calculator;
mod ui;

use eframe::egui;
use storage::Storage;
use ui::App;

fn main() -> eframe::Result<()> {
    println!("启动项目管理系统GUI界面...");

    // 初始化存储
    let storage = Storage::new("./data".to_string());

    // 尝试加载保存的数据
    let app = match storage.load_data() {
        Ok(data) => {
            println!("已加载保存的数据");
            App::from_data(data)
        }
        Err(e) => {
            println!("无法加载数据，使用新的应用状态: {}", e);
            App::new()
        }
    };

    // 运行egui应用
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "项目管理系统",
        native_options,
        Box::new(|_cc| {
            Box::new(EguiApp::new(app, storage))
        }),
    )
}

struct EguiApp {
    app: App,
    storage: Storage,
}

impl EguiApp {
    fn new(app: App, storage: Storage) -> Self {
        Self { app, storage }
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.update(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // 保存数据
        if let Err(e) = self.storage.save_data(&self.app.project_manager, &self.app.event_manager) {
            eprintln!("保存数据失败: {}", e);
        } else {
            println!("数据已保存");
        }
    }
}
