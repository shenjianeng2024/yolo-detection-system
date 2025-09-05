/*!
YOLO ONNX 检测器 - 基于yolo-rs和ONNX运行时的实现
支持实时摄像头、视频文件和单张图片检测
*/

use anyhow::{anyhow, Result};
use image::{GenericImageView};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// YOLO检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub detections: Vec<Detection>,
    pub image_width: u32,
    pub image_height: u32,
    pub processing_time_ms: u64,
}

/// 单个检测框
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: BoundingBox,
}

/// 边界框
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// YOLO检测器状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionState {
    pub is_initialized: bool,
    pub model_path: Option<String>,
    pub class_names: Vec<String>,
    pub confidence_thresholds: HashMap<String, f32>,
    pub selected_classes: Vec<u32>,
    pub is_running: bool,
}

/// YOLO ONNX检测器
pub struct YoloOnnxDetector {
    /// 模型路径
    model_path: Option<String>,
    /// 类别名称映射
    class_names: Vec<String>,
    /// 置信度阈值设置
    confidence_thresholds: RwLock<HashMap<String, f32>>,
    /// 选中的检测类别
    selected_classes: RwLock<Vec<u32>>,
    /// 检测器状态
    state: RwLock<DetectionState>,
}

impl YoloOnnxDetector {
    /// 创建新的检测器实例
    pub fn new() -> Self {
        Self {
            model_path: None,
            class_names: Vec::new(),
            confidence_thresholds: RwLock::new(HashMap::new()),
            selected_classes: RwLock::new(Vec::new()),
            state: RwLock::new(DetectionState {
                is_initialized: false,
                model_path: None,
                class_names: Vec::new(),
                confidence_thresholds: HashMap::new(),
                selected_classes: Vec::new(),
                is_running: false,
            }),
        }
    }

    /// 初始化YOLO模型
    pub async fn init_model(&mut self, model_path: &str) -> Result<()> {
        // 处理相对路径，确保从正确的工作目录查找模型
        let model_path_obj = if Path::new(model_path).is_absolute() {
            Path::new(model_path).to_path_buf()
        } else {
            // 获取当前执行文件的目录作为基准
            let current_dir = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            current_dir.join(model_path)
        };
        
        println!("🔍 查找模型文件: {}", model_path_obj.display());
        
        if !model_path_obj.exists() {
            return Err(anyhow!("模型文件不存在: {}", model_path_obj.display()));
        }
        
        if model_path_obj.extension().unwrap_or_default() != "onnx" {
            return Err(anyhow!("只支持ONNX格式模型"));
        }

        println!("🔄 初始化YOLO模型: {}", model_path_obj.display());

        // 保存模型路径
        self.model_path = Some(model_path_obj.to_string_lossy().to_string());

        // 加载类别名称
        self.load_class_names(model_path_obj.parent().unwrap()).await?;

        // 初始化默认配置
        self.init_default_config().await?;

        // 更新状态
        let mut state = self.state.write().await;
        state.is_initialized = true;
        state.model_path = Some(model_path.to_string());
        state.class_names = self.class_names.clone();

        println!("✅ YOLO模型初始化成功 (模拟)");
        println!("📊 支持类别数量: {}", self.class_names.len());

        Ok(())
    }

    /// 加载类别名称
    async fn load_class_names(&mut self, model_dir: &Path) -> Result<()> {
        let class_names_file = model_dir.join("class_names.txt");
        
        if class_names_file.exists() {
            let content = tokio::fs::read_to_string(&class_names_file).await?;
            self.class_names = content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            
            println!("📄 从文件加载类别名称: {:?}", self.class_names);
        } else {
            // 使用默认的二分类类别
            self.class_names = vec!["异常".to_string(), "正常".to_string()];
            println!("⚠️  未找到类别文件，使用默认类别: {:?}", self.class_names);
        }

        Ok(())
    }

    /// 初始化默认配置
    async fn init_default_config(&self) -> Result<()> {
        let mut thresholds = self.confidence_thresholds.write().await;
        let mut selected = self.selected_classes.write().await;

        // 设置默认置信度阈值
        for class_name in &self.class_names {
            thresholds.insert(class_name.clone(), 0.5);
        }

        // 默认选择所有类别
        *selected = (0..self.class_names.len() as u32).collect();

        println!("⚙️  默认配置已加载");
        Ok(())
    }

