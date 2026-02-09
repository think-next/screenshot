use base64::{engine::general_purpose, Engine as _};
use image::imageops;
use log::{error, info};
use rayon::prelude::*;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use tauri::command;
use xcap::Monitor;

#[cfg(target_os = "macos")]
use arboard::Clipboard;

/// 截图并返回Base64编码的JPEG图像
#[command]
pub fn capture_screen() -> Result<String, String> {
    let total_start = Instant::now();
    info!("开始执行截图任务");

    // 添加日志文件位置提示
    let log_path_hint = if let Some(config_dir) = dirs::config_dir() {
        let log_path = config_dir.join("screenshot-app").join("screenshot.log");
        format!("日志文件位于: {:?}", log_path)
    } else {
        "日志文件位于应用根目录下的 screenshot.log".to_string()
    };
    info!("{}", log_path_hint);

    // 获取所有显示器
    let monitors_start = Instant::now();
    let monitors = Monitor::all().map_err(|e| {
        error!("获取显示器列表失败: {}", e);
        e.to_string()
    })?;
    info!("获取显示器列表完成, 耗时: {:?}", monitors_start.elapsed());

    // 获取主显示器（如果没有主显示器，则获取第一个显示器）
    let monitor_select_start = Instant::now();
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or("未找到可用显示器".to_string())?;
    info!("选择显示器完成, 耗时: {:?}", monitor_select_start.elapsed());

    // 截图
    let capture_start = Instant::now();
    let image = monitor.capture_image().map_err(|e| {
        error!("截图失败: {}", e);
        format!("截图失败: {}", e)
    })?;
    info!("截图操作完成, 耗时: {:?}", capture_start.elapsed());

    let width = image.width();
    let height = image.height();
    let rgba_data = image.as_raw();

    // 使用并行处理将RGBA转换为RGB，预分配缓冲区避免动态增长
    let convert_start = Instant::now();
    let pixel_count = (width * height) as usize;
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    rgb_data.resize(pixel_count * 3, 0);

    // 使用并行迭代器高效转换，直接写入预分配的缓冲区
    rgba_data
        .par_chunks_exact(4)
        .zip(rgb_data.par_chunks_exact_mut(3))
        .for_each(|(rgba, rgb)| {
            rgb[0] = rgba[0];
            rgb[1] = rgba[1];
            rgb[2] = rgba[2];
            // 跳过alpha通道
        });
    info!("RGBA到RGB转换完成, 耗时: {:?}", convert_start.elapsed());

    // 将图像编码为JPEG格式的字节（质量设置为85，平衡文件大小和图像质量）
    // JPEG不支持alpha通道，需要将RGBA转换为RGB
    let encode_start = Instant::now();
    let mut buffer: Vec<u8> = Vec::new();
    {
        let encoder = jpeg_encoder::Encoder::new(&mut buffer, 100);
        encoder
            .encode(
                &rgb_data,
                width as u16,
                height as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| {
                error!("图像JPEG编码失败: {}", e);
                format!("图像JPEG编码失败: {}", e)
            })?;
    }
    info!("图像JPEG编码完成, 耗时: {:?}", encode_start.elapsed());

    // 将字节转换为Base64字符串
    let base64_start = Instant::now();
    let base64_image = general_purpose::STANDARD.encode(&buffer);
    info!("Base64编码完成, 耗时: {:?}", base64_start.elapsed());

    info!("截图任务完成, 总耗时: {:?}", total_start.elapsed());
    Ok(base64_image)
}

