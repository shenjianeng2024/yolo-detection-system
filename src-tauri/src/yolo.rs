use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YoloDetection {
    pub class_id: u32,
    pub class_name: String,
    pub confidence: f32,
    pub bbox: [f32; 4], // [x, y, width, height]
}

#[derive(Debug)]
pub enum YoloBackend {
    Python,      // 使用 Python ultralytics 库
    #[cfg(feature = "onnx-runtime")]
    OnnxRuntime, // 使用 ONNX Runtime
    #[cfg(feature = "candle")]
    Candle,      // 使用 Candle 框架
}

pub struct YoloModel {
    backend: YoloBackend,
    model_path: String,
    class_names: Vec<String>,
}

impl YoloModel {
    pub fn new(model_path: &str, backend: YoloBackend) -> Result<Self, Box<dyn std::error::Error>> {
        // COCO 数据集的默认类别名称
        let class_names = vec![
            "person".to_string(), "bicycle".to_string(), "car".to_string(), 
            "motorcycle".to_string(), "airplane".to_string(), "bus".to_string(), 
            "train".to_string(), "truck".to_string(), "boat".to_string(), 
            "traffic light".to_string(), "fire hydrant".to_string(), "stop sign".to_string(),
            "parking meter".to_string(), "bench".to_string(), "bird".to_string(),
            "cat".to_string(), "dog".to_string(), "horse".to_string(),
            "sheep".to_string(), "cow".to_string(), "elephant".to_string(),
            "bear".to_string(), "zebra".to_string(), "giraffe".to_string(),
            // ... 更多类别
        ];

        Ok(YoloModel {
            backend,
            model_path: model_path.to_string(),
            class_names,
        })
    }

    pub fn get_class_names(&self) -> &[String] {
        &self.class_names
    }

    pub async fn detect_image(&self, image_path: &str) -> Result<Vec<YoloDetection>, Box<dyn std::error::Error + Send + Sync>> {
        match self.backend {
            YoloBackend::Python => self.detect_with_python(image_path).await,
            #[cfg(feature = "onnx-runtime")]
            YoloBackend::OnnxRuntime => self.detect_with_onnx(image_path).await,
            #[cfg(feature = "candle")]
            YoloBackend::Candle => self.detect_with_candle(image_path).await,
        }
    }

    async fn detect_with_python(&self, image_path: &str) -> Result<Vec<YoloDetection>, Box<dyn std::error::Error + Send + Sync>> {
        // 创建临时 Python 脚本
        let script_content = format!(r#"
import sys
import json
from ultralytics import YOLO
import cv2

try:
    # 加载模型
    model = YOLO('{}')
    
    # 运行推理
    results = model('{}')
    
    detections = []
    for result in results:
        boxes = result.boxes
        if boxes is not None:
            for box in boxes:
                x1, y1, x2, y2 = box.xyxy[0].tolist()
                conf = float(box.conf[0])
                cls = int(box.cls[0])
                class_name = model.names[cls]
                
                detection = {{
                    "class_id": cls,
                    "class_name": class_name,
                    "confidence": conf,
                    "bbox": [x1, y1, x2-x1, y2-y1]
                }}
                detections.append(detection)
    
    print(json.dumps(detections))
    
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#, self.model_path, image_path);

        // 写入临时文件
        let temp_script = "/tmp/yolo_detect.py";
        tokio::fs::write(temp_script, script_content).await?;

        // 执行 Python 脚本
        let output = Command::new("python3")
            .arg(temp_script)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Python detection failed: {}", error).into());
        }

        // 解析 JSON 结果
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detections: Vec<YoloDetection> = serde_json::from_str(&stdout)?;
        
        // 清理临时文件
        let _ = tokio::fs::remove_file(temp_script).await;
        
        Ok(detections)
    }

    #[cfg(feature = "onnx-runtime")]
    async fn detect_with_onnx(&self, image_path: &str) -> Result<Vec<YoloDetection>, Box<dyn std::error::Error + Send + Sync>> {
        // ONNX Runtime 实现
        // 这需要预处理图像、运行推理、后处理结果
        todo!("ONNX Runtime implementation")
    }

    #[cfg(feature = "candle")]
    async fn detect_with_candle(&self, image_path: &str) -> Result<Vec<YoloDetection>, Box<dyn std::error::Error + Send + Sync>> {
        // Candle 实现
        todo!("Candle implementation")
    }
}

// 实用函数：检查模型文件是否存在
pub fn check_model_file(model_path: &str) -> bool {
    Path::new(model_path).exists()
}

// 实用函数：下载预训练模型
pub async fn download_model(model_name: &str, save_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let url = match model_name {
        "yolov8n" => "https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8n.pt",
        "yolov8s" => "https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8s.pt",
        "yolov8m" => "https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8m.pt",
        "yolov8l" => "https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8l.pt",
        "yolov8x" => "https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8x.pt",
        _ => return Err("Unknown model name".into()),
    };
    
    // 使用 reqwest 下载文件
    // 这里简化实现，实际应该有进度回调
    println!("Downloading model from: {}", url);
    println!("Save to: {}", save_path);
    
    Ok(())
}