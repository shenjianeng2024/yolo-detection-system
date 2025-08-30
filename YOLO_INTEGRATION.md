# YOLO 模型集成指南

## 🎯 YOLO 模型加载方案

### 方案 1：Python 后端（推荐，简单易用）

这是目前实现的方案，通过 Rust 调用 Python 脚本来使用 ultralytics 库。

#### 环境准备
```bash
# 安装 Python 依赖
pip install ultralytics opencv-python torch torchvision

# 下载预训练模型
python -c "from ultralytics import YOLO; YOLO('yolov8n.pt')"
```

#### 使用方法
1. 将训练好的模型文件（.pt 格式）放在项目目录
2. 在应用中调用 `load_yolo_model` 命令加载模型
3. 选择图像后调用 `detect_image_file` 进行检测

```javascript
// 加载模型
await invoke('load_yolo_model', { modelPath: 'best.pt' })

// 检测图像
const results = await invoke('detect_image_file', { imagePath: '/path/to/image.jpg' })
```

### 方案 2：ONNX Runtime（高性能）

适合生产环境，需要将 PyTorch 模型转换为 ONNX 格式。

#### 模型转换
```python
from ultralytics import YOLO

# 加载训练好的模型
model = YOLO('best.pt')

# 导出为 ONNX 格式
model.export(format='onnx', dynamic=True, simplify=True)
```

#### Rust 依赖
```toml
[dependencies]
ort = "2.0"
image = "0.25"
ndarray = "0.15"
```

#### 启用 ONNX 支持
```bash
cargo build --features onnx-runtime
```

### 方案 3：Candle 框架（纯 Rust）

使用 Hugging Face 的 Candle 框架，完全 Rust 实现。

#### 启用 Candle 支持
```bash
cargo build --features candle
```

## 🔧 模型格式支持

| 格式 | 说明 | 优势 | 劣势 |
|------|------|------|------|
| .pt (PyTorch) | Python 后端 | 易用，功能完整 | 需要 Python 环境 |
| .onnx | ONNX Runtime | 高性能，跨平台 | 需要模型转换 |
| .safetensors | Candle | 纯 Rust，安全 | 实现复杂 |

## 🚀 使用步骤

### 1. 准备模型文件

#### 使用预训练模型
```bash
# YOLOv8n (最小模型，快速)
wget https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8n.pt

# YOLOv8s (小模型，平衡)
wget https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8s.pt

# YOLOv8m (中等模型，精度更高)
wget https://github.com/ultralytics/assets/releases/download/v8.2.0/yolov8m.pt
```

#### 使用自训练模型
将您训练好的 `best.pt` 文件放在项目根目录。

### 2. 安装 Python 环境
```bash
# 使用 conda
conda create -n yolo python=3.9
conda activate yolo
pip install ultralytics

# 或使用 pip
pip install ultralytics opencv-python torch torchvision
```

### 3. 运行应用
```bash
# 开发模式
pnpm tauri:dev

# 或使用脚本
./scripts/dev.sh
```

### 4. 在应用中使用
1. 点击"选择图片"或"选择视频"按钮
2. 应用会自动尝试加载 `best.pt` 模型
3. 点击"开始检测"进行目标检测
4. 检测结果会实时显示在右侧面板

## 📊 性能优化

### Python 后端优化
```python
# 在 yolo.rs 中的 Python 脚本优化
import torch
torch.backends.cudnn.benchmark = True  # 启用 cuDNN 优化

# 使用 GPU 加速
device = 'cuda' if torch.cuda.is_available() else 'cpu'
model = YOLO('best.pt').to(device)
```

### 推理优化
- **图像预处理**：调整图像尺寸以匹配模型输入
- **批处理**：一次处理多张图像
- **模型量化**：使用 INT8 量化减少内存占用
- **TensorRT**：NVIDIA GPU 上的极致优化

## 🎨 自定义类别

如果使用自训练模型，需要更新类别列表：

```rust
// 在 yolo.rs 中更新 class_names
let class_names = vec![
    "your_class_1".to_string(),
    "your_class_2".to_string(),
    "your_class_3".to_string(),
    // ... 更多自定义类别
];
```

## 🔍 调试和故障排除

### 常见问题

1. **模型文件未找到**
   ```
   Error: Model file not found: best.pt
   ```
   解决：确保模型文件路径正确

2. **Python 环境问题**
   ```
   Error: Python detection failed: ModuleNotFoundError: No module named 'ultralytics'
   ```
   解决：安装 ultralytics 库

3. **CUDA 内存不足**
   ```
   Error: CUDA out of memory
   ```
   解决：使用 CPU 推理或减小输入图像尺寸

### 日志调试
在开发模式下，检查控制台输出获取详细错误信息：

```bash
# 查看 Tauri 日志
pnpm tauri:dev

# 查看 Python 脚本输出
tail -f /tmp/yolo_detect.py
```

## 📈 扩展功能

### 实时视频检测
目前支持静态图像检测，可扩展为：
- 摄像头实时检测
- 视频文件逐帧分析
- 批量图像处理

### 结果导出
- CSV 格式检测报告
- JSON 格式结构化数据
- 可视化结果图像保存

### 模型管理
- 多模型切换
- 模型下载管理器
- 性能基准测试

这个集成方案提供了灵活的 YOLO 模型加载和推理能力，可以根据实际需求选择最适合的方案。