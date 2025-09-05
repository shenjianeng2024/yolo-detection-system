# 端口更改记录

## 🔄 端口变更

**原端口**: 1420  
**新端口**: 3842  
**变更原因**: 避免与其他Tauri程序端口冲突  
**变更时间**: 2025-09-02 23:32  

## 📝 修改的文件

### 1. Vite配置文件
**文件**: `vite.config.ts`
```typescript
// 修改前
server: {
  port: 1420,
  strictPort: true,
  // ...
}

// 修改后  
server: {
  port: 3842,
  strictPort: true,
  // ...
}
```

### 2. Tauri配置文件
**文件**: `src-tauri/tauri.conf.json`
```json
// 修改前
"devUrl": "http://localhost:1420",

// 修改后
"devUrl": "http://localhost:3842",
```

### 3. 测试脚本更新
**文件**: `scripts/test_error_scenarios.py`
- 更新了应用连接检查URL
- 更新了测试报告中的端口信息

### 4. 测试指南更新  
**文件**: `MANUAL_TEST_GUIDE.md`
- 更新了访问地址为 http://localhost:3842

## ✅ 验证结果

- **应用启动**: ✅ 成功在新端口3842启动
- **模型加载**: ✅ ONNX模型正常加载
- **前端访问**: ✅ http://localhost:3842 正常响应
- **功能完整性**: ✅ 所有功能正常工作

## 📄 相关链接

- **应用访问**: http://localhost:3842
- **开发命令**: `pnpm tauri dev` 
- **测试脚本**: `python3 scripts/test_error_scenarios.py`

---
**变更完成**: 端口从1420成功更改为3842，所有功能正常运行