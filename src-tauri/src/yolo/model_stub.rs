use anyhow::Result;
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
    model_path: String,
    class_names: HashMap<u32, String>,
}

impl YoloModel {
    pub fn new(model_path: &str) -> Result<Self> {
        // 模拟YOLO模型初始化
        let mut class_names = HashMap::new();
        class_names.insert(0, "异常".to_string());
        class_names.insert(1, "正常".to_string());

        Ok(Self {
            model_path: model_path.to_string(),
            class_names,
        })
    }

    pub fn get_class_names(&self) -> &HashMap<u32, String> {
        &self.class_names
    }

    pub fn get_input_size(&self) -> (usize, usize) {
        (640, 640)
    }

    pub async fn detect_image(&self, _image_data: &[u8]) -> Result<Vec<YoloDetection>> {
        // 模拟检测结果 - 实际实现需要ONNX Runtime
        let mock_detection = YoloDetection {
            class_id: 1,
            class_name: "正常".to_string(),
            confidence: 0.85,
            bbox: [100.0, 100.0, 200.0, 150.0],
        };

        Ok(vec![mock_detection])
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