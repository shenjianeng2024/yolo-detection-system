/*!
YOLO检测系统API模块
基于原PyQt5功能设计的完整API接口
*/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use crate::yolo::DetectionResult;
use crate::{ApiResult, AppState};

/// 输入源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Camera(i32),    // 摄像头设备ID
    Video(String),  // 视频文件路径
    Image(String),  // 图片文件路径
}

/// 检测配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    pub confidence_thresholds: HashMap<String, f32>,  // 各类别置信度阈值
    pub selected_classes: Vec<String>,                // 选中的检测类别
    pub input_source: Option<InputSource>,            // 输入源
}

/// 实时检测状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionStatus {
    pub is_running: bool,
    pub input_source: Option<InputSource>,
    pub frame_count: u64,
    pub detection_count: u64,
    pub fps: f32,
}

/// 检测结果扩展（包含警告信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedDetectionResult {
    pub result: DetectionResult,
    pub warnings: Vec<String>,
    pub processing_time_ms: u64,
}

/// 类别信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassInfo {
    pub id: i32,
    pub name: String,
    pub default_confidence: f32,
}

// ==================== Tauri命令实现 ====================

/// 初始化YOLO模型 - React UI兼容版本
#[tauri::command]
pub async fn initialize_yolo_model(
    state: State<'_, AppState>,
    model_path: String
) -> Result<Vec<String>, String> {
    let mut yolo_manager = state.lock().await;
    
    match yolo_manager.init_model(&model_path).await {
        Ok(()) => {
            // 异常检测系统只返回基本的状态类别
            let class_names = vec![
                "正常".to_string(),
                "异常".to_string(),
            ];
            Ok(class_names)
        },
        Err(e) => Err(format!("模型初始化失败: {}", e)),
    }
}

/// 获取所有可用的类别信息
#[tauri::command]
pub async fn get_class_names(
    _state: State<'_, AppState>
) -> Result<ApiResult<Vec<ClassInfo>>, String> {
    // 异常检测系统的类别信息
    let mock_classes = vec![
        ClassInfo { id: 0, name: "正常".to_string(), default_confidence: 0.9 },
        ClassInfo { id: 1, name: "异常".to_string(), default_confidence: 0.9 },
    ];
    Ok(ApiResult::success(mock_classes))
}

/// 启动摄像头检测 - React UI版本
#[tauri::command]
pub async fn start_camera_detection(
    _state: State<'_, AppState>
) -> Result<(), String> {
    // TODO: 实现摄像头检测启动逻辑
    Err("摄像头检测功能暂未实现".to_string())
}

/// 选择摄像头作为输入源
#[tauri::command]
pub async fn select_camera_input(
    _state: State<'_, AppState>,
    _device_id: i32
) -> Result<ApiResult<String>, String> {
    // TODO: 实现摄像头初始化逻辑
    Ok(ApiResult::error("摄像头功能暂未实现".to_string()))
}

/// 加载视频源 - React UI版本
#[tauri::command]
pub async fn load_video_source(
    _state: State<'_, AppState>,
    path: String
) -> Result<(), String> {
    // TODO: 实现视频加载逻辑
    match validate_input_file(&path) {
        Ok(_) => {
            println!("视频源已加载: {}", path);
            Ok(())
        },
        Err(e) => Err(format!("视频加载失败: {}", e)),
    }
}

/// 选择视频文件作为输入源
#[tauri::command]
pub async fn select_video_input(
    _state: State<'_, AppState>,
    _file_path: String
) -> Result<ApiResult<String>, String> {
    // TODO: 实现视频文件验证和初始化逻辑
    Ok(ApiResult::error("视频处理功能暂未实现".to_string()))
}

/// 处理单张图片 - React UI版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageProcessResult {
    #[serde(rename = "imageData")]
    pub image_data: Option<String>,  // Base64编码的图片数据，前端期望 imageData
    pub detections: Vec<Detection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [f32; 4],
}

