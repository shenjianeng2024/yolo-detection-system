# 技术栈详细信息

## Rust后端技术栈

### 核心框架
- **Tauri**: v2.0 - 跨平台桌面应用框架
- **Tokio**: v1.0 - 异步运行时，支持并发处理
- **Serde**: v1.0 - 序列化和反序列化
- **Anyhow**: v1.0 - 错误处理
- **Thiserror**: v1.0 - 自定义错误类型

### AI/ML核心
- **Candle Core**: v0.9 - HuggingFace的Rust ML框架
- **Candle ONNX**: v0.9 - ONNX模型支持
- **Candle NN**: v0.9 - 神经网络层
- **Candle Transformers**: v0.9 - 变换器模型支持

### 图像处理
- **Image**: v0.25 - Rust图像处理库（支持JPEG, PNG, BMP）
- **Imageproc**: v0.25 - 图像处理算法
- **Base64**: v0.22 - Base64编码解码
- **Ndarray**: v0.15 - 数值计算

### 同步和性能
- **Parking Lot**: v0.12 - 高性能同步原语
- **Futures**: v0.3 - Future处理
- **Tokio Stream**: v0.1 - 异步流处理
- **Chrono**: v0.4 - 时间处理

## 前端技术栈

### 核心框架
- **React**: v19.0.0 - 最新React版本
- **TypeScript**: v5.6.2 - 类型安全
- **Vite**: v6.0.1 - 快速构建工具

### UI组件和样式
- **Radix UI**: 完整的无障碍UI组件库
  - @radix-ui/react-checkbox: v1.1.2
  - @radix-ui/react-dialog: v1.1.2  
  - @radix-ui/react-progress: v1.1.7
  - @radix-ui/react-scroll-area: v1.2.10
  - @radix-ui/react-slider: v1.2.1
  - @radix-ui/react-switch: v1.2.6
  - @radix-ui/react-tabs: v1.1.1

- **Tailwind CSS**: v3.4.0 - 实用优先CSS框架
- **Tailwind Animate**: v1.0.7 - 动画扩展
- **Tailwind Merge**: v2.5.4 - 条件样式合并

- **Magic UI组件**: 增强UI体验
  - Shimmer Button - 闪光按钮效果
  - Ripple Button - 涟漪点击效果  
  - Pulsating Button - 脉冲动画按钮

### Tauri集成
- **@tauri-apps/api**: v2 - Tauri API绑定
- **@tauri-apps/plugin-dialog**: v2.3.3 - 文件对话框
- **@tauri-apps/plugin-shell**: v2 - 系统shell集成

### 动画和交互
- **Framer Motion**: v12.23.12 - 高性能动画库
- **Lucide React**: v0.454.0 - 现代图标库
- **Class Variance Authority**: v0.7.1 - 组件变体管理
- **Clsx**: v2.1.1 - 条件CSS类名

## 开发工具链

### 构建和打包
- **pnpm**: 包管理器（明确指定，不使用npm）
- **ESLint**: v9.12.0 - 代码质量检查
- **TypeScript ESLint**: v8.8.1 - TypeScript特定检查
- **PostCSS**: v8.5.6 - CSS后处理
- **Autoprefixer**: v10.4.21 - CSS前缀自动添加

### Tauri构建
- **@tauri-apps/cli**: v2 - Tauri命令行工具
- **tauri-build**: v2 - 构建系统集成

## 性能和优化特性

### Rust后端优势
- **零成本抽象**: Rust编译时优化
- **内存安全**: 无垃圾回收的内存管理
- **并发安全**: 编译时并发检查
- **原生性能**: 接近C++的执行效率

### 前端性能
- **Tree Shaking**: Vite自动移除未使用代码
- **代码分割**: 按需加载组件
- **热模块替换**: 开发时快速更新
- **TypeScript编译**: 编译时类型检查和优化

## 特色技术选择

### 为什么选择Candle而不是PyTorch？
1. **原生Rust**: 无需Python运行时
2. **内存效率**: 更低的内存占用
3. **部署简便**: 单一可执行文件
4. **性能优势**: 编译时优化，无GIL限制
5. **HuggingFace生态**: 直接支持HF模型

### 为什么选择Tauri而不是Electron？
1. **资源占用**: 显著更少的内存和CPU使用
2. **安全性**: Rust后端提供更强的安全保障
3. **性能**: 原生性能，无V8开销
4. **体积**: 更小的应用程序包体积
5. **系统集成**: 更好的操作系统集成