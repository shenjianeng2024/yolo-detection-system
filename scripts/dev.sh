#!/bin/bash

# YOLOv8 Detection System - Development Script
echo "🚀 启动 YOLOv8 实时检测系统..."

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

# 检查依赖是否安装
if [ ! -d "node_modules" ]; then
    echo "📦 安装前端依赖..."
    pnpm install
fi

# 启动开发服务器
echo "🔥 启动开发服务器..."
echo "请耐心等待 Rust 后端编译完成（首次运行可能需要几分钟）..."
pnpm tauri:dev