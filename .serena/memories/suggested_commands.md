# YOLO检测系统 - 建议命令

## 开发和调试命令

### 启动开发环境
```bash
# 启动Tauri开发服务器（前端+后端）
pnpm tauri:dev

# 或使用项目脚本
pnpm start
# 实际执行: ./scripts/dev.sh
```

### 构建命令
```bash
# 构建前端
pnpm build
# 实际执行: tsc && vite build

# 构建完整应用程序
pnpm tauri:build

# 或使用项目脚本
pnpm build:app
# 实际执行: ./scripts/build.sh
```

### 代码质量检查
```bash
# 运行ESLint检查
pnpm lint
# 实际执行: eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0

# 预览构建结果
pnpm preview
# 实际执行: vite preview
```

## Rust后端专用命令

### Cargo命令（在src-tauri目录下）
```bash
cd src-tauri

# 编译Rust后端
cargo build

# 发布版本编译（优化）
cargo build --release

# 运行Rust测试
cargo test

# 检查代码
cargo check

# 格式化代码
cargo fmt

# 静态分析
cargo clippy
```

### Candle特定编译
```bash
# 编译基础版本（仅支持图片检测）
cargo build --manifest-path=src-tauri/Cargo.toml

# 编译GPU版本（如果支持CUDA）
cargo build --manifest-path=src-tauri/Cargo.toml --features cuda

# 编译完整版本（需要OpenCV，如果已安装）
cargo build --manifest-path=src-tauri/Cargo.toml --features opencv-support
```

## 系统依赖安装

### macOS环境
```bash
# 安装OpenCV（可选，用于摄像头和视频功能）
brew install opencv

# 设置环境变量（如果需要）
export DYLD_FALLBACK_LIBRARY_PATH="$(xcode-select --print-path)/Toolchains/XcodeDefault.xctoolchain/usr/lib/"
```

### Ubuntu环境
```bash
# 安装系统依赖
sudo apt update
sudo apt install libopencv-dev clang libclang-dev

# 安装Rust（如果未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 模型和资源管理

### 模型文件管理
```bash
# 创建模型目录
mkdir -p models

# 下载示例YOLO模型（需要替换为实际模型）
# wget -c https://huggingface.co/lmz/candle-yolo-v8/resolve/main/yolov8s.safetensors -O models/yolov8s.safetensors

# 检查模型文件
ls -la models/
```

### 测试和验证
```bash
# 运行手动测试
# 参考 test_manual.md 文件中的测试步骤

# 检查应用程序状态
pnpm tauri info
```

## 故障排除命令

### 清理和重置
```bash
# 清理前端依赖
rm -rf node_modules pnpm-lock.yaml
pnpm install

# 清理Rust构建缓存
cd src-tauri
cargo clean
cd ..

# 重新构建
pnpm tauri:build
```

### 日志和调试
```bash
# 启动时显示详细日志
RUST_LOG=debug pnpm tauri:dev

# 启用Tauri调试模式
pnpm tauri dev --debug
```

## Git工作流命令

### 分支管理
```bash
# 检查当前状态
git status
git branch

# 创建功能分支
git checkout -b feature/your-feature-name

# 提交更改
git add .
git commit -m "描述性的提交信息"

# 推送到远程
git push origin feature/your-feature-name
```

## 性能分析命令

### Rust性能分析
```bash
cd src-tauri

# 构建性能分析版本
cargo build --release --bin benchmark
# 注意：当前benchmark二进制被注释，需要启用

# 运行基准测试（如果可用）
cargo bench
```

### 前端性能分析
```bash
# 构建分析包大小
pnpm build

# 查看构建产物
ls -la dist/
```

## 重要注意事项

1. **优先使用项目脚本**: scripts/目录下的.sh脚本，而不是直接使用npm/cargo命令
2. **环境依赖**: OpenCV功能需要系统级依赖，可选安装
3. **模型格式**: 当前使用模拟检测，需要真实的Candle格式模型文件
4. **跨平台**: 某些命令在不同操作系统上可能有差异
5. **版本锁定**: 前端使用pnpm-lock.yaml，Rust使用Cargo.lock锁定依赖版本