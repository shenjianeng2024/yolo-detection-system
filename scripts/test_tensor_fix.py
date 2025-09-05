#!/usr/bin/env python3
"""
验证张量维度修复的测试脚本
"""

import urllib.request
import urllib.error
import json
import time

def test_image_processing():
    """测试图片处理是否正常"""
    print("🧪 开始测试张量维度修复")
    print("=" * 50)
    
    # 测试应用连接
    try:
        req = urllib.request.Request("http://localhost:3842")
        with urllib.request.urlopen(req, timeout=5) as response:
            if response.status == 200:
                print("✅ 应用连接正常")
            else:
                print(f"❌ 应用连接异常: HTTP {response.status}")
                return False
    except Exception as e:
        print(f"❌ 应用连接失败: {e}")
        return False
    
    print("\n📝 测试说明:")
    print("由于这是Tauri桌面应用，需要通过界面手动测试")
    print("请按照以下步骤验证修复:")
    print()
    print("1. 打开浏览器访问: http://localhost:3842")
    print("2. 点击'选择图片'按钮")
    print("3. 选择任意图片文件（建议选择之前失败的abnormal109.jpg）")
    print("4. 观察是否出现 'unexpected rank' 错误")
    print()
    print("🔍 预期结果:")
    print("- ✅ 不应该出现张量维度错误")
    print("- ✅ 应该显示张量维度调试信息: '[DEBUG] 输入张量维度: 4维'")
    print("- ✅ 应该显示: '[DEBUG] 处理后张量维度: [3, 640, 640]'")
    print("- ✅ 图片应该成功处理并显示检测结果")
    print()
    print("📱 如何查看调试日志:")
    print("- 在终端中查看运行 'pnpm tauri dev' 的窗口")
    print("- 或在浏览器开发者工具的Console中查看")
    
    return True

def main():
    """主函数"""
    success = test_image_processing()
    
    if success:
        print("\n" + "=" * 50)
        print("🎉 测试脚本运行完成")
        print("📋 请手动验证修复效果并查看控制台日志")
        print("🔗 应用地址: http://localhost:3842")
    else:
        print("\n" + "=" * 50)
        print("❌ 测试环境检查失败")
        print("💡 请确保应用正在运行: pnpm tauri dev")

if __name__ == "__main__":
    main()