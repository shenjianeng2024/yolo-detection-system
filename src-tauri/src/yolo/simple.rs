/*!
简化YOLO检测模块实现

暂时使用模拟实现，保持与原API兼容，后续可升级为真实ONNX推理
*/

use anyhow::{Result, anyhow};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use image::DynamicImage;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
};
use tokio::sync::RwLock;

/// YOLO检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloDetection {
    pub class_id: i32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [f32; 4], // [x, y, width, height]
}

/// 检测结果包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub detections: Vec<YoloDetection>,
    pub frame_data: Option<String>, // base64编码的图像数据
    pub timestamp: DateTime<Utc>,
}

/// 检测状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionState {
    pub is_running: bool,
    pub current_source: Option<InputSource>,
    pub results: Vec<DetectionResult>,
    pub selected_classes: Vec<i32>,
}

/// 输入源类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputSource {
    Image { path: String },
    Video { path: String },
    Camera { device_id: i32 },
}

/// YOLO检测器管理器 (简化实现)
pub struct YoloManager {
    model_initialized: bool,
    class_names: Vec<String>,
    confidence_thresholds: HashMap<String, f32>,
    selected_classes: Vec<i32>,
    detection_state: std::sync::Arc<RwLock<DetectionState>>,
    model_path: Option<PathBuf>,
}

impl YoloManager {
    /// 创建新的YOLO管理器
    pub fn new() -> Self {
        // 尝试从models/class_names.txt读取类别名称
        let class_names = Self::load_class_names()
            .unwrap_or_else(|_| vec!["异常".to_string(), "正常".to_string()]);
        
        let confidence_thresholds = {
            let mut map = HashMap::new();
            for (i, class_name) in class_names.iter().enumerate() {
                map.insert(class_name.clone(), if i == 0 { 0.7 } else { 0.5 });
            }
            map
        };
        
        let selected_classes: Vec<i32> = (0..class_names.len() as i32).collect();

        Self {
            model_initialized: false,
            class_names,
            confidence_thresholds,
            selected_classes: selected_classes.clone(),
            detection_state: std::sync::Arc::new(RwLock::new(DetectionState {
                is_running: false,
                current_source: None,
                results: Vec::new(),
                selected_classes,
            })),
            model_path: None,
        }
    }

    /// 加载类别名称
    fn load_class_names() -> Result<Vec<String>> {
        let class_names_path = Path::new("models/class_names.txt");
        if !class_names_path.exists() {
            return Err(anyhow!("类别名称文件不存在: {}", class_names_path.display()));
        }

        let content = fs::read_to_string(class_names_path)?;
        let class_names: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        if class_names.is_empty() {
            return Err(anyhow!("类别名称文件为空"));
        }

        Ok(class_names)
    }

    /// 初始化YOLO模型
    pub async fn init_model(&mut self, model_path: &str) -> Result<()> {
        let model_path = Path::new(model_path);
        
        if !model_path.exists() {
            return Err(anyhow!("模型文件不存在: {}", model_path.display()));
        }

        println!("正在加载YOLO模型: {}", model_path.display());
        
        // 简化实现：只检查文件存在即可
        self.model_initialized = true;
        self.model_path = Some(model_path.to_path_buf());
        
        println!("YOLO模型初始化成功 (简化模式)");
        Ok(())
    }

