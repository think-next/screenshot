mod screenshot;

use env_logger::Builder;
use log::{error, info, LevelFilter};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSApplication, NSApplicationPresentationOptions};
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
use objc::runtime::Object;
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 在主线程上设置 macOS 的全屏演示模式
#[cfg(target_os = "macos")]
fn set_presentation_mode_on_main_thread(app: id, options: NSApplicationPresentationOptions) -> i32 {
    info!("在主线程上设置 presentation options");
    unsafe {
        let result: i32 = msg_send![app, setPresentationOptions: options];
        info!("主线程 setPresentationOptions_ 返回值: {:?}", result);
        result
    }
}

/// 设置 macOS 的全屏演示模式，自动隐藏菜单栏和 Dock
#[cfg(target_os = "macos")]
#[tauri::command]
async fn set_macos_presentation_mode(fullscreen: bool) -> Result<(), String> {
    info!(
        "set_macos_presentation_mode 被调用，参数: fullscreen={}",
        fullscreen
    );

    unsafe {
        let app = NSApplication::sharedApplication(nil);
        info!("成功获取 NSApplication 共享实例");

        // 激活应用程序，确保它处于活动状态
        info!("激活应用程序");
        app.activateIgnoringOtherApps_(cocoa::base::YES);

        if fullscreen {
            info!("正在设置全屏演示模式（隐藏 Dock 和菜单栏）");

            let options = NSApplicationPresentationOptions::NSApplicationPresentationHideDock
                    | NSApplicationPresentationOptions::NSApplicationPresentationHideMenuBar
                    | NSApplicationPresentationOptions::NSApplicationPresentationDisableHideApplication
                    | NSApplicationPresentationOptions::NSApplicationPresentationDisableProcessSwitching;

            info!("方法1: 使用 setPresentationOptions_（在主线程上）");
            info!("Presentation Options 值: {:?}", options);

            // 在主线程上执行
            let result = set_presentation_mode_on_main_thread(app, options);
            info!("setPresentationOptions_ 返回值: {:?}", result);

            // 检查返回值是否为错误码（负数表示错误）
            if result < 0 {
                error!(
                    "setPresentationOptions_ 返回错误码: {} (OSStatus错误)",
                    result
                );
                error!("可能的原因：应用程序状态不正确或某些选项不被支持");

                // 方法2：尝试简化选项，只使用基本的隐藏选项
                info!("方法2: 尝试使用简化的 presentation options...");
                let simple_options =
                    NSApplicationPresentationOptions::NSApplicationPresentationHideDock
                        | NSApplicationPresentationOptions::NSApplicationPresentationHideMenuBar;
                info!("简化后的 Options 值: {:?}", simple_options);

                let simple_result = set_presentation_mode_on_main_thread(app, simple_options);
                info!("简化选项的返回值: {:?}", simple_result);

                if simple_result < 0 {
                    error!("简化选项也失败，返回错误码: {}", simple_result);

                    // 方法3：尝试只隐藏 Dock
                    info!("方法3: 尝试只隐藏 Dock...");
                    let dock_only =
                        NSApplicationPresentationOptions::NSApplicationPresentationHideDock;

                    let dock_result = set_presentation_mode_on_main_thread(app, dock_only);
                    info!("只隐藏 Dock 的返回值: {:?}", dock_result);

                    if dock_result < 0 {
                        error!("所有方法都失败了");

                        // 方法4：尝试直接设置 presentationOptions 属性（不使用方法）
                        info!("方法4: 尝试直接设置 presentationOptions 属性...");
                        let _: () = msg_send![app, setPresentationOptions: options];
                        info!("已尝试直接设置 presentationOptions 属性");
                    }
                }
            }

            info!("全屏演示模式设置成功");
        } else {
            info!("正在恢复默认演示模式");
            let options = NSApplicationPresentationOptions::NSApplicationPresentationDefault;

            info!("Presentation Options 值: {:?}", options);

            let result = set_presentation_mode_on_main_thread(app, options);
            info!("setPresentationOptions_ 返回值: {:?}", result);

            // 检查返回值是否为错误码
            if result < 0 {
                error!("恢复默认演示模式失败，错误码: {}", result);
                error!("警告: 可能无法完全恢复演示模式，但程序会继续运行");

                // 尝试直接设置属性
                info!("尝试直接设置 presentationOptions 属性...");
                let _: () = msg_send![app, setPresentationOptions: options];
                info!("已尝试直接设置 presentationOptions 属性");
            }

            info!("默认演示模式设置成功");
        }
    }

    info!("set_macos_presentation_mode 执行完成");
    Ok(())
}

/// 在非 macOS 平台上的空实现
#[cfg(not(target_os = "macos"))]
#[tauri::command]
async fn set_macos_presentation_mode(_fullscreen: bool) -> Result<(), String> {
    info!(
        "set_macos_presentation_mode 在非 macOS 平台上被调用，参数: fullscreen={}",
        _fullscreen
    );
    info!("当前平台不是 macOS，方法为空实现");
    Ok(())
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
        .open(&log_path)
    {
        info!("成功创建日志文件: {:?}", log_path);
        let mut builder = Builder::new();
        builder
            .target(env_logger::Target::Pipe(Box::new(log_file)))
            .filter_level(LevelFilter::Info)
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();
    } else {
        // 如果无法创建日志文件，则使用默认日志记录器
        env_logger::builder().filter_level(LevelFilter::Info).init();
        eprintln!(
            "警告: 无法创建日志文件 {:?}，将使用标准输出记录日志",
            log_path
        );
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            screenshot::capture_screen,
            screenshot::capture_region,
            screenshot::save_screenshot,
            screenshot::capture_and_save_region,
            set_macos_presentation_mode
        ])
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
