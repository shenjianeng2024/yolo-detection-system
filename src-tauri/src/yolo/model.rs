use anyhow::{Context, Result};
use ort::{environment::Environment, execution_providers::ExecutionProvider, session::{Session, builder::SessionBuilder}, value::Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloDetection {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [f32; 4], // [x, y, width, height]
}

pub struct YoloModel {
    session: Arc<Session>,
    class_names: HashMap<u32, String>,
    input_width: usize,
    input_height: usize,
}

impl YoloModel {
    pub fn new(model_path: &str) -> Result<Self> {
        // 初始化ONNX Runtime环境
        let environment = Arc::new(
            Environment::builder()
                .with_name("YOLOv8")
                .build()
                .context("Failed to create ONNX Runtime environment")?
        );

        // 创建会话
        let session = SessionBuilder::new(&environment)?
            .with_execution_providers([ExecutionProvider::CPU(Default::default())])?
            .with_model_from_file(model_path)
            .context("Failed to load YOLO model")?;

        // 从资源文件读取类别名称（基于Python代码中的二分类）
        let mut class_names = HashMap::new();
        class_names.insert(0, "异常".to_string());
        class_names.insert(1, "正常".to_string());

        Ok(Self {
            session: Arc::new(session),
            class_names,
            input_width: 640,
            input_height: 640,
        })
    }

    pub fn get_class_names(&self) -> &HashMap<u32, String> {
        &self.class_names
    }

    pub fn get_input_size(&self) -> (usize, usize) {
        (self.input_width, self.input_height)
    }

    pub async fn detect_image(&self, image_data: &[u8]) -> Result<Vec<YoloDetection>> {
        // 在后台线程中运行推理以避免阻塞async运行时
        let session = Arc::clone(&self.session);
        let class_names = self.class_names.clone();
        let input_size = (self.input_width, self.input_height);
        let image_data = image_data.to_vec();

        tokio::task::spawn_blocking(move || {
            Self::run_inference(session, class_names, input_size, &image_data)
        })
        .await
        .context("Inference task failed")?
    }

    fn run_inference(
        session: Arc<Session>,
        class_names: HashMap<u32, String>,
        input_size: (usize, usize),
        image_data: &[u8],
    ) -> Result<Vec<YoloDetection>> {
        // 1. 加载并预处理图像
        let img = image::load_from_memory(image_data)
            .context("Failed to load image")?
            .to_rgb8();
        
        let (original_width, original_height) = (img.width(), img.height());
        
        // 2. 预处理：调整大小并规范化
        let processed_image = crate::yolo::preprocessing::preprocess_image(
            &img, 
            input_size.0, 
            input_size.1
        )?;

        // 3. 创建输入张量
        let input_tensor = Value::from_array(session.allocator(), &processed_image)?;

        // 4. 运行推理
        let outputs = session.run(vec![input_tensor])?;
        
        // 5. 后处理
        let detections = crate::yolo::postprocessing::postprocess_outputs(
            &outputs,
            &class_names,
            (original_width as f32, original_height as f32),
            input_size,
        )?;

        Ok(detections)
    }
}

// 置信度阈值管理
pub struct ConfidenceThresholds {
    thresholds: Arc<RwLock<HashMap<String, f32>>>,
}

impl ConfidenceThresholds {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("异常".to_string(), 0.5);
        thresholds.insert("正常".to_string(), 0.5);
        
        Self {
            thresholds: Arc::new(RwLock::new(thresholds)),
        }
    }

    pub async fn update_threshold(&self, class_name: &str, threshold: f32) {
        let mut thresholds = self.thresholds.write().await;
        thresholds.insert(class_name.to_string(), threshold);
    }

    pub async fn get_threshold(&self, class_name: &str) -> f32 {
        let thresholds = self.thresholds.read().await;
        thresholds.get(class_name).copied().unwrap_or(0.5)
    }

    pub async fn get_all_thresholds(&self) -> HashMap<String, f32> {
        let thresholds = self.thresholds.read().await;
        thresholds.clone()
    }
}