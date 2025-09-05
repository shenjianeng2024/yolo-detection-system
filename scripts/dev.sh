#!/bin/bash
# 开发服务器启动脚本

set -e

echo "🚀 启动YOLO检测系统开发服务器"
echo "================================"

# 检查模型文件
if [ ! -f "models/best.onnx" ]; then
    echo "⚠️  未找到ONNX模型文件 models/best.onnx"
    echo "请先运行模型转换："
    echo "  python3 scripts/convert_model.py -i resource/best.pt -o models/best.onnx"
    echo ""
    echo "或者将已有的ONNX模型复制到 models/best.onnx"
    echo ""
    read -p "是否继续启动？(y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 检查依赖
echo "📦 检查依赖..."
if [ ! -d "node_modules" ]; then
    echo "安装前端依赖..."
    pnpm install
fi

# 启动开发服务器
echo "🎯 启动Tauri开发服务器..."
echo "前端服务器: http://localhost:1420"
echo "按 Ctrl+C 停止服务器"
echo ""

pnpm tauri dev