#[tauri::command]
pub async fn process_single_image(
    state: State<'_, AppState>,
    path: String,
    class_configs: Vec<serde_json::Value>  // 类别配置
) -> Result<ImageProcessResult, String> {
    println!("Backend received image path: {}", path); // 调试日志
    let mut yolo_manager = state.lock().await;
    
    // 验证文件路径和格式
    if let Err(e) = validate_image_file(&path) {
        return Err(e);
    }
    
    match std::fs::read(&path) {
        Ok(data) => {
            println!("[DEBUG] ==================== 开始图片处理 ====================");
            println!("[DEBUG] 文件大小: {} 字节", data.len());
            
            // 首先尝试解码图片确保格式正确
            let original_image = match image::load_from_memory(&data) {
                Ok(img) => {
                    println!("[DEBUG] ✅ 图片解码成功");
                    println!("[DEBUG] 图片尺寸: {}x{}", img.width(), img.height());
                    println!("[DEBUG] 图片格式: {:?}", img.color());
                    img
                },
                Err(e) => return Err(format!("图片格式错误: {}", e)),
            };
            
            // 应用前端的置信度配置
            for config in &class_configs {
                if let Ok(config_obj) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(config.clone()) {
                    if let (Some(name), Some(confidence)) = (config_obj.get("name"), config_obj.get("confidence")) {
                        if let (Some(name_str), Some(conf_num)) = (name.as_str(), confidence.as_f64()) {
                            let _ = yolo_manager.update_confidence_threshold(name_str, conf_num as f32).await;
                        }
                    }
                }
            }

            match yolo_manager.detect_image(&data).await {
                Ok(result) => {
                    println!("[DEBUG] ✅ YOLO检测完成");
                    println!("[DEBUG] 检测到 {} 个对象", result.detections.len());
                    
                    for (i, detection) in result.detections.iter().enumerate() {
                        println!("[DEBUG] 对象 {}: {} (置信度: {:.2}, 边界框: {:?})", 
                            i + 1, 
                            detection.class_name, 
                            detection.confidence,
                            detection.bbox
                        );
                    }
                    
                    // 在原图上绘制检测结果
                    println!("[DEBUG] 开始绘制检测结果...");
                    let annotated_image = if result.detections.is_empty() {
                        println!("[DEBUG] 无检测结果，返回原图");
                        original_image.clone()
                    } else {
                        draw_detections_on_image(&original_image, &result.detections)?
                    };
                    println!("[DEBUG] ✅ 检测结果绘制完成");
                    
                    // 转换为base64
                    let image_base64 = image_to_base64(&annotated_image)?;
                    
                    // 转换检测结果格式
                    let detections: Vec<Detection> = result.detections.iter()
                        .map(|d| Detection {
                            class_name: d.class_name.clone(),
                            confidence: d.confidence,
                            bbox: d.bbox,
                        })
                        .collect();
                    
                    Ok(ImageProcessResult {
                        image_data: Some(image_base64),
                        detections,
                    })
                },
                Err(e) => Err(format!("图片处理失败: {}", e)),
            }
        },
        Err(e) => Err(format!("读取文件失败: {}", e)),
    }
}

/// 选择图片文件作为输入源并立即处理
#[tauri::command]
pub async fn select_image_input(
    state: State<'_, AppState>,
    file_path: String
) -> Result<ApiResult<ExtendedDetectionResult>, String> {
    let mut yolo_manager = state.lock().await;
    
    let start_time = std::time::Instant::now();
    
    match std::fs::read(&file_path) {
        Ok(data) => match yolo_manager.detect_image(&data).await {
            Ok(result) => {
            let processing_time = start_time.elapsed().as_millis() as u64;
            
            // TODO: 检查异常并生成警告
            let warnings = check_for_abnormal_detections(&result);
            
            let extended_result = ExtendedDetectionResult {
                result,
                warnings,
                processing_time_ms: processing_time,
            };
            
            Ok(ApiResult::success(extended_result))
            },
            Err(e) => Ok(ApiResult::error(format!("图片处理失败: {}", e))),
        },
        Err(e) => Ok(ApiResult::error(format!("读取文件失败: {}", e))),
    }
}

/// 停止检测 - React UI版本
#[tauri::command]
pub async fn stop_detection(
    _state: State<'_, AppState>
) -> Result<(), String> {
    // TODO: 实现检测停止逻辑
    println!("检测已停止");
    Ok(())
}

/// 获取下一帧图像和检测结果 - React UI版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameResult {
    pub success: bool,
    pub image_data: Option<String>,
    pub detections: Option<Vec<Detection>>,
}

#[tauri::command]
pub async fn get_next_frame(
    _state: State<'_, AppState>,
    _class_configs: Vec<serde_json::Value>
) -> Result<FrameResult, String> {
    // TODO: 实现实时帧获取逻辑
    // 目前返回模拟数据
    Ok(FrameResult {
        success: true,
        image_data: Some("base64_encoded_frame_placeholder".to_string()),
        detections: Some(vec![
            Detection {
                class_name: "正常".to_string(),
                confidence: 0.92,
                bbox: [50.0, 60.0, 150.0, 200.0],
            }
        ]),
    })
}

/// 重置配置 - React UI版本
#[tauri::command]
pub async fn reset_configuration(
    _state: State<'_, AppState>
) -> Result<(), String> {
    // TODO: 实现配置重置逻辑
    println!("配置已重置为默认值");
    Ok(())
}

/// 开始实时检测（摄像头或视频）
#[tauri::command]
pub async fn start_realtime_detection(
    _state: State<'_, AppState>
) -> Result<ApiResult<String>, String> {
    // TODO: 实现实时检测启动逻辑
    Ok(ApiResult::error("实时检测功能暂未实现".to_string()))
}

