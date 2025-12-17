mod screenshot;

use std::fs::OpenOptions;
use std::io::Write;
use log::{LevelFilter, info};
use env_logger::Builder;
use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志记录器
    let log_path = get_log_path();
    info!("尝试创建日志文件: {:?}", log_path);
    
    if let Ok(log_file) = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path) {
        info!("成功创建日志文件: {:?}", log_path);
        let mut builder = Builder::new();
        builder
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter_level(LevelFilter::Info)
            .format(|buf, record| {
                writeln!(buf, "{} [{}] - {}", 
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();
    } else {
        // 如果无法创建日志文件，则使用默认日志记录器
        env_logger::builder()
            .filter_level(LevelFilter::Info)
            .init();
        eprintln!("警告: 无法创建日志文件 {:?}，将使用标准输出记录日志", log_path);
    }
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, screenshot::capture_screen])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 获取日志文件路径
/// 在不同平台上将日志文件存储在合适的位置
fn get_log_path() -> PathBuf {
    // 尝试获取应用配置目录
    if let Some(config_dir) = dirs::config_dir() {
        let app_log_dir = config_dir.join("screenshot-app");
        // 确保目录存在
        if !app_log_dir.exists() {
            let _ = std::fs::create_dir_all(&app_log_dir);
        }
        return app_log_dir.join("screenshot.log");
    }
    
    // 如果无法获取配置目录，则使用可执行文件所在目录
    if let Ok(current_dir) = std::env::current_exe() {
        if let Some(parent) = current_dir.parent() {
            return parent.join("screenshot.log");
        }
    }
    
    // 最后回退到当前工作目录
    PathBuf::from("screenshot.log")
}