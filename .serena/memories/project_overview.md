# YOLO检测系统 - 项目概览

## 项目目的
基于Tauri + React + Rust构建的现代化YOLOv8实时目标检测桌面应用程序。从Python PyQt5版本迁移到现代化技术栈，提供更好的用户体验和性能。

## 技术栈

### 前端
- **框架**: React 19 + TypeScript
- **UI库**: shadcn/ui + Magic UI组件
- **样式**: Tailwind CSS v3.4.0
- **构建**: Vite v6.0.1
- **图标**: Lucide React

### 后端  
- **框架**: Tauri 2.0
- **语言**: Rust (Edition 2021)
- **AI框架**: Candle Framework (HuggingFace)
- **异步**: Tokio + futures
- **图像处理**: image crate + imageproc

### AI/ML组件
- **推理引擎**: Candle Core v0.9
- **模型格式**: ONNX (candle-onnx v0.9)
- **神经网络**: candle-nn v0.9
- **变换器**: candle-transformers v0.9

## 项目结构
```
yolo-detection-system/
├── src/                     # React前端
│   ├── components/ui/       # shadcn/ui + Magic UI组件
│   ├── YoloApp.tsx         # 主应用组件
│   └── App.tsx             # 应用入口
├── src-tauri/              # Tauri Rust后端
│   ├── src/
│   │   ├── yolo/           # YOLO检测模块
│   │   ├── main.rs         # 主程序入口
│   │   └── yolo_api.rs     # API接口层
│   └── Cargo.toml          # Rust依赖管理
├── models/                 # AI模型文件目录
├── scripts/                # 构建和开发脚本
└── package.json            # 前端依赖管理
```

## 核心功能实现状态

### ✅ 已实现
1. **图像检测**: 完整的单张图片检测流程
2. **用户界面**: 现代化React + Magic UI界面
3. **参数控制**: 置信度阈值调节、类别选择
4. **Tauri集成**: 完整的前后端API通信
5. **异步架构**: 基于Tokio的异步处理
6. **错误处理**: 完整的错误处理和用户反馈

### ⚠️ 部分实现
1. **摄像头检测**: 代码已实现但需要OpenCV支持
2. **视频检测**: 代码已实现但需要OpenCV支持
3. **模型推理**: 使用增强模拟，需要真实Candle模型

### ❌ 未实现
1. **真实模型加载**: 需要将PyTorch模型转换为Candle格式
2. **GPU加速**: 需要配置CUDA支持
3. **实时视频流**: 需要OpenCV集成

## 代码质量特点

### 优势
- **类型安全**: 全面的TypeScript类型定义
- **内存安全**: Rust零成本抽象和所有权系统
- **模块化设计**: 清晰的模块分离和职责划分
- **现代UI**: 使用最新的UI组件和设计系统
- **异步处理**: 高效的并发处理能力

### 架构设计
- **前后端分离**: 清晰的API边界
- **组件化**: 模块化的UI和业务逻辑组件
- **状态管理**: 统一的React状态管理
- **错误处理**: 分层的错误处理机制