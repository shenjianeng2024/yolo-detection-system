// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod yolo;
mod yolo_api;

use std::sync::{Arc};
use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use yolo::{CandleYoloDetector, DetectionResult, ModelStats};
use yolo_api::*;

/// API响应结果包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

type AppState = Arc<Mutex<CandleYoloDetector>>;

/// 初始化YOLO模型
#[tauri::command]
async fn init_yolo_model(
    state: State<'_, AppState>,
    model_path: String
) -> Result<ApiResult<String>, String> {
    let mut yolo_manager = state.lock().await;
    
    match yolo_manager.init_model(&model_path).await {
        Ok(()) => Ok(ApiResult::success("YOLO模型初始化成功".to_string())),
        Err(e) => Ok(ApiResult::error(format!("模型初始化失败: {}", e))),
    }
}

/// 处理图像检测
#[tauri::command]
async fn process_image(
    state: State<'_, AppState>,
    image_path: String
) -> Result<ApiResult<DetectionResult>, String> {
    let mut yolo_detector = state.lock().await;
    
    // 读取图像文件
    match std::fs::read(&image_path) {
        Ok(image_data) => {
            match yolo_detector.detect_image(&image_data).await {
                Ok(result) => Ok(ApiResult::success(result)),
                Err(e) => Ok(ApiResult::error(format!("图像处理失败: {}", e))),
            }
        }
        Err(e) => Ok(ApiResult::error(format!("读取图像文件失败: {}", e))),
    }
}

/// 开始摄像头检测 (原版本)
#[tauri::command]
async fn start_camera_detection_legacy(
    _state: State<'_, AppState>,
    _device_id: i32
) -> Result<ApiResult<String>, String> {
    // 暂时不支持摄像头
    Ok(ApiResult::error("摄像头功能暂未实现".to_string()))
}

/// 开始视频检测
#[tauri::command]
async fn start_video_detection(
    _state: State<'_, AppState>,
    _video_path: String
) -> Result<ApiResult<String>, String> {
    // 暂时不支持视频
    Ok(ApiResult::error("视频检测功能暂未实现".to_string()))
}

/// 停止检测 (原版本)
#[tauri::command]
async fn stop_detection_legacy(_state: State<'_, AppState>) -> Result<ApiResult<String>, String> {
    // Candle检测器不需要显式停止操作
    Ok(ApiResult::success("检测已停止".to_string()))
}

/// 获取检测统计信息
#[tauri::command]
async fn get_detection_state(
    state: State<'_, AppState>
) -> Result<ApiResult<ModelStats>, String> {
    let yolo_detector = state.lock().await;
    let stats = yolo_detector.get_stats().await;
    Ok(ApiResult::success(stats))
}

/// 更新置信度阈值
#[tauri::command]
async fn update_confidence_threshold(
    state: State<'_, AppState>,
    class_name: String,
    threshold: f32
) -> Result<ApiResult<String>, String> {
    let yolo_detector = state.lock().await;
    
    match yolo_detector.update_confidence_threshold(&class_name, threshold).await {
        Ok(()) => Ok(ApiResult::success("置信度阈值已更新".to_string())),
        Err(e) => Ok(ApiResult::error(format!("更新失败: {}", e))),
    }
}

/// 设置选中的类别
#[tauri::command]
async fn set_selected_classes(
    state: State<'_, AppState>,
    class_ids: Vec<i32>
) -> Result<ApiResult<String>, String> {
    let yolo_detector = state.lock().await;
    
    // 转换i32到u32
    let class_ids_u32: Vec<u32> = class_ids.into_iter().map(|id| id as u32).collect();
    
    match yolo_detector.set_enabled_classes(class_ids_u32).await {
        Ok(()) => Ok(ApiResult::success("类别选择已更新".to_string())),
        Err(e) => Ok(ApiResult::error(format!("更新失败: {}", e))),
    }
}

fn main() {
    // 初始化YOLO Candle检测器
    let yolo_detector = CandleYoloDetector::new();

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(yolo_detector)))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // 原有API (legacy)
            init_yolo_model,
            process_image,
            start_video_detection,
            start_camera_detection_legacy,
            stop_detection_legacy,
            get_detection_state,
            update_confidence_threshold,
            set_selected_classes,
            // React UI兼容API (现在使用的主要API)
            initialize_yolo_model,
            start_camera_detection,
            load_video_source,
            process_single_image,
            stop_detection,
            get_next_frame,
            reset_configuration,
            // 扩展API（基于PyQt5功能设计）
            get_class_names,
            select_camera_input,
            select_video_input,
            select_image_input,
            start_realtime_detection,
            stop_realtime_detection,
            get_realtime_status,
            update_confidence_thresholds,
            update_selected_classes,
            get_detection_config,
            reset_to_defaults
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}