/// 截图指定区域并返回Base64编码的JPEG图像
#[command]
pub fn capture_region(x: u32, y: u32, width: u32, height: u32) -> Result<String, String> {
    let total_start = Instant::now();
    info!(
        "开始执行区域截图任务: x={}, y={}, width={}, height={}",
        x, y, width, height
    );

    // 获取所有显示器
    let monitors_start = Instant::now();
    let monitors = Monitor::all().map_err(|e| {
        error!("获取显示器列表失败: {}", e);
        e.to_string()
    })?;
    info!("获取显示器列表完成, 耗时: {:?}", monitors_start.elapsed());

    // 获取主显示器（如果没有主显示器，则获取第一个显示器）
    let monitor_select_start = Instant::now();
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or("未找到可用显示器".to_string())?;
    info!("选择显示器完成, 耗时: {:?}", monitor_select_start.elapsed());

    // 截图整个屏幕
    let capture_start = Instant::now();
    let image = monitor.capture_image().map_err(|e| {
        error!("截图失败: {}", e);
        format!("截图失败: {}", e)
    })?;
    info!("截图操作完成, 耗时: {:?}", capture_start.elapsed());

    let screen_width = image.width();
    let screen_height = image.height();
    let rgba_data = image.as_raw();

    // 验证区域是否在屏幕范围内
    if x + width > screen_width || y + height > screen_height {
        error!(
            "指定区域超出屏幕范围: screen={}x{}, region=({}, {}, {}x{})",
            screen_width, screen_height, x, y, width, height
        );
        return Err("指定区域超出屏幕范围".to_string());
    }

    info!("开始裁剪指定区域...");

    // 将RGBA数据转换为image::RgbaImage
    let full_image = image::RgbaImage::from_raw(screen_width, screen_height, rgba_data.clone())
        .ok_or("无法创建图像".to_string())?;

    // 裁剪指定区域
    let crop_start = Instant::now();
    let cropped_image = imageops::crop_imm(&full_image, x, y, width, height).to_image();
    info!("区域裁剪完成, 耗时: {:?}", crop_start.elapsed());

    // 将裁剪后的图像转换为RGB数据
    let convert_start = Instant::now();
    let pixel_count = (width * height) as usize;
    let mut rgb_data = Vec::with_capacity(pixel_count * 3);
    rgb_data.resize(pixel_count * 3, 0);

    let cropped_rgba = cropped_image.as_raw();
    cropped_rgba
        .par_chunks_exact(4)
        .zip(rgb_data.par_chunks_exact_mut(3))
        .for_each(|(rgba, rgb)| {
            rgb[0] = rgba[0];
            rgb[1] = rgba[1];
            rgb[2] = rgba[2];
        });
    info!("RGBA到RGB转换完成, 耗时: {:?}", convert_start.elapsed());

    // 将图像编码为JPEG格式
    let encode_start = Instant::now();
    let mut buffer: Vec<u8> = Vec::new();
    {
        let encoder = jpeg_encoder::Encoder::new(&mut buffer, 100);
        encoder
            .encode(
                &rgb_data,
                width as u16,
                height as u16,
                jpeg_encoder::ColorType::Rgb,
            )
            .map_err(|e| {
                error!("图像JPEG编码失败: {}", e);
                format!("图像JPEG编码失败: {}", e)
            })?;
    }
    info!("图像JPEG编码完成, 耗时: {:?}", encode_start.elapsed());

    // 将字节转换为Base64字符串
    let base64_start = Instant::now();
    let base64_image = general_purpose::STANDARD.encode(&buffer);
    info!("Base64编码完成, 耗时: {:?}", base64_start.elapsed());

    info!("区域截图任务完成, 总耗时: {:?}", total_start.elapsed());
    Ok(base64_image)
}

/// 获取截图保存目录
fn get_screenshots_dir() -> Result<PathBuf, String> {
    if let Some(pictures_dir) = dirs::picture_dir() {
        let screenshots_dir = pictures_dir.join("screenshots");

        // 创建目录（如果不存在）
        if !screenshots_dir.exists() {
            fs::create_dir_all(&screenshots_dir).map_err(|e| {
                error!("无法创建截图目录: {}", e);
                format!("无法创建截图目录: {}", e)
            })?;
        }

        Ok(screenshots_dir)
    } else {
        // 如果无法获取图片目录，使用当前目录
        let current_dir = std::env::current_dir().map_err(|e| {
            error!("无法获取当前目录: {}", e);
            format!("无法获取当前目录: {}", e)
        })?;
        Ok(current_dir.join("screenshots"))
    }
}

/// 生成截图文件名（带时间戳）
fn generate_screenshot_filename() -> String {
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    format!("screenshot_{}.jpg", timestamp)
}

/// 保存截图到文件
#[command]
pub fn save_screenshot(base64_data: String, filename: Option<String>) -> Result<String, String> {
    let total_start = Instant::now();
    info!("开始保存截图任务...");

    // 解码Base64数据
    let decode_start = Instant::now();
    let image_data = general_purpose::STANDARD
        .decode(&base64_data)
        .map_err(|e| {
            error!("Base64解码失败: {}", e);
            format!("Base64解码失败: {}", e)
        })?;
    info!("Base64解码完成, 耗时: {:?}", decode_start.elapsed());

    // 获取保存目录
    let dir_start = Instant::now();
    let screenshots_dir = get_screenshots_dir()?;
    info!(
        "获取截图目录完成: {:?}, 耗时: {:?}",
        screenshots_dir,
        dir_start.elapsed()
    );

    // 生成文件名
    let filename = filename.unwrap_or_else(generate_screenshot_filename);
    let file_path = screenshots_dir.join(&filename);
    info!("准备保存到文件: {:?}", file_path);

    // 保存文件
    let save_start = Instant::now();
    fs::write(&file_path, &image_data).map_err(|e| {
        error!("保存文件失败: {}", e);
        format!("保存文件失败: {}", e)
    })?;
    info!("文件保存完成, 耗时: {:?}", save_start.elapsed());

    info!("截图保存任务完成, 总耗时: {:?}", total_start.elapsed());
    Ok(file_path.to_string_lossy().to_string())
}

/// 截图指定区域并自动保存到文件
#[command]
pub fn capture_and_save_region(x: u32, y: u32, width: u32, height: u32) -> Result<String, String> {
    info!("开始捕获并保存区域截图...");

    // 捕获区域
    let base64_image = capture_region(x, y, width, height)?;

    // 生成文件名并保存
    let filename = generate_screenshot_filename();
    let file_path = save_screenshot(base64_image, Some(filename))?;

    info!("区域截图已保存: {}", file_path);
    Ok(file_path)
}