    /// 处理图像检测
    pub async fn process_image(&mut self, image_path: &str) -> Result<DetectionResult> {
        if !self.model_initialized {
            return Err(anyhow!("模型未初始化"));
        }

        let image_path = Path::new(image_path);
        if !image_path.exists() {
            return Err(anyhow!("图像文件不存在: {}", image_path.display()));
        }

        // 读取图像
        let img = image::open(image_path)
            .map_err(|e| anyhow!("无法读取图像 {}: {:?}", image_path.display(), e))?;

        // 生成基于真实图像尺寸的检测结果
        let detections = self.generate_realistic_detections(&img).await?;

        // 转换图像为base64
        let frame_data = self.image_to_base64(&img).await?;

        let result = DetectionResult {
            detections,
            frame_data: Some(frame_data),
            timestamp: Utc::now(),
        };

        // 更新状态
        let mut state = self.detection_state.write().await;
        state.current_source = Some(InputSource::Image { 
            path: image_path.to_string_lossy().to_string()
        });
        state.results.push(result.clone());
        
        // 保持结果数量不超过100个
        if state.results.len() > 100 {
            let len = state.results.len();
            state.results.drain(0..len - 100);
        }

        println!("图像处理完成，检测到 {} 个对象", result.detections.len());
        Ok(result)
    }

    /// 生成基于图像的现实检测结果
    async fn generate_realistic_detections(&self, img: &DynamicImage) -> Result<Vec<YoloDetection>> {
        let mut results = Vec::new();
        let (width, height) = (img.width() as f32, img.height() as f32);
        
        // 根据图像尺寸生成合理的检测框
        let detections_data = [
            ("异常", 0.85, (0.1, 0.2, 0.3, 0.4)),  // 相对坐标
            ("正常", 0.92, (0.5, 0.3, 0.35, 0.25)),
            ("异常", 0.76, (0.05, 0.05, 0.2, 0.3)),
        ];

        for (class_name, confidence, (rel_x, rel_y, rel_w, rel_h)) in &detections_data {
            let class_id = if *class_name == "异常" { 0 } else { 1 };
            
            // 检查类别是否被选中
            if !self.selected_classes.contains(&class_id) {
                continue;
            }
            
            // 检查置信度阈值
            let threshold = self.confidence_thresholds
                .get(&class_name.to_string())
                .unwrap_or(&0.5);
            
            if confidence >= threshold {
                // 转换为绝对坐标
                let bbox = [
                    rel_x * width,
                    rel_y * height,
                    rel_w * width,
                    rel_h * height,
                ];
                
                results.push(YoloDetection {
                    class_id,
                    class_name: class_name.to_string(),
                    confidence: *confidence,
                    bbox,
                });
            }
        }

        Ok(results)
    }

    /// 将图像转换为base64字符串
    async fn image_to_base64(&self, img: &DynamicImage) -> Result<String> {
        let mut buffer = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buffer);
        
        img.write_to(&mut cursor, image::ImageFormat::Jpeg)
            .map_err(|e| anyhow!("图像编码失败: {:?}", e))?;
        
        Ok(BASE64_STANDARD.encode(buffer))
    }

    /// 更新置信度阈值
    pub async fn update_confidence_threshold(&mut self, class_name: &str, threshold: f32) -> Result<()> {
        if !self.class_names.contains(&class_name.to_string()) {
            return Err(anyhow!("未知类别: {}", class_name));
        }
        
        self.confidence_thresholds.insert(class_name.to_string(), threshold);
        println!("更新置信度阈值: {} -> {}", class_name, threshold);
        Ok(())
    }

    /// 设置选中的类别
    pub async fn set_selected_classes(&mut self, class_ids: Vec<i32>) -> Result<()> {
        self.selected_classes = class_ids.clone();
        
        let mut state = self.detection_state.write().await;
        state.selected_classes = class_ids;
        
        println!("更新选中类别: {:?}", self.selected_classes);
        Ok(())
    }

    /// 获取检测状态
    pub async fn get_detection_state(&self) -> DetectionState {
        self.detection_state.read().await.clone()
    }

    /// 停止检测
    pub async fn stop_detection(&mut self) -> Result<()> {
        let mut state = self.detection_state.write().await;
        state.is_running = false;
        state.current_source = None;
        
        println!("检测已停止");
        Ok(())
    }

    /// 获取类别名称列表
    pub fn get_class_names(&self) -> &Vec<String> {
        &self.class_names
    }

    /// 检查模型是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.model_initialized
    }
}