use base64::{engine::general_purpose, Engine as _};
use tauri::command;
use xcap::Monitor;

/// 截图并返回Base64编码的PNG图像
#[command]
pub fn capture_screen() -> Result<String, String> {
    // 获取所有显示器
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    
    // 获取主显示器（如果没有主显示器，则获取第一个显示器）
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or("未找到可用显示器".to_string())?;

    // 截图
    let image = monitor
        .capture_image()
        .map_err(|e| format!("截图失败: {}", e))?;

    // 将图像编码为PNG格式的字节
    let mut buffer: Vec<u8> = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("图像编码失败: {}", e))?;

    // 将字节转换为Base64字符串
    let base64_image = general_purpose::STANDARD.encode(&buffer);

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
        let image_result = image::load_from_memory_with_format(&decoded_data, image::ImageFormat::Png);
        assert!(image_result.is_ok());
    }
}