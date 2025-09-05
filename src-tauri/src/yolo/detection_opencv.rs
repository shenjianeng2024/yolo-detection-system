use anyhow::{Context, Result};
use opencv::{
    core::{Mat, Vector},
    imgcodecs::{imread, IMREAD_COLOR},
    imgproc::{cvt_color, resize, COLOR_BGR2RGB, INTER_LINEAR},
    prelude::*,
    videoio::{VideoCapture, CAP_ANY},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use super::{YoloDetection, CandleYoloModel as YoloModel, ConfidenceThresholds};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Image { path: String },
    Camera { device_id: i32 },
    Video { path: String },
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

struct FrameProcessor {
    model: Arc<YoloModel>,
    thresholds: Arc<ConfidenceThresholds>,
    selected_classes: Vec<u32>,
}

pub struct YoloDetectionEngine {
    model: Arc<YoloModel>,
    thresholds: Arc<ConfidenceThresholds>,
    state: Arc<RwLock<DetectionState>>,
    frame_sender: Option<mpsc::UnboundedSender<DetectionResult>>,
    stop_signal: Arc<RwLock<bool>>,
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
            frame_sender: None,
            stop_signal: Arc::new(RwLock::new(false)),
        })
    }

    pub async fn process_image(&self, image_path: &str) -> Result<DetectionResult> {
        // 检查文件是否存在
        if !Path::new(image_path).exists() {
            return Err(anyhow::anyhow!("Image file does not exist: {}", image_path));
        }

        // 使用OpenCV读取图像
        let image = imread(image_path, IMREAD_COLOR)
            .context("Failed to load image with OpenCV")?;
        
        if image.empty() {
            return Err(anyhow::anyhow!("Failed to load image: empty image"));
        }

        // 转换为RGB格式
        let mut rgb_image = Mat::default();
        cvt_color(&image, &mut rgb_image, COLOR_BGR2RGB, 0)?;

        // 将Mat转换为字节数组
        let image_data = self.mat_to_bytes(&rgb_image)?;

        // 运行检测
        let detections = self.model.detect_image(&image_data).await?;

        // 过滤检测结果
        let filtered_detections = self.filter_detections(detections).await;

        // 将原始图像转换为base64
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

    pub async fn start_camera(&self, device_id: i32) -> Result<()> {
        // 设置运行状态
        {
            let mut state = self.state.write().await;
            if state.is_running {
                return Err(anyhow::anyhow!("Detection is already running"));
            }
            state.is_running = true;
            state.current_source = Some(InputSource::Camera { device_id });
        }

        // 重置停止信号
        *self.stop_signal.write().await = false;

        // 启动摄像头处理任务
        let model = self.model.clone();
        let thresholds = self.thresholds.clone();
        let state = self.state.clone();
        let stop_signal = self.stop_signal.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::camera_processing_loop(device_id, model, thresholds, state, stop_signal).await {
                eprintln!("Camera processing error: {}", e);
            }
        });

        Ok(())
    }

    pub async fn start_video(&self, video_path: &str) -> Result<()> {
        // 检查文件是否存在
        if !Path::new(video_path).exists() {
            return Err(anyhow::anyhow!("Video file does not exist: {}", video_path));
        }

        // 设置运行状态
        {
            let mut state = self.state.write().await;
            if state.is_running {
                return Err(anyhow::anyhow!("Detection is already running"));
            }
            state.is_running = true;
            state.current_source = Some(InputSource::Video { path: video_path.to_string() });
        }

        // 重置停止信号
        *self.stop_signal.write().await = false;

        // 启动视频处理任务
        let model = self.model.clone();
        let thresholds = self.thresholds.clone();
        let state = self.state.clone();
        let stop_signal = self.stop_signal.clone();
        let video_path = video_path.to_string();

        tokio::spawn(async move {
            if let Err(e) = Self::video_processing_loop(video_path, model, thresholds, state, stop_signal).await {
                eprintln!("Video processing error: {}", e);
            }
        });

        Ok(())
    }

    pub async fn stop_detection(&self) -> Result<()> {
        // 设置停止信号
        *self.stop_signal.write().await = true;

        // 更新状态
        let mut state = self.state.write().await;
        state.is_running = false;
        state.current_source = None;
        
        Ok(())
    }

    pub async fn get_next_frame(&self) -> Result<Option<DetectionResult>> {
        // 从结果队列中获取最新的检测结果
        let state = self.state.read().await;
        Ok(state.results.last().cloned())
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

    // 私有方法：摄像头处理循环
    async fn camera_processing_loop(
        device_id: i32,
        model: Arc<YoloModel>,
        thresholds: Arc<ConfidenceThresholds>,
        state: Arc<RwLock<DetectionState>>,
        stop_signal: Arc<RwLock<bool>>,
    ) -> Result<()> {
        let mut cap = VideoCapture::new(device_id, CAP_ANY)?;
        
        if !cap.is_opened()? {
            return Err(anyhow::anyhow!("Cannot open camera {}", device_id));
        }

        let mut frame = Mat::default();
        
        loop {
            // 检查停止信号
            if *stop_signal.read().await {
                break;
            }

            // 读取帧
            if !cap.read(&mut frame)? || frame.empty() {
                eprintln!("Failed to read frame from camera");
                tokio::time::sleep(tokio::time::Duration::from_millis(33)).await; // ~30fps
                continue;
            }

            // 处理帧
            if let Ok(result) = Self::process_frame(&frame, &model, &thresholds, &state).await {
                // 更新状态中的结果
                let mut state_lock = state.write().await;
                state_lock.results.push(result);
                
                // 保持结果队列大小合理（最多保留10个结果）
                if state_lock.results.len() > 10 {
                    state_lock.results.remove(0);
                }
            }

            // 控制帧率 (~30fps)
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }

        // 更新停止状态
        let mut state_lock = state.write().await;
        state_lock.is_running = false;
        
        Ok(())
    }

    // 私有方法：视频处理循环
    async fn video_processing_loop(
        video_path: String,
        model: Arc<YoloModel>,
        thresholds: Arc<ConfidenceThresholds>,
        state: Arc<RwLock<DetectionState>>,
        stop_signal: Arc<RwLock<bool>>,
    ) -> Result<()> {
        let mut cap = VideoCapture::from_file(&video_path, CAP_ANY)?;
        
        if !cap.is_opened()? {
            return Err(anyhow::anyhow!("Cannot open video file: {}", video_path));
        }

        let mut frame = Mat::default();
        
        loop {
            // 检查停止信号
            if *stop_signal.read().await {
                break;
            }

            // 读取帧
            if !cap.read(&mut frame)? {
                // 视频结束，重新开始播放
                cap = VideoCapture::from_file(&video_path, CAP_ANY)?;
                if !cap.is_opened()? {
                    break;
                }
                continue;
            }

            if frame.empty() {
                continue;
            }

            // 处理帧
            if let Ok(result) = Self::process_frame(&frame, &model, &thresholds, &state).await {
                // 更新状态中的结果
                let mut state_lock = state.write().await;
                state_lock.results.push(result);
                
                // 保持结果队列大小合理
                if state_lock.results.len() > 10 {
                    state_lock.results.remove(0);
                }
            }

            // 控制帧率 (~30fps)
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }

        // 更新停止状态
        let mut state_lock = state.write().await;
        state_lock.is_running = false;
        
        Ok(())
    }

    // 私有方法：处理单个帧
    async fn process_frame(
        frame: &Mat,
        model: &Arc<YoloModel>,
        thresholds: &Arc<ConfidenceThresholds>,
        state: &Arc<RwLock<DetectionState>>,
    ) -> Result<DetectionResult> {
        // 转换为RGB格式
        let mut rgb_frame = Mat::default();
        cvt_color(frame, &mut rgb_frame, COLOR_BGR2RGB, 0)?;

        // 将Mat转换为字节数组
        let image_data = Self::mat_to_bytes_static(&rgb_frame)?;

        // 运行检测
        let detections = model.detect_image(&image_data).await?;

        // 过滤检测结果
        let state_lock = state.read().await;
        let selected_classes = state_lock.selected_classes.clone();
        drop(state_lock);

        let mut filtered = Vec::new();
        for detection in detections {
            // 检查类别是否被选中
            if !selected_classes.contains(&detection.class_id) {
                continue;
            }

            // 检查置信度阈值
            let threshold = thresholds.get_threshold(&detection.class_name).await;
            if detection.confidence >= threshold {
                filtered.push(detection);
            }
        }

        // 将帧转换为base64
        use base64::Engine;
        let frame_base64 = base64::engine::general_purpose::STANDARD.encode(&image_data);

        Ok(DetectionResult {
            detections: filtered,
            frame_data: Some(frame_base64),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        })
    }

    // 工具方法：将Mat转换为字节数组
    fn mat_to_bytes(&self, mat: &Mat) -> Result<Vec<u8>> {
        Self::mat_to_bytes_static(mat)
    }

    fn mat_to_bytes_static(mat: &Mat) -> Result<Vec<u8>> {
        let rows = mat.rows();
        let cols = mat.cols();
        let channels = mat.channels();
        
        if channels != 3 {
            return Err(anyhow::anyhow!("Expected 3-channel image, got {}", channels));
        }

        let mut bytes = Vec::with_capacity((rows * cols * channels) as usize);
        
        for row in 0..rows {
            for col in 0..cols {
                let pixel = mat.at_2d::<opencv::core::Vec3b>(row, col)?;
                bytes.push(pixel[0]); // R
                bytes.push(pixel[1]); // G
                bytes.push(pixel[2]); // B
            }
        }
        
        Ok(bytes)
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
}