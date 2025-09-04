# Rust 项目管理系统

一个基于 Rust 的可视化项目管理系统，支持项目跟踪、时间记录和报表生成。

## 功能特性

- 📋 **项目管理**：创建、删除和切换项目
- 📅 **事件管理**：添加项目内和项目外事件，设置开始和结束时间
- ⏱️ **时间跟踪**：自动计算项目内和项目外时间
- 📊 **报表生成**：生成每周报表，显示工作效率和时间分布
- 💾 **数据持久化**：自动保存和加载数据，支持备份和恢复
- 🖥️ **可视化界面**：基于终端的用户界面（TUI）

## 系统要求

- Rust 1.70+
- 支持的操作系统：Linux、macOS、Windows

## 安装

1. 克隆仓库
```bash
git clone <repository-url>
cd project_manager
```

2. 构建项目
```bash
cargo build --release
```

## 使用方法

运行程序：
```bash
cargo run
```

### 主要操作

- `Q` - 退出程序
- `H` - 显示帮助
- `A` - 添加新项目
- `E` - 查看事件列表
- `R` - 查看报表
- `↑/↓` - 选择项目/事件
- `Enter` - 确认选择/完成事件
- `Esc` - 返回上一级

### 项目管理

1. 添加项目：在项目列表界面按 `A`，输入项目名称
2. 切换项目：使用方向键选择项目，按 `Enter` 切换
3. 查看事件：选择项目后按 `E` 查看相关事件

### 事件管理

1. 添加项目事件：在事件列表界面按 `P`
2. 添加项目外事件：在事件列表界面按 `A`
3. 完成事件：选择事件后按 `Enter`

### 报表查看

在任意界面按 `R` 查看每周报表，包括：
- 项目内时间统计
- 项目外时间统计
- 工作效率分析

## 数据存储

- 数据文件位置：`./data/app_data.json`
- 备份文件位置：`./data/backups/`
- 支持自动备份和数据完整性检查

## 开发

### 运行测试

```bash
cargo test
```

### 代码格式化

```bash
cargo fmt
```

### 检查代码

```bash
cargo clippy
```

## 项目结构

```
project_manager/
├── src/
│   ├── main.rs              # 程序入口
│   ├── models.rs            # 数据模型定义
│   ├── project_manager.rs   # 项目管理逻辑
│   ├── event_manager.rs     # 事件管理逻辑
│   ├── time_calculator.rs   # 时间计算功能
│   ├── report_generator.rs  # 报表生成
│   ├── storage.rs           # 数据持久化
│   └── ui.rs                # 用户界面
├── data/                    # 数据存储目录
├── Cargo.toml              # 项目配置
└── README.md               # 项目说明
```

## 技术栈

- **Rust**：主要编程语言
- **Ratatui**：终端用户界面库
- **Crossterm**：终端处理库
- **Serde**：JSON 序列化/反序列化
- **Chrono**：日期时间处理
- **UUID**：唯一标识符生成

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可证

MIT License