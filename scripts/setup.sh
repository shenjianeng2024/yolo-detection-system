#!/bin/bash
# YOLO检测系统环境设置脚本

set -e

echo "🔧 YOLO检测系统环境设置"
echo "========================"

# 检查必要的工具
check_command() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ 未找到 $1，请先安装"
        exit 1
    else
        echo "✅ 找到 $1"
    fi
}

echo "检查必要工具..."
check_command "python3"
check_command "pip"
check_command "cargo"
check_command "pnpm"

# 安装Python依赖
echo "📦 安装Python依赖..."
pip install ultralytics torch onnx onnxruntime opencv-python

# 创建模型目录
echo "📁 创建模型目录..."
mkdir -p models

# 转换模型（如果存在best.pt）
if [ -f "resource/best.pt" ]; then
    echo "🔄 转换YOLO模型..."
    python3 scripts/convert_model.py -i resource/best.pt -o models/best.onnx --validate
else
    echo "⚠️  未找到resource/best.pt，请手动放置模型文件"
    echo "   然后运行: python3 scripts/convert_model.py -i resource/best.pt -o models/best.onnx --validate"
fi

# 安装前端依赖
echo "📦 安装前端依赖..."
pnpm install

# 编译Rust依赖
echo "🦀 编译Rust依赖..."
cd src-tauri
cargo build
cd ..

echo ""
echo "🎉 环境设置完成！"
echo ""
echo "下一步："
echo "1. 将YOLO模型文件(best.pt)放在resource/目录下"
echo "2. 运行模型转换: python3 scripts/convert_model.py -i resource/best.pt -o models/best.onnx"
echo "3. 启动开发服务器: ./scripts/dev.sh"