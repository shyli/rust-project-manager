mod event_manager;
mod models;
mod project_manager;
mod report_generator;
mod storage;
mod time_calculator;
mod ui;

use storage::Storage;
use ui::{run_app, App};

fn main() -> std::io::Result<()> {
    println!("启动项目管理系统TUI界面...");
    println!("按 H 键查看帮助，按 Q 键退出程序");

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

    // 运行应用
    let mut app = app;
    let result = run_app(&mut app);

    // 保存数据
    if let Err(e) = storage.save_data(&app.project_manager, &app.event_manager) {
        eprintln!("保存数据失败: {}", e);
    } else {
        println!("数据已保存");
    }

    println!("应用已退出");

    result
}
