#!/bin/bash

# YOLOv8 Detection System - Build Script
echo "🏗️  构建 YOLOv8 实时检测系统..."

# 检查 pnpm 是否安装
if ! command -v pnpm &> /dev/null
then
    echo "❌ pnpm 未安装，请先安装 pnpm"
    echo "npm install -g pnpm"
    exit 1
fi

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null
then
    echo "❌ Rust 未安装，请先安装 Rust"
    echo "https://rustup.rs/"
    exit 1
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
rm -rf dist/
rm -rf src-tauri/target/release/

# 安装依赖
echo "📦 安装依赖..."
pnpm install

# 构建应用
echo "🚀 开始构建应用程序..."
echo "这可能需要几分钟时间，请耐心等待..."
pnpm tauri:build

echo "✅ 构建完成！"
echo "可执行文件位置："
find src-tauri/target/release/bundle -name "*.app" -o -name "*.exe" -o -name "*.deb" -o -name "*.AppImage" 2>/dev/null || echo "构建文件在 src-tauri/target/release/bundle/ 目录中"