/// 截图指定区域并复制到剪贴板
#[command]
pub fn capture_and_copy_region(x: u32, y: u32, width: u32, height: u32) -> Result<String, String> {
    info!("开始捕获并复制区域截图到剪贴板...");
    info!(
        "接收到的参数: x={}, y={}, width={}, height={}",
        x, y, width, height
    );

    // 获取所有显示器
    let monitors = Monitor::all().map_err(|e| {
        error!("获取显示器列表失败: {}", e);
        e.to_string()
    })?;

    // 获取主显示器
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or("未找到可用显示器".to_string())?;

    // 截图整个屏幕
    let image = monitor.capture_image().map_err(|e| {
        error!("截图失败: {}", e);
        format!("截图失败: {}", e)
    })?;

    let screen_width = image.width();
    let screen_height = image.height();
    let rgba_data = image.as_raw();

    info!("屏幕尺寸: {}x{}", screen_width, screen_height);
    info!("RGBA数据总大小: {} 字节", rgba_data.len());

    // 验证区域是否在屏幕范围内
    if x + width > screen_width || y + height > screen_height {
        error!(
            "指定区域超出屏幕范围: screen={}x{}, region=({}, {}, {}x{})",
            screen_width, screen_height, x, y, width, height
        );
        return Err("指定区域超出屏幕范围".to_string());
    }

    // 计算预期的数据大小
    let expected_size = (width * height * 4) as usize;
    info!(
        "裁剪区域预期数据大小: {} 字节 ({} x {} x 4)",
        expected_size, width, height
    );

    // 将RGBA数据转换为image::RgbaImage
    let full_image = image::RgbaImage::from_raw(screen_width, screen_height, rgba_data.to_vec())
        .ok_or("无法创建图像".to_string())?;

    // 裁剪指定区域
    let cropped_view = imageops::crop_imm(&full_image, x, y, width, height);

    // 手动提取裁剪区域的像素数据，创建新的独立图像
    let cropped_data = cropped_view.to_image();

    info!(
        "裁剪后图像实际尺寸: {}x{}",
        cropped_data.width(),
        cropped_data.height()
    );
    info!(
        "裁剪后图像实际数据大小: {} 字节",
        cropped_data.as_raw().len()
    );

    // 将裁剪后的图像转换为PNG格式的字节数据（PNG支持透明度，适合剪贴板）
    let mut buffer = Vec::new();
    {
        let cropped_width = cropped_data.width();
        let cropped_height = cropped_data.height();
        let raw_data = cropped_data.as_raw();

        info!("PNG编码器使用尺寸: {}x{}", cropped_width, cropped_height);
        info!("PNG编码器将处理数据: {} 字节", raw_data.len());

        let mut encoder = png::Encoder::new(&mut buffer, cropped_width, cropped_height);
        // 设置颜色类型为RGBA，因为我们传递的是RGBA数据（每像素4字节）
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().map_err(|e| {
            error!("PNG编码头失败: {}", e);
            format!("PNG编码头失败: {}", e)
        })?;
        writer.write_image_data(raw_data).map_err(|e| {
            error!("PNG编码数据失败: {}", e);
            format!("PNG编码数据失败: {}", e)
        })?;
    }

    // 将图像复制到剪贴板
    #[cfg(target_os = "macos")]
    {
        let mut clipboard = Clipboard::new().map_err(|e| {
            error!("无法访问剪贴板: {}", e);
            format!("无法访问剪贴板: {}", e)
        })?;

        let raw_data = cropped_data.as_raw().to_vec();
        clipboard
            .set_image(arboard::ImageData {
                width: cropped_data.width() as usize,
                height: cropped_data.height() as usize,
                bytes: Cow::Owned(raw_data),
            })
            .map_err(|e| {
                error!("无法复制图像到剪贴板: {}", e);
                format!("无法复制图像到剪贴板: {}", e)
            })?;

        info!("区域截图已复制到剪贴板");
    }

    #[cfg(not(target_os = "macos"))]
    {
        return Err("剪贴板功能仅支持macOS".to_string());
    }

    Ok("截图已复制到剪贴板".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_screen() {
        // 测试截图功能是否正常工作
        let result = capture_screen();

        // 验证结果是成功的
        assert!(result.is_ok());

        // 获取Base64字符串
        let base64_string = result.unwrap();

        // 验证Base64字符串不为空
        assert!(!base64_string.is_empty());

        // 验证Base64字符串可以解码
        let decoded = general_purpose::STANDARD.decode(&base64_string);
        assert!(decoded.is_ok());

        // 验证解码后的数据可以作为PNG图像加载
        let decoded_data = decoded.unwrap();
        let image_result =
            image::load_from_memory_with_format(&decoded_data, image::ImageFormat::Png);
        assert!(image_result.is_ok());
    }
}
