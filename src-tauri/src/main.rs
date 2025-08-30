// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    class_name: String,
    confidence: f32,
    bbox: [f32; 4], // [x, y, width, height]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionState {
    is_running: bool,
    current_source: Option<String>,
    source_type: Option<String>, // "camera", "video", "image"
    results: Vec<DetectionResult>,
}

type AppState = Arc<Mutex<DetectionState>>;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn start_camera(state: State<'_, AppState>) -> Result<String, String> {
    let mut detection_state = state.lock().unwrap();
    detection_state.current_source = Some("camera".to_string());
    detection_state.source_type = Some("camera".to_string());
    detection_state.is_running = false; // 需要手动开始检测
    
    Ok("Camera started successfully".to_string())
}

#[tauri::command]
async fn stop_detection(state: State<'_, AppState>) -> Result<String, String> {
    let mut detection_state = state.lock().unwrap();
    detection_state.is_running = false;
    detection_state.results.clear();
    
    Ok("Detection stopped successfully".to_string())
}

#[tauri::command]
async fn select_video_file(state: State<'_, AppState>) -> Result<String, String> {
    let mut detection_state = state.lock().unwrap();
    detection_state.current_source = Some("/path/to/video.mp4".to_string());
    detection_state.source_type = Some("video".to_string());
    Ok("Video file selected (demo)".to_string())
}

#[tauri::command]
async fn select_image_file(state: State<'_, AppState>) -> Result<String, String> {
    let mut detection_state = state.lock().unwrap();
    detection_state.current_source = Some("/path/to/image.jpg".to_string());
    detection_state.source_type = Some("image".to_string());
    Ok("Image file selected (demo)".to_string())
}

#[tauri::command]
async fn get_detection_state(state: State<'_, AppState>) -> Result<DetectionState, String> {
    let detection_state = state.lock().unwrap();
    Ok(detection_state.clone())
}

#[tauri::command]
async fn start_detection(state: State<'_, AppState>) -> Result<String, String> {
    let mut detection_state = state.lock().unwrap();
    
    if detection_state.current_source.is_none() {
        return Err("No input source selected".to_string());
    }
    
    detection_state.is_running = true;
    
    // 模拟检测结果
    detection_state.results = vec![
        DetectionResult {
            class_name: "person".to_string(),
            confidence: 0.85,
            bbox: [100.0, 150.0, 200.0, 400.0],
        },
        DetectionResult {
            class_name: "car".to_string(),
            confidence: 0.92,
            bbox: [300.0, 200.0, 150.0, 100.0],
        },
        DetectionResult {
            class_name: "bicycle".to_string(),
            confidence: 0.76,
            bbox: [500.0, 250.0, 120.0, 180.0],
        },
    ];
    
    Ok("Detection started successfully".to_string())
}

fn main() {
    let initial_state = DetectionState {
        is_running: false,
        current_source: None,
        source_type: None,
        results: Vec::new(),
    };

    tauri::Builder::default()
        .manage(Arc::new(Mutex::new(initial_state)))
        .invoke_handler(tauri::generate_handler![
            greet,
            start_camera,
            stop_detection,
            select_video_file,
            select_image_file,
            get_detection_state,
            start_detection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}