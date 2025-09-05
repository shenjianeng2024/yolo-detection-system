use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::{YoloDetection, CandleYoloModel as YoloModel, ConfidenceThresholds};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Image { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub detections: Vec<YoloDetection>,
    pub frame_data: Option<String>, // base64编码的图像数据
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionState {
    pub is_running: bool,
    pub current_source: Option<InputSource>,
    pub results: Vec<DetectionResult>,
    pub selected_classes: Vec<u32>,
}

pub struct YoloDetectionEngine {
    model: Arc<YoloModel>,
    thresholds: Arc<ConfidenceThresholds>,
    state: Arc<RwLock<DetectionState>>,
}

impl YoloDetectionEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        let model = Arc::new(YoloModel::new(model_path)?);
        let thresholds = Arc::new(ConfidenceThresholds::new());
        
        let initial_state = DetectionState {
            is_running: false,
            current_source: None,
            results: Vec::new(),
            selected_classes: vec![0, 1], // 默认选择所有类别
        };

        Ok(Self {
            model,
            thresholds,
            state: Arc::new(RwLock::new(initial_state)),
        })
    }

    pub async fn process_image(&self, image_path: &str) -> Result<DetectionResult> {
        // 检查文件是否存在
        if !Path::new(image_path).exists() {
            return Err(anyhow::anyhow!("Image file does not exist: {}", image_path));
        }

        // 读取图像文件
        let image_data = tokio::fs::read(image_path).await
            .context("Failed to read image file")?;

        // 运行检测
        let detections = self.model.detect_image(&image_data).await?;

        // 过滤检测结果
        let filtered_detections = self.filter_detections(detections).await;

        // 将原始图像转换为base64（暂时使用原图，后续可以添加绘制结果的功能）
        use base64::Engine;
        let image_base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.current_source = Some(InputSource::Image { path: image_path.to_string() });
        }

        Ok(DetectionResult {
            detections: filtered_detections,
            frame_data: Some(image_base64),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    pub async fn stop_detection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.is_running = false;
        state.current_source = None;
        Ok(())
    }

    pub async fn update_confidence_threshold(&self, class_name: &str, threshold: f32) -> Result<()> {
        self.thresholds.update_threshold(class_name, threshold).await;
        Ok(())
    }

    pub async fn get_detection_state(&self) -> DetectionState {
        self.state.read().await.clone()
    }

    pub async fn set_selected_classes(&self, class_ids: Vec<u32>) -> Result<()> {
        let mut state = self.state.write().await;
        state.selected_classes = class_ids;
        Ok(())
    }

    async fn filter_detections(&self, detections: Vec<YoloDetection>) -> Vec<YoloDetection> {
        let state = self.state.read().await;
        let mut filtered = Vec::new();

        for detection in detections {
            // 检查类别是否被选中
            if !state.selected_classes.contains(&detection.class_id) {
                continue;
            }

            // 检查置信度阈值
            let threshold = self.thresholds.get_threshold(&detection.class_name).await;
            if detection.confidence >= threshold {
                filtered.push(detection);
            }
        }

        filtered
    }

    // 简化版本的摄像头和视频功能（需要OpenCV支持）
    pub async fn start_camera(&self, _device_id: i32) -> Result<()> {
        Err(anyhow::anyhow!(
            "摄像头实时检测功能需要OpenCV支持。\n\
            要启用此功能，请：\n\
            1. 安装OpenCV: brew install opencv (macOS) 或 apt install libopencv-dev (Ubuntu)\n\
            2. 使用 --features opencv-support 编译项目\n\
            3. 或者切换到Python版本获得完整功能"
        ))
    }

    pub async fn start_video(&self, _video_path: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "视频文件检测功能需要OpenCV支持。\n\
            要启用此功能，请：\n\
            1. 安装OpenCV: brew install opencv (macOS) 或 apt install libopencv-dev (Ubuntu)\n\
            2. 使用 --features opencv-support 编译项目\n\
            3. 或者切换到Python版本获得完整功能"
        ))
    }

    pub async fn get_next_frame(&self) -> Result<Option<DetectionResult>> {
        // 简化版本不支持实时帧流
        Ok(None)
    }
}