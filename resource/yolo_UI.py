import sys
import cv2
import os
from PyQt5.QtWidgets import (QApplication, QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, 
                             QLabel, QPushButton, QGroupBox, QSlider, QCheckBox, 
                             QTextEdit, QFileDialog, QComboBox)
from PyQt5.QtCore import Qt, QTimer, QThread, pyqtSignal, QObject
from PyQt5.QtGui import QImage, QPixmap
from ultralytics import YOLO  # yolo官方的库

# 设置环境变量解决平台插件问题（Linux系统必需）
os.environ["QT_QPA_PLATFORM_PLUGIN_PATH"] = "" # qt的问题

class YOLOv8DetectionApp(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("YOLOv8实时检测系统")
        self.setGeometry(100, 100, 1200, 800)
        
        # 加载YOLOv8模型
        self.model = YOLO("best.pt")
        self.class_names = self.model.names
        self.conf_thresholds = {cls: 0.5 for cls in self.class_names.values()}
        
        # 初始化摄像头
        self.cap = None
        self.video_path = None
        self.is_camera = False
        self.is_running = False
        
        # 初始化UI
        self.init_ui()
        
    def init_ui(self):
        # 主布局
        main_widget = QWidget()
        main_layout = QHBoxLayout()
        main_widget.setLayout(main_layout)
        self.setCentralWidget(main_widget)
        
        # 左侧区域 - 视频显示
        self.video_label = QLabel()
        self.video_label.setAlignment(Qt.AlignCenter)
        self.video_label.setStyleSheet("background-color: black;")
        self.video_label.setMinimumSize(800, 600)
        
        # 右侧区域 - 参数设置和结果
        right_layout = QVBoxLayout()
        
        # 参数设置组
        param_group = QGroupBox("参数设置")
        param_layout = QVBoxLayout()
        
        # 输入源选择
        source_group = QGroupBox("输入源")
        source_layout = QHBoxLayout()
        self.camera_btn = QPushButton("摄像头")
        self.camera_btn.clicked.connect(self.start_camera)
        self.video_btn = QPushButton("选择视频")
        self.video_btn.clicked.connect(self.select_video)
        self.image_btn = QPushButton("选择图片")
        self.image_btn.clicked.connect(self.select_image)
        source_layout.addWidget(self.camera_btn)
        source_layout.addWidget(self.video_btn)
        source_layout.addWidget(self.image_btn)
        source_group.setLayout(source_layout)
        param_layout.addWidget(source_group)
        
        # 置信度阈值设置
        conf_group = QGroupBox("置信度阈值")
        conf_layout = QVBoxLayout()
        self.conf_sliders = {}
        
        for cls_id, cls_name in self.class_names.items():
            group = QGroupBox(cls_name)
            layout = QHBoxLayout()
            slider = QSlider(Qt.Horizontal)
            slider.setRange(0, 100)
            slider.setValue(50)
            slider.valueChanged.connect(lambda value, cls=cls_name: self.update_conf_threshold(cls, value/100))
            label = QLabel("0.5")
            layout.addWidget(slider)
            layout.addWidget(label)
            group.setLayout(layout)
            conf_layout.addWidget(group)
            self.conf_sliders[cls_name] = (slider, label)
        conf_group.setLayout(conf_layout)
        param_layout.addWidget(conf_group)
        
        # 类别选择
        cls_group = QGroupBox("检测类别")
        cls_layout = QVBoxLayout()
        self.cls_checks = {}
        
        for cls_id, cls_name in self.class_names.items():
            check = QCheckBox(cls_name)
            check.setChecked(True)
            cls_layout.addWidget(check)
            self.cls_checks[cls_name] = check
        cls_group.setLayout(cls_layout)
        param_layout.addWidget(cls_group)
        
        # 控制按钮
        control_layout = QHBoxLayout()
        self.start_btn = QPushButton("开始")
        self.start_btn.clicked.connect(self.start_detection)
        self.stop_btn = QPushButton("停止")
        self.stop_btn.clicked.connect(self.stop_detection)
        self.stop_btn.setEnabled(False)
        control_layout.addWidget(self.start_btn)
        control_layout.addWidget(self.stop_btn)
        param_layout.addLayout(control_layout)
        
        param_group.setLayout(param_layout)
        right_layout.addWidget(param_group)
        
        # 结果展示区
        result_group = QGroupBox("检测结果")
        result_layout = QVBoxLayout()
        self.result_text = QTextEdit()
        self.result_text.setReadOnly(True)
        result_layout.addWidget(self.result_text)
        result_group.setLayout(result_layout)
        right_layout.addWidget(result_group)
        
        # 添加到主布局
        main_layout.addWidget(self.video_label)
        main_layout.addLayout(right_layout)
        
        # 定时器
        self.timer = QTimer()
        self.timer.timeout.connect(self.update_frame)
        
    def update_conf_threshold(self, cls_name, value):
        self.conf_thresholds[cls_name] = value
        _, label = self.conf_sliders[cls_name]
        label.setText(f"{value:.2f}")
        
    def start_camera(self):
        self.stop_detection()
        self.cap = cv2.VideoCapture(0)
        self.is_camera = True
        self.video_path = None
        self.start_detection()
        
    def select_video(self):
        file, _ = QFileDialog.getOpenFileName(self, "选择视频文件", "", "视频文件 (*.mp4 *.avi *.mov)")
        if file:
            self.stop_detection()
            self.cap = cv2.VideoCapture(file)
            self.is_camera = False
            self.video_path = file
            self.start_detection()
            
    def select_image(self):
        file, _ = QFileDialog.getOpenFileName(self, "选择图片文件", "", "图片文件 (*.jpg *.png *.bmp)")
        if file:
            self.stop_detection()
            self.cap = None
            self.is_camera = False
            self.video_path = file
            self.process_image(file)
            
    def process_image(self, image_path):
        img = cv2.imread(image_path)
        if img is not None:
            # 获取选中的类别
            selected_classes = [cls_id for cls_id, cls_name in self.class_names.items() 
                              if self.cls_checks[cls_name].isChecked()]
            
            # 运行检测
            results = self.model.predict(
                img,
                conf=min(self.conf_thresholds.values()),
                classes=selected_classes
            )
            # 后处理：根据类别特定的置信度阈值过滤结果
            filtered_boxes = []
            for box in results[0].boxes:
                cls_id = int(box.cls)
                cls_name = self.model.names[cls_id]
                conf = float(box.conf)
                
            # 检查是否达到该类别的置信度阈值
                if conf >= self.conf_thresholds[cls_name]:
                    	filtered_boxes.append(box)
            
            # 更新结果中的boxes
            results[0].boxes = filtered_boxes
            # 绘制结果
            annotated_img = results[0].plot()
            
            # 检查异常
            self.check_abnormal(results[0])
            
            # 显示图像
            self.display_image(annotated_img)
            
    def start_detection(self):
        if self.cap is None and self.video_path is None:
            return
            
        self.is_running = True
        self.start_btn.setEnabled(False)
        self.stop_btn.setEnabled(True)
        
        if self.cap is not None:
            self.timer.start(30)  # 约30fps
            
    def stop_detection(self):
        self.is_running = False
        self.timer.stop()
        self.start_btn.setEnabled(True)
        self.stop_btn.setEnabled(False)
        
        if self.cap is not None:
            self.cap.release()
            self.cap = None
            
    def update_frame(self):
        if self.cap is not None and self.cap.isOpened():
            ret, frame = self.cap.read()
            if ret:
                # 获取选中的类别
                selected_classes = [cls_id for cls_id, cls_name in self.class_names.items() 
                                  if self.cls_checks[cls_name].isChecked()]
                
                # 运行检测
                results = self.model.predict(
                    frame,
                    conf=min(self.conf_thresholds.values()),
                    classes=selected_classes,
                    verbose=False
                )
                # 后处理：根据类别特定的置信度阈值过滤结果
                filtered_boxes = []
                for box in results[0].boxes:
                        cls_id = int(box.cls)
                        cls_name = self.model.names[cls_id]
                        conf = float(box.conf)
                
                        # 检查是否达到该类别的置信度阈值
                        if conf >= self.conf_thresholds[cls_name]:
                            filtered_boxes.append(box)
            
                # 更新结果中的boxes
                results[0].boxes = filtered_boxes
                
                # 绘制结果
                annotated_img = results[0].plot()
                
                # 检查异常
                self.check_abnormal(results[0])
                
                # 显示图像
                self.display_image(annotated_img)
            else:
                if not self.is_camera:
                    # 视频结束，重新开始
                    self.cap.release()
                    self.cap = cv2.VideoCapture(self.video_path)
                    
    # 异常检测
    def check_abnormal(self, result):
        for box in result.boxes:
            cls_name = self.class_names[int(box.cls)]
            conf = float(box.conf)
            
            if conf >= self.conf_thresholds[cls_name]:
                    # 后续可在此补充向主控报警等逻辑
                    message = f"警告: 检测到 {cls_name} (置信度: {conf:.2f})!\n"
                    self.result_text.append(message)

    def display_image(self, img):
        # 转换颜色空间从BGR到RGB
        img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
        
        # 获取图像尺寸
        h, w, ch = img.shape
        bytes_per_line = ch * w
        
        # 创建QImage
        q_img = QImage(img.data, w, h, bytes_per_line, QImage.Format_RGB888)
        
        # 缩放图像以适应标签大小
        pixmap = QPixmap.fromImage(q_img)
        pixmap = pixmap.scaled(
            self.video_label.width(), 
            self.video_label.height(), 
            Qt.KeepAspectRatio
        )
        
        # 显示图像
        self.video_label.setPixmap(pixmap)
        
    def closeEvent(self, event):
        self.stop_detection()
        if self.cap is not None:
            self.cap.release()
        event.accept()

if __name__ == "__main__":
    # 解决平台插件问题（Linux系统必需）
    os.environ["QT_QPA_PLATFORM"] = "xcb" # qt的问题
    
    app = QApplication(sys.argv)
    window = YOLOv8DetectionApp()
    window.show()
    sys.exit(app.exec_())


tauri +react +shadcn/ui