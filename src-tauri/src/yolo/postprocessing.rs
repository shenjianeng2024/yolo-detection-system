use anyhow::Result;
use ort::value::Value;
use std::collections::HashMap;
use crate::yolo::YoloDetection;

pub fn postprocess_outputs(
    outputs: &[Value],
    class_names: &HashMap<u32, String>,
    original_size: (f32, f32),
    input_size: (usize, usize),
) -> Result<Vec<YoloDetection>> {
    if outputs.is_empty() {
        return Ok(Vec::new());
    }

    // 获取输出张量
    let output = &outputs[0];
    let output_data = output.try_extract::<f32>()?.view();
    let shape = output_data.shape();
    
    // YOLOv8 输出格式: [batch_size, num_classes + 4, num_anchors]
    // 其中前4个是坐标，后面是类别概率
    let mut detections = Vec::new();
    
    if shape.len() != 3 {
        return Ok(detections);
    }
    
    let num_classes = class_names.len();
    let num_boxes = shape[2];
    
    // 计算缩放因子
    let (orig_width, orig_height) = original_size;
    let scale_x = orig_width / input_size.0 as f32;
    let scale_y = orig_height / input_size.1 as f32;
    
    for i in 0..num_boxes {
        // 提取边界框坐标 (center_x, center_y, width, height)
        let center_x = output_data[[0, 0, i]];
        let center_y = output_data[[0, 1, i]];
        let width = output_data[[0, 2, i]];
        let height = output_data[[0, 3, i]];
        
        // 找到最高置信度的类别
        let mut max_confidence = 0.0;
        let mut best_class_id = 0;
        
        for class_id in 0..num_classes {
            let confidence = output_data[[0, 4 + class_id, i]];
            if confidence > max_confidence {
                max_confidence = confidence;
                best_class_id = class_id as u32;
            }
        }
        
        // 只保留置信度高于基本阈值的检测
        if max_confidence > 0.1 {
            // 转换坐标格式：center -> top-left corner
            let x = (center_x - width / 2.0) * scale_x;
            let y = (center_y - height / 2.0) * scale_y;
            let w = width * scale_x;
            let h = height * scale_y;
            
            let class_name = class_names
                .get(&best_class_id)
                .cloned()
                .unwrap_or_else(|| format!("class_{}", best_class_id));
            
            detections.push(YoloDetection {
                class_id: best_class_id,
                class_name,
                confidence: max_confidence,
                bbox: [x, y, w, h],
            });
        }
    }
    
    // 应用非最大抑制
    Ok(apply_nms(detections, 0.4)) // IoU阈值0.4
}

fn apply_nms(mut detections: Vec<YoloDetection>, iou_threshold: f32) -> Vec<YoloDetection> {
    if detections.is_empty() {
        return detections;
    }
    
    // 按置信度排序（降序）
    detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
    
    let mut keep = Vec::new();
    let mut suppressed = vec![false; detections.len()];
    
    for i in 0..detections.len() {
        if suppressed[i] {
            continue;
        }
        
        keep.push(detections[i].clone());
        
        // 计算与所有后续边界框的IoU
        for j in (i + 1)..detections.len() {
            if suppressed[j] {
                continue;
            }
            
            let iou = calculate_iou(&detections[i].bbox, &detections[j].bbox);
            if iou > iou_threshold {
                suppressed[j] = true;
            }
        }
    }
    
    keep
}

fn calculate_iou(box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
    let [x1, y1, w1, h1] = *box1;
    let [x2, y2, w2, h2] = *box2;
    
    // 计算交集
    let inter_x1 = x1.max(x2);
    let inter_y1 = y1.max(y2);
    let inter_x2 = (x1 + w1).min(x2 + w2);
    let inter_y2 = (y1 + h1).min(y2 + h2);
    
    let inter_width = (inter_x2 - inter_x1).max(0.0);
    let inter_height = (inter_y2 - inter_y1).max(0.0);
    let inter_area = inter_width * inter_height;
    
    // 计算并集
    let area1 = w1 * h1;
    let area2 = w2 * h2;
    let union_area = area1 + area2 - inter_area;
    
    if union_area <= 0.0 {
        0.0
    } else {
        inter_area / union_area
    }
}