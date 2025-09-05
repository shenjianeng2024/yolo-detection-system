use anyhow::{Result, anyhow};
use candle_core::{Device, Tensor, DType};
use image::{GenericImageView, DynamicImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloDetection {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [f32; 4], // [x, y, width, height]
}

pub struct CandleYoloModel {
    device: Device,
    model_path: String,
    class_names: HashMap<u32, String>,
    input_size: (usize, usize),
}

impl CandleYoloModel {
    pub fn new(model_path: &str) -> Result<Self> {
        // 检查模型文件是否存在
        if !Path::new(model_path).exists() {
            return Err(anyhow!("Model file not found: {}", model_path));
        }

        // 初始化设备 (CPU first, GPU if available)
        let device = Device::Cpu;
        
        // 设置类别名称（从 Box.yaml 配置）
        let mut class_names = HashMap::new();
        class_names.insert(0, "异常".to_string());
        class_names.insert(1, "正常".to_string());

        Ok(Self {
            device,
            model_path: model_path.to_string(),
            class_names,
            input_size: (640, 640), // YOLOv8 标准输入尺寸
        })
    }

    pub fn get_class_names(&self) -> &HashMap<u32, String> {
        &self.class_names
    }

    pub fn get_input_size(&self) -> (usize, usize) {
        self.input_size
    }

    // 预处理图像数据
    fn preprocess_image(&self, image_data: &[u8]) -> Result<Tensor> {
        // 解码图像
        let img = image::load_from_memory(image_data)?;
        let rgb_img = img.to_rgb8();
        let (_orig_width, _orig_height) = rgb_img.dimensions();

        // 调整到模型输入尺寸
        let resized = image::imageops::resize(
            &rgb_img,
            self.input_size.0 as u32,
            self.input_size.1 as u32,
            image::imageops::FilterType::Lanczos3,
        );

        // 转换为张量格式 [1, 3, 640, 640]，归一化到 [0, 1]
        let mut tensor_data = Vec::with_capacity(3 * self.input_size.0 * self.input_size.1);
        
        // RGB 通道分离并归一化
        for c in 0..3 {
            for y in 0..self.input_size.1 {
                for x in 0..self.input_size.0 {
                    let pixel = resized.get_pixel(x as u32, y as u32);
                    let val = pixel[c] as f32 / 255.0;
                    tensor_data.push(val);
                }
            }
        }

        let tensor = Tensor::from_vec(
            tensor_data,
            &[1, 3, self.input_size.1, self.input_size.0],
            &self.device,
        )?;

        Ok(tensor)
    }

    // 后处理检测结果
    fn postprocess_detections(&self, output: &Tensor, confidence_threshold: f32) -> Result<Vec<YoloDetection>> {
        // YOLOv8 输出格式通常是 [1, 84, 8400] 对于2个类别
        // 其中 84 = 4 (bbox) + 2 (classes)
        let output_data = output.to_vec2::<f32>()?;
        let mut detections = Vec::new();

        if output_data.is_empty() {
            return Ok(detections);
        }

        // 解析检测结果
        let num_detections = output_data[0].len() / 6; // 假设每个检测有6个值 [x,y,w,h,conf_0,conf_1]
        
        for i in 0..num_detections {
            if i * 6 + 5 >= output_data[0].len() {
                break;
            }

            let x = output_data[0][i * 6];
            let y = output_data[0][i * 6 + 1];
            let w = output_data[0][i * 6 + 2];
            let h = output_data[0][i * 6 + 3];
            let conf_0 = output_data[0][i * 6 + 4]; // 异常
            let conf_1 = output_data[0][i * 6 + 5]; // 正常

            // 选择置信度最高的类别
            let (class_id, confidence) = if conf_0 > conf_1 {
                (0, conf_0)
            } else {
                (1, conf_1)
            };

            // 过滤低置信度检测
            if confidence >= confidence_threshold {
                let class_name = self.class_names.get(&class_id)
                    .unwrap_or(&"未知".to_string())
                    .clone();

                detections.push(YoloDetection {
                    class_id,
                    class_name,
                    confidence,
                    bbox: [x, y, w, h],
                });
            }
        }

        Ok(detections)
    }

    // 主要的图像检测方法
    pub async fn detect_image(&self, image_data: &[u8]) -> Result<Vec<YoloDetection>> {
        // 注意：由于我们目前有 PyTorch 模型(.pt)，但 Candle 需要特定格式
        // 这里先提供一个增强的模拟实现，带有真实的图像处理
        
        // 预处理图像（真实的图像处理）
        let _tensor = self.preprocess_image(image_data)?;
        
        // TODO: 当有 Candle 格式模型时，替换以下模拟逻辑
        // let output = self.model.forward(&tensor)?;
        // return self.postprocess_detections(&output, 0.5);

        // 临时的增强模拟 - 基于真实图像特征
        let img = image::load_from_memory(image_data)?;
        let (width, height) = img.dimensions();
        
        // 基于图像尺寸和内容生成更真实的检测结果
        let mut detections = Vec::new();
        
        // 模拟检测逻辑：大图可能有多个目标
        let num_objects = if width > 800 || height > 600 { 2 } else { 1 };
        
        for i in 0..num_objects {
            let class_id = if i % 2 == 0 { 1 } else { 0 }; // 交替正常/异常
            let confidence = 0.70 + (i as f32 * 0.1);
            let x = (width as f32 * 0.2) + (i as f32 * width as f32 * 0.3);
            let y = (height as f32 * 0.2) + (i as f32 * height as f32 * 0.2);
            let w = width as f32 * 0.25;
            let h = height as f32 * 0.3;

            let class_name = self.class_names.get(&class_id)
                .unwrap_or(&"未知".to_string())
                .clone();

            detections.push(YoloDetection {
                class_id,
                class_name,
                confidence,
                bbox: [x, y, w, h],
            });
        }

        Ok(detections)
    }

    // 检查模型文件状态
    pub fn get_model_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("model_path".to_string(), self.model_path.clone());
        info.insert("device".to_string(), format!("{:?}", self.device));
        info.insert("input_size".to_string(), format!("{:?}", self.input_size));
        info.insert("num_classes".to_string(), self.class_names.len().to_string());
        info
    }
}

// 置信度阈值管理 (保持与原来相同)
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ConfidenceThresholds {
    thresholds: Arc<RwLock<HashMap<String, f32>>>,
}

impl ConfidenceThresholds {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert("异常".to_string(), 0.7); // 异常检测阈值稍高
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