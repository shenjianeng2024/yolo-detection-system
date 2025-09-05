#!/usr/bin/env python3
"""
YOLO模型转换脚本 - 将.pt模型转换为.onnx格式
依赖: pip install ultralytics
"""

import os
import sys
from pathlib import Path
from ultralytics import YOLO

def convert_pt_to_onnx(pt_path: str, output_dir: str = None, img_size: int = 640):
    """
    将YOLO .pt模型转换为.onnx格式
    
    Args:
        pt_path: .pt模型文件路径
        output_dir: 输出目录，默认为模型同目录
        img_size: 输入图像尺寸
    """
    try:
        # 验证输入文件
        pt_file = Path(pt_path)
        if not pt_file.exists():
            raise FileNotFoundError(f"模型文件不存在: {pt_path}")
        
        if not pt_file.suffix == '.pt':
            raise ValueError(f"文件必须是.pt格式: {pt_path}")
        
        print(f"✅ 找到模型文件: {pt_file}")
        
        # 加载YOLO模型
        print("🔄 加载YOLO模型...")
        model = YOLO(str(pt_file))
        
        # 设置输出路径
        if output_dir is None:
            output_dir = pt_file.parent
        else:
            output_dir = Path(output_dir)
            output_dir.mkdir(parents=True, exist_ok=True)
        
        # 导出为ONNX格式
        print(f"🔄 转换为ONNX格式 (图像尺寸: {img_size}x{img_size})...")
        
        # 导出参数详细配置
        export_results = model.export(
            format='onnx',           # 导出格式
            opset=12,               # ONNX opset版本（兼容性好）
            dynamic=True,           # 支持动态输入尺寸
            simplify=True,          # 简化模型
            imgsz=img_size,         # 输入图像尺寸
        )
        
        onnx_path = Path(export_results)
        
        # 移动到指定目录（如果需要）
        if onnx_path.parent != output_dir:
            final_path = output_dir / onnx_path.name
            onnx_path.rename(final_path)
            onnx_path = final_path
        
        print(f"✅ 转换完成!")
        print(f"📄 输出文件: {onnx_path}")
        print(f"📊 文件大小: {onnx_path.stat().st_size / (1024*1024):.2f} MB")
        
        return str(onnx_path)
        
    except Exception as e:
        print(f"❌ 转换失败: {e}")
        sys.exit(1)

def create_class_names_file(output_dir: str):
    """创建类别名称文件"""
    try:
        output_dir = Path(output_dir)
        names_file = output_dir / "class_names.txt"
        
        # 简化的二分类系统
        class_names = [
            "异常",  # class 0
            "正常",  # class 1
        ]
        
        with open(names_file, 'w', encoding='utf-8') as f:
            for name in class_names:
                f.write(f"{name}\n")
        
        print(f"✅ 创建类别文件: {names_file}")
        return str(names_file)
        
    except Exception as e:
        print(f"❌ 创建类别文件失败: {e}")
        return None

def main():
    """主函数"""
    print("🚀 YOLO模型转换工具")
    print("=" * 50)
    
    # 项目根目录
    project_root = Path(__file__).parent.parent
    resource_dir = project_root / "resource"
    models_dir = project_root / "models"
    
    # 创建models目录
    models_dir.mkdir(exist_ok=True)
    
    # 查找.pt文件
    pt_files = list(resource_dir.glob("*.pt"))
    
    if not pt_files:
        print(f"❌ 在 {resource_dir} 中未找到.pt模型文件")
        sys.exit(1)
    
    print(f"📁 找到 {len(pt_files)} 个.pt模型文件:")
    for i, pt_file in enumerate(pt_files, 1):
        print(f"  {i}. {pt_file.name}")
    
    # 转换所有模型
    converted_models = []
    
    for pt_file in pt_files:
        print(f"\n🔄 处理模型: {pt_file.name}")
        print("-" * 30)
        
        try:
            onnx_path = convert_pt_to_onnx(
                str(pt_file), 
                str(models_dir), 
                img_size=640
            )
            converted_models.append(onnx_path)
            
        except Exception as e:
            print(f"❌ 转换 {pt_file.name} 失败: {e}")
            continue
    
    # 创建类别名称文件
    create_class_names_file(models_dir)
    
    # 总结
    print("\n" + "=" * 50)
    print("🎉 转换完成!")
    print(f"✅ 成功转换 {len(converted_models)} 个模型")
    print(f"📁 输出目录: {models_dir}")
    
    if converted_models:
        print("\n📋 转换结果:")
        for model_path in converted_models:
            print(f"  • {Path(model_path).name}")
    
    print(f"\n💡 下一步:")
    print(f"  1. 检查 {models_dir} 目录中的.onnx文件")
    print(f"  2. 在Rust代码中使用转换后的模型")
    print(f"  3. 运行: cargo tauri dev")

if __name__ == "__main__":
    main()