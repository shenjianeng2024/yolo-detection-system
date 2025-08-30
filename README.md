# YOLOv8实时检测系统

一个基于 Tauri + React + shadcn/ui 构建的现代化 YOLOv8 实时目标检测桌面应用程序。

## 功能特性

### 🎯 检测功能
- 支持多种输入源：摄像头、视频文件、静态图像
- 实时目标检测与识别
- 可调节的置信度阈值（按类别独立设置）
- 灵活的检测类别选择
- 实时检测结果展示

### 🎨 用户界面
- 现代化的 Material Design 风格界面
- 响应式布局，支持不同屏幕尺寸
- 直观的控制面板
- 实时状态指示器
- 检测结果可视化

### ⚙️ 技术架构
- **前端**: React 19 + TypeScript + shadcn/ui + Tailwind CSS
- **后端**: Rust + Tauri 2.0
- **构建工具**: Vite + pnpm
- **AI模型**: YOLOv8 (ultralytics)

## 界面组成

### 左侧区域 - 视频显示
- 大型视频显示窗口
- 支持实时视频流和文件播放
- 检测结果实时叠加显示
- 状态指示器

### 右侧区域 - 控制面板
1. **输入源选择**
   - 摄像头启动按钮
   - 视频文件选择按钮  
   - 图像文件选择按钮

2. **检测控制**
   - 开始检测按钮
   - 停止检测按钮

3. **置信度阈值设置**
   - 每个检测类别独立的滑块控制
   - 实时数值显示
   - 范围：0.00 - 1.00

4. **检测类别选择**
   - 支持的类别：person, bicycle, car, motorcycle, airplane, bus, train, truck 等
   - 复选框形式的类别开关
   - 支持批量选择/取消

5. **检测结果显示**
   - 滚动式结果文本框
   - 包含检测类别、置信度信息
   - 异常警告提示

## 安装与运行

### 环境要求
- Node.js 18+
- pnpm 8+
- Rust 1.70+
- Python 3.8+ (用于 YOLOv8 模型)

### 安装依赖
```bash
# 安装前端依赖
pnpm install

# 安装 Python 依赖 (YOLOv8)
pip install ultralytics
```

### 开发模式
```bash
# 启动开发服务器
pnpm tauri:dev
```

### 构建发布版本
```bash
# 构建应用程序
pnpm tauri:build
```

## 项目结构

```
yolo-detection-system/
├── src/                    # React 前端源码
│   ├── components/         # UI 组件
│   │   └── ui/            # shadcn/ui 组件
│   ├── App.tsx            # 主应用组件
│   ├── main.tsx           # 应用入口
│   └── globals.css        # 全局样式
├── src-tauri/             # Tauri 后端源码
│   ├── src/
│   │   └── main.rs        # Rust 后端主文件
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置文件
├── public/                # 静态资源
├── package.json           # 前端依赖配置
├── tailwind.config.js     # Tailwind CSS 配置
├── vite.config.ts         # Vite 配置
└── tsconfig.json          # TypeScript 配置
```

## 核心功能实现

### 前端 (React)
- 使用 shadcn/ui 组件库构建现代化界面
- TypeScript 提供类型安全
- 响应式设计适配不同屏幕尺寸
- 实时状态管理和数据轮询

### 后端 (Rust/Tauri)
- 高性能的 Rust 后端处理
- 安全的文件系统访问
- 跨平台桌面应用支持
- 与前端的异步通信

### 检测集成
- 支持多种 YOLOv8 模型
- 可配置的检测参数
- 实时结果处理和展示

## 特色亮点

1. **现代化设计**: 采用最新的设计系统和组件库
2. **高性能**: Rust 后端确保处理性能
3. **跨平台**: 支持 Windows、macOS、Linux
4. **可扩展**: 模块化设计便于功能扩展
5. **用户友好**: 直观的操作界面和实时反馈

## 开发计划

- [ ] 集成真实的 YOLOv8 检测引擎
- [ ] 添加检测结果导出功能
- [ ] 支持更多输入格式
- [ ] 添加检测历史记录
- [ ] 支持模型切换
- [ ] 添加设置持久化

## 技术特性

- **响应式设计**: 界面自适应不同屏幕尺寸
- **类型安全**: 全面的 TypeScript 类型定义  
- **组件化**: 模块化的 UI 组件设计
- **现代化**: 使用最新的前端技术栈
- **高性能**: Rust 后端提供优秀性能
- **跨平台**: 原生桌面应用体验

这个应用程序成功地将原有的 Python PyQt5 界面重新实现为现代化的 Tauri + React 应用，提供了更好的用户体验和更高的性能。