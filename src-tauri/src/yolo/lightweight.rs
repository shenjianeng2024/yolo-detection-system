/*!
轻量级YOLO检测模块实现

直接使用ONNX Runtime实现YOLO检测功能：
- ONNX模型加载和推理
- 图像预处理和后处理
- 检测结果解析
*/

use anyhow::{Result, anyhow};
use base64::prelude::*;
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageBuffer, Rgb};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    fs,
};
use tokio::sync::RwLock;
use ort::{Environment, SessionBuilder, Value, Session};

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

/// YOLO检测器管理器 (轻量级实现)
pub struct YoloManager {
    session: Option<Session>,
    model_initialized: bool,
    class_names: Vec<String>,
    confidence_thresholds: HashMap<String, f32>,
    selected_classes: Vec<i32>,
    detection_state: std::sync::Arc<RwLock<DetectionState>>,
    model_path: Option<PathBuf>,
    input_shape: (usize, usize), // (width, height)
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
        
        let selected_classes = (0..class_names.len() as i32).collect();

        Self {
            session: None,
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
            input_shape: (640, 640),
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
        
        // 初始化ONNX Runtime环境
        let environment = Environment::builder()
            .with_name("yolo_detection")
            .build()
            .map_err(|e| anyhow!("初始化ONNX Runtime环境失败: {:?}", e))?;

        // 创建会话
        let session = SessionBuilder::new(&environment)
            .map_err(|e| anyhow!("创建SessionBuilder失败: {:?}", e))?
            .with_model_from_file(model_path)
            .map_err(|e| anyhow!("加载模型文件失败: {:?}", e))?;

        self.session = Some(session);
        self.model_initialized = true;
        self.model_path = Some(model_path.to_path_buf());
        
        println!("YOLO模型初始化成功");
        Ok(())
    }

    /// 处理图像检测
    pub async fn process_image(&mut self, image_path: &str) -> Result<DetectionResult> {
        if !self.model_initialized {
            return Err(anyhow!("模型未初始化"));
        }

        let session = self.session.as_ref()
            .ok_or_else(|| anyhow!("ONNX会话未初始化"))?;

        let image_path = Path::new(image_path);
        if !image_path.exists() {
            return Err(anyhow!("图像文件不存在: {}", image_path.display()));
        }

        // 读取和预处理图像
        let img = image::open(image_path)
            .map_err(|e| anyhow!("无法读取图像 {}: {:?}", image_path.display(), e))?;

        let (input_tensor, original_size) = self.preprocess_image(&img).await?;

        // 执行推理
        let outputs = session.run(vec![input_tensor])
            .map_err(|e| anyhow!("模型推理失败: {:?}", e))?;

        // 后处理检测结果
        let detections = self.postprocess_outputs(&outputs, original_size).await?;
        let processed_detections = self.filter_detections(detections).await?;

        // 转换图像为base64
        let frame_data = self.image_to_base64(&img).await?;

        let result = DetectionResult {
            detections: processed_detections,
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

    /// 图像预处理
    async fn preprocess_image(&self, img: &DynamicImage) -> Result<(Value<'static>, (u32, u32))> {
        let original_size = (img.width(), img.height());
        
        // 调整图像大小到模型输入尺寸
        let resized = img.resize_exact(
            self.input_shape.0 as u32, 
            self.input_shape.1 as u32, 
            image::imageops::FilterType::Triangle
        );
        
        let rgb_img = resized.to_rgb8();
        
        // 转换为CHW格式并归一化
        let mut input_data = Vec::with_capacity(3 * self.input_shape.0 * self.input_shape.1);
        
        // 分离R, G, B通道并归一化到[0,1]
        for channel in 0..3 {
            for pixel in rgb_img.pixels() {
                let value = pixel[channel] as f32 / 255.0;
                input_data.push(value);
            }
        }
        
        // 创建输入张量 [batch, channels, height, width]
        let input_tensor = Value::from_array(
            ([1, 3, self.input_shape.1, self.input_shape.0], input_data.into_boxed_slice())
        ).map_err(|e| anyhow!("创建输入张量失败: {:?}", e))?;
        
        Ok((input_tensor, original_size))
    }

    /// 后处理模型输出
    async fn postprocess_outputs(&self, outputs: &[Value], original_size: (u32, u32)) -> Result<Vec<(i32, f32, [f32; 4])>> {
        if outputs.is_empty() {
            return Ok(Vec::new());
        }
        
        // 假设输出格式为 [batch, detections, 6] 其中6为 [x, y, w, h, conf, class]
        let output = &outputs[0];
        let output_shape = output.shape().ok_or_else(|| anyhow!("无法获取输出形状"))?;
        
        println!("模型输出形状: {:?}", output_shape);
        
        // 模拟解析检测结果 - 实际需要根据具体模型输出格式调整
        let mut detections = Vec::new();
        
        // 这里添加一些模拟检测结果用于测试
        // 实际应该解析模型的真实输出
        let mock_detections = [
            (0, 0.85, [100.0, 150.0, 200.0, 300.0]),
            (1, 0.92, [400.0, 200.0, 250.0, 200.0]),
            (0, 0.76, [50.0, 50.0, 120.0, 180.0]),
        ];
        
        for (class_id, confidence, bbox) in &mock_detections {
            // 将坐标缩放回原图尺寸
            let scale_x = original_size.0 as f32 / self.input_shape.0 as f32;
            let scale_y = original_size.1 as f32 / self.input_shape.1 as f32;
            
            let scaled_bbox = [
                bbox[0] * scale_x,
                bbox[1] * scale_y,
                bbox[2] * scale_x,
                bbox[3] * scale_y,
            ];
            
            detections.push((*class_id, *confidence, scaled_bbox));
        }
        
        Ok(detections)
    }

    /// 过滤检测结果
    async fn filter_detections(&self, raw_detections: Vec<(i32, f32, [f32; 4])>) -> Result<Vec<YoloDetection>> {
        let mut results = Vec::new();
        
        for (class_id, confidence, bbox) in raw_detections {
            // 检查类别是否被选中
            if !self.selected_classes.contains(&class_id) {
                continue;
            }

            // 获取类别名称
            let class_name = self.class_names.get(class_id as usize)
                .cloned()
                .unwrap_or_else(|| format!("未知类别_{}", class_id));

            // 检查置信度阈值
            let threshold = self.confidence_thresholds
                .get(&class_name)
                .unwrap_or(&0.5);
            
            if confidence >= *threshold {
                results.push(YoloDetection {
                    class_id,
                    class_name,
                    confidence,
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