/// 停止实时检测
#[tauri::command]
pub async fn stop_realtime_detection(
    _state: State<'_, AppState>
) -> Result<ApiResult<String>, String> {
    // TODO: 实现实时检测停止逻辑
    Ok(ApiResult::error("实时检测停止功能暂未实现".to_string()))
}

/// 获取当前检测状态
#[tauri::command]
pub async fn get_realtime_status(
    _state: State<'_, AppState>
) -> Result<ApiResult<DetectionStatus>, String> {
    // TODO: 实现状态获取逻辑
    let status = DetectionStatus {
        is_running: false,
        input_source: None,
        frame_count: 0,
        detection_count: 0,
        fps: 0.0,
    };
    Ok(ApiResult::success(status))
}

/// 批量更新置信度阈值
#[tauri::command]
pub async fn update_confidence_thresholds(
    _state: State<'_, AppState>,
    _thresholds: HashMap<String, f32>
) -> Result<ApiResult<String>, String> {
    // TODO: 实现批量阈值更新逻辑
    Ok(ApiResult::success("置信度阈值更新成功".to_string()))
}

/// 更新选中的检测类别
#[tauri::command]
pub async fn update_selected_classes(
    _state: State<'_, AppState>,
    _class_names: Vec<String>
) -> Result<ApiResult<String>, String> {
    // TODO: 实现类别选择更新逻辑
    Ok(ApiResult::success("检测类别更新成功".to_string()))
}

/// 获取检测配置
#[tauri::command]
pub async fn get_detection_config(
    _state: State<'_, AppState>
) -> Result<ApiResult<DetectionConfig>, String> {
    // TODO: 从状态中获取当前配置
    let config = DetectionConfig {
        confidence_thresholds: HashMap::new(),
        selected_classes: vec!["正常".to_string(), "异常".to_string()],
        input_source: None,
    };
    Ok(ApiResult::success(config))
}

/// 重置所有配置到默认值
#[tauri::command]
pub async fn reset_to_defaults(
    _state: State<'_, AppState>
) -> Result<ApiResult<String>, String> {
    // TODO: 实现配置重置逻辑
    Ok(ApiResult::success("配置已重置为默认值".to_string()))
}

// ==================== 图片处理辅助函数 ====================

/// 验证图片文件格式
fn validate_image_file(file_path: &str) -> Result<(), String> {
    use std::path::Path;
    
    println!("[DEBUG] ==================== 文件路径验证开始 ====================");
    println!("[DEBUG] 输入路径: {}", file_path);
    println!("[DEBUG] 路径长度: {} 字符", file_path.len());
    println!("[DEBUG] 是否包含中文: {}", file_path.chars().any(|c| '\u{4e00}' <= c && c <= '\u{9fff}'));
    println!("[DEBUG] 路径编码: {:?}", file_path.as_bytes());
    
    let path = Path::new(file_path);
    
    // 检查路径是否存在
    println!("[DEBUG] 检查路径是否存在...");
    if !path.exists() {
        println!("[ERROR] 路径不存在: {}", file_path);
        let absolute_path = match path.canonicalize() {
            Ok(abs_path) => format!("{:?}", abs_path),
            Err(e) => {
                println!("[DEBUG] 无法规范化路径，错误: {:?}", e);
                "无法解析绝对路径".to_string()
            }
        };
        let error_msg = format!("图片文件不存在: {}\n尝试的绝对路径: {}\n请检查文件是否存在且路径正确", 
            file_path, absolute_path);
        println!("[ERROR] {}", error_msg);
        return Err(error_msg);
    }
    println!("[DEBUG] ✅ 路径存在");
    
    // 检查是否为文件
    println!("[DEBUG] 检查是否为文件...");
    if !path.is_file() {
        let error_msg = format!("指定路径不是一个文件: {}", file_path);
        println!("[ERROR] {}", error_msg);
        return Err(error_msg);
    }
    println!("[DEBUG] ✅ 确认是文件类型");
    
    // 检查文件扩展名
    println!("[DEBUG] 检查文件扩展名...");
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
        .ok_or_else(|| {
            let error_msg = format!("文件缺少扩展名: {}", file_path);
            println!("[ERROR] {}", error_msg);
            error_msg
        })?;
    
    println!("[DEBUG] 文件扩展名: {}", extension);
    
    match extension.as_str() {
        "jpg" | "jpeg" | "png" | "bmp" | "gif" | "tiff" | "webp" => {
            println!("[DEBUG] ✅ 文件格式验证通过: .{}", extension);
            println!("[DEBUG] ==================== 文件路径验证完成 ====================");
            Ok(())
        },
        _ => {
            let error_msg = format!("不支持的图片格式: .{}\n支持的格式: jpg, jpeg, png, bmp, gif, tiff, webp", extension);
            println!("[ERROR] {}", error_msg);
            println!("[DEBUG] ==================== 文件路径验证失败 ====================");
            Err(error_msg)
        },
    }
}