    /// 处理单张图片
    pub async fn process_image(&mut self, image_path: &str) -> Result<DetectionResult> {
        if self.model_path.is_none() {
            return Err(anyhow!("模型未初始化"));
        }

        let start_time = std::time::Instant::now();

        // 加载图片
        let img = image::open(image_path)?;
        let (width, height) = img.dimensions();

        println!("🖼️  处理图片: {}x{}", width, height);

        // TODO: 实际的ONNX推理 - 目前返回模拟结果
        let detections = self.create_mock_detections(width, height).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        println!("✅ 检测完成 (模拟)，用时: {}ms，检测到 {} 个目标", 
                processing_time, detections.len());

        Ok(DetectionResult {
            detections,
            image_width: width,
            image_height: height,
            processing_time_ms: processing_time,
        })
    }

    /// 创建模拟检测结果 (临时实现)
    async fn create_mock_detections(&self, width: u32, height: u32) -> Result<Vec<Detection>> {
        let confidence_thresholds = self.confidence_thresholds.read().await;
        let selected_classes = self.selected_classes.read().await;

        let mut detections = Vec::new();

        // 模拟检测一些目标
        if !selected_classes.is_empty() && !self.class_names.is_empty() {
            // 模拟检测第一个选中的类别
            let class_id = selected_classes[0];
            let class_name = self.class_names
                .get(class_id as usize)
                .unwrap_or(&format!("class_{}", class_id))
                .clone();

            let threshold = confidence_thresholds
                .get(&class_name)
                .unwrap_or(&0.5);

            // 只在满足置信度阈值时添加模拟检测
            let mock_confidence = 0.85;
            if mock_confidence >= *threshold {
                detections.push(Detection {
                    class_id,
                    class_name,
                    confidence: mock_confidence,
                    bbox: BoundingBox {
                        x: width as f32 * 0.2,
                        y: height as f32 * 0.2,
                        width: width as f32 * 0.3,
                        height: height as f32 * 0.4,
                    },
                });
            }
        }

        Ok(detections)
    }

    /// 更新置信度阈值
    pub async fn update_confidence_threshold(&self, class_name: &str, threshold: f32) -> Result<()> {
        let mut thresholds = self.confidence_thresholds.write().await;
        thresholds.insert(class_name.to_string(), threshold.clamp(0.0, 1.0));
        
        // 更新状态
        let mut state = self.state.write().await;
        state.confidence_thresholds = thresholds.clone();
        
        println!("⚙️  更新 {} 的置信度阈值为: {:.2}", class_name, threshold);
        Ok(())
    }

    /// 设置选中的类别
    pub async fn set_selected_classes(&self, class_ids: Vec<u32>) -> Result<()> {
        let valid_ids: Vec<u32> = class_ids
            .into_iter()
            .filter(|&id| (id as usize) < self.class_names.len())
            .collect();

        let mut selected = self.selected_classes.write().await;
        *selected = valid_ids.clone();
        
        // 更新状态
        let mut state = self.state.write().await;
        state.selected_classes = valid_ids;

        println!("⚙️  更新选中的类别: {:?}", *selected);
        Ok(())
    }

    /// 获取检测器状态
    pub async fn get_detection_state(&self) -> DetectionState {
        let state = self.state.read().await;
        state.clone()
    }

    /// 获取类别名称列表
    pub fn get_class_names(&self) -> &Vec<String> {
        &self.class_names
    }

    /// 检查是否已初始化
    pub async fn is_initialized(&self) -> bool {
        let state = self.state.read().await;
        state.is_initialized
    }

    /// 开始实时检测（摄像头/视频）
    pub async fn start_detection(&self) -> Result<()> {
        if self.model_path.is_none() {
            return Err(anyhow!("模型未初始化"));
        }

        let mut state = self.state.write().await;
        state.is_running = true;
        
        println!("🎥 开始实时检测 (模拟)");
        // TODO: 实现实时检测逻辑
        Ok(())
    }

    /// 停止实时检测
    pub async fn stop_detection(&self) -> Result<()> {
        let mut state = self.state.write().await;
        state.is_running = false;
        
        println!("⏹️  停止实时检测");
        Ok(())
    }
}

impl Default for YoloOnnxDetector {
    fn default() -> Self {
        Self::new()
    }
}