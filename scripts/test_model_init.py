#!/usr/bin/env python3
"""
测试YOLO模型初始化的脚本
通过HTTP API调用Tauri应用的模型初始化功能
"""

import requests
import json
import os
from pathlib import Path

def test_model_initialization():
    """测试模型初始化API"""
    print("🧪 测试YOLO模型初始化")
    print("=" * 50)
    
    # 项目根目录
    project_root = Path(__file__).parent.parent
    models_dir = project_root / "models"
    
    # 检查模型文件
    best_model = models_dir / "best.onnx"
    
    if not best_model.exists():
        print(f"❌ 模型文件不存在: {best_model}")
        return False
    
    print(f"✅ 找到模型文件: {best_model}")
    print(f"📊 模型大小: {best_model.stat().st_size / (1024*1024):.2f} MB")
    
    # 构建API调用数据
    api_data = {
        "cmd": "initialize_yolo_model",
        "args": {
            "model_path": str(best_model.absolute())
        }
    }
    
    print(f"\n🔄 调用API初始化模型...")
    print(f"API数据: {json.dumps(api_data, indent=2, ensure_ascii=False)}")
    
    # 这个脚本主要用于文档和验证，实际的API调用需要在Tauri应用内部进行
    print("\n💡 API调用方法:")
    print("在React组件中使用以下代码:")
    print(f"""
    const initModel = async () => {{
      try {{
        const classNames = await invoke('initialize_yolo_model', {{
          modelPath: '{best_model.absolute()}'
        }});
        console.log('模型初始化成功:', classNames);
      }} catch (error) {{
        console.error('模型初始化失败:', error);
      }}
    }};
    """)
    
    return True

def main():
    """主函数"""
    if test_model_initialization():
        print("\n🎉 模型文件就绪，可以在UI中测试初始化")
        print("\n📋 下一步:")
        print("  1. 确保Tauri应用正在运行 (cargo tauri dev)")
        print("  2. 在浏览器中打开 http://localhost:1420")
        print("  3. 点击'初始化模型'按钮测试")
        print("  4. 上传图片进行检测测试")
    else:
        print("\n❌ 模型文件检查失败")

if __name__ == "__main__":
    main()