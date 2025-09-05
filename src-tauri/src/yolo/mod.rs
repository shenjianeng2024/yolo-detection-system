/*!
YOLO检测模块

支持基于Candle框架的真实YOLO ONNX检测
*/

mod simple;
mod onnx_detector;
mod candle_detector;

// 重新导出Candle检测器作为主要实现
pub use candle_detector::*;

// 保留ONNX检测器以备兼容
#[allow(unused)]
pub use onnx_detector::{YoloOnnxDetector};

// 保留简化版本以备兼容
#[allow(unused)]
pub use simple::YoloManager;