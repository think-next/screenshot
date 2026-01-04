use base64::{engine::general_purpose, Engine as _};
use log::{error, info};
use rayon::prelude::*;
use std::time::Instant;
use tauri::command;
use xcap::Monitor;

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
    info!("获取显示器列表完成，耗时: {:?}", monitors_start.elapsed());

    // 获取主显示器（如果没有主显示器，则获取第一个显示器）
    let monitor_select_start = Instant::now();
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or("未找到可用显示器".to_string())?;
    info!("选择显示器完成，耗时: {:?}", monitor_select_start.elapsed());

    // 截图
    let capture_start = Instant::now();
    let image = monitor.capture_image().map_err(|e| {
        error!("截图失败: {}", e);
        format!("截图失败: {}", e)
    })?;
    info!("截图操作完成，耗时: {:?}", capture_start.elapsed());

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
    info!("RGBA到RGB转换完成，耗时: {:?}", convert_start.elapsed());

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
    info!("图像JPEG编码完成，耗时: {:?}", encode_start.elapsed());

    // 将字节转换为Base64字符串
    let base64_start = Instant::now();
    let base64_image = general_purpose::STANDARD.encode(&buffer);
    info!("Base64编码完成，耗时: {:?}", base64_start.elapsed());

    info!("截图任务完成，总耗时: {:?}", total_start.elapsed());
    Ok(base64_image)
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