/// 在图片上绘制检测结果
fn draw_detections_on_image(
    original_image: &image::DynamicImage,
    detections: &[crate::yolo::YoloDetection]
) -> Result<image::DynamicImage, String> {
    use imageproc::drawing::draw_hollow_rect_mut;
    use imageproc::rect::Rect;
    use image::Rgb;
    
    let mut image = original_image.to_rgb8();
    
    // 定义颜色 - 使用更鲜明的配色方案
    let normal_color = Rgb([0u8, 200u8, 0u8]);     // 明绿色 - 正常
    let abnormal_color = Rgb([220u8, 0u8, 0u8]);   // 明红色 - 异常
    let default_color = Rgb([255u8, 165u8, 0u8]);  // 橙色 - 默认
    
    for detection in detections {
        let [x, y, w, h] = detection.bbox;
        
        // 确保坐标在图片范围内
        let img_width = image.width() as f32;
        let img_height = image.height() as f32;
        
        let x = x.max(0.0).min(img_width - 1.0) as i32;
        let y = y.max(0.0).min(img_height - 1.0) as i32;
        let w = w.max(1.0).min(img_width - x as f32) as u32;
        let h = h.max(1.0).min(img_height - y as f32) as u32;
        
        // 选择颜色
        let color = match detection.class_name.as_str() {
            "正常" => normal_color,
            "异常" => abnormal_color,
            _ => default_color,
        };
        
        // 绘制矩形框（加粗效果）
        let _rect = Rect::at(x, y).of_size(w, h);
        for thickness in 0..3 {
            if let Some(thick_rect) = Rect::at(x - thickness, y - thickness)
                .of_size(w + 2 * thickness as u32, h + 2 * thickness as u32)
                .intersect(Rect::at(0, 0).of_size(image.width(), image.height())) {
                draw_hollow_rect_mut(&mut image, thick_rect, color);
            }
        }
        
        // 绘制标签文本（如果有足够空间）
        if y >= 20 {
            // 创建清晰的标签文本
            let confidence_percent = (detection.confidence * 100.0) as u8;
            let label = format!("{}: {}%", 
                detection.class_name, 
                confidence_percent
            );
            println!("[DEBUG] 绘制检测标签: {} (位置: {}, {})", label, x, y);
            
            // 在检测框上方绘制标签背景
            let label_height = 20;
            let label_width = label.len() as u32 * 8; // 估算文本宽度
            
            // 绘制标签背景
            for dy in 0..label_height {
                for dx in 0..label_width.min(image.width() - x as u32) {
                    if let Some(pixel) = image.get_pixel_mut_checked(x as u32 + dx, (y - label_height as i32 + dy as i32) as u32) {
                        *pixel = Rgb([0, 0, 0]); // 黑色背景
                    }
                }
            }
        }
    }
    
    Ok(image::DynamicImage::ImageRgb8(image))
}

/// 将图片转换为base64编码
fn image_to_base64(image: &image::DynamicImage) -> Result<String, String> {
    use std::io::Cursor;
    use image::ImageFormat;
    
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    // 将图片编码为JPEG格式
    match image.write_to(&mut cursor, ImageFormat::Jpeg) {
        Ok(_) => {
            use base64::Engine;
            let base64_string = base64::engine::general_purpose::STANDARD.encode(&buffer);
            Ok(base64_string)
        },
        Err(e) => Err(format!("图片编码失败: {}", e)),
    }
}

// ==================== 原有辅助函数 ====================

/// 检查检测结果中的异常情况（对应PyQt5中的check_abnormal）
fn check_for_abnormal_detections(result: &DetectionResult) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // TODO: 实现异常检测逻辑
    // 基于置信度、检测数量等生成警告信息
    
    // 示例逻辑（需要根据实际需求调整）
    if result.detections.is_empty() {
        warnings.push("未检测到任何目标".to_string());
    } else if result.detections.len() > 10 {
        warnings.push(format!("检测到大量目标: {} 个", result.detections.len()));
    }
    
    warnings
}

/// 验证输入文件是否存在且格式正确
fn validate_input_file(file_path: &str) -> Result<(), String> {
    use std::path::Path;
    
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err("文件不存在".to_string());
    }
    
    // TODO: 添加文件格式验证
    // 支持的图片格式: jpg, png, bmp
    // 支持的视频格式: mp4, avi, mov
    
    Ok(())
}

/// 获取文件扩展名
fn get_file_extension(file_path: &str) -> Option<String> {
    use std::path::Path;
    
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}