import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { resolve } from '@tauri-apps/api/path'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Progress } from '@/components/ui/progress'
import { ShimmerButton } from '@/components/ui/shimmer-button'
import { RippleButton } from '@/components/ui/ripple-button'
import { PulsatingButton } from '@/components/ui/pulsating-button'
import { FileUpload, type FileInfo } from '@/components/ui/file-upload'
import { ImagePreview, type DetectionBox } from '@/components/ui/image-preview'
import { Spinner } from '@/components/ui/spinner'
import { Toaster } from '@/components/ui/toaster'
import { useToast } from '@/lib/use-toast'
import { 
  Camera, 
  Video, 
  Image as ImageIcon, 
  Square, 
  Settings, 
  Activity,
  AlertCircle,
  CheckCircle,
  RotateCcw,
  Zap,
  RefreshCcw
} from 'lucide-react'

// 类型定义
interface ClassConfig {
  id: number
  name: string
  confidence: number
  enabled: boolean
}

interface DetectionResult {
  timestamp: string
  className: string
  confidence: number
  bbox: [number, number, number, number]
  message: string
}

interface YoloAppState {
  isRunning: boolean
  inputSource: 'camera' | 'video' | 'image' | null
  classes: ClassConfig[]
  detectionResults: DetectionResult[]
  currentFrame: string | null
  loading: boolean
  error: string | null
  progress: number
  processingImage: boolean
  selectedImageFile: FileInfo | null
  imageProcessingProgress: number
  detectionBoxes: DetectionBox[]
}

const YoloApp: React.FC = () => {
  // 状态管理
  const [state, setState] = useState<YoloAppState>({
    isRunning: false,
    inputSource: null,
    classes: [],
    detectionResults: [],
    currentFrame: null,
    loading: true,
    error: null,
    progress: 0,
    processingImage: false,
    selectedImageFile: null,
    imageProcessingProgress: 0,
    detectionBoxes: []
  })

  const { toast } = useToast()

  const videoRef = useRef<HTMLDivElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const frameUpdateRef = useRef<number | null>(null)

  // 初始化YOLO模型
  useEffect(() => {
    initializeYolo()
    
    return () => {
      // 清理资源
      if (frameUpdateRef.current) {
        clearTimeout(frameUpdateRef.current)
      }
      if (state.isRunning) {
        invoke('stop_detection').catch(console.error)
      }
    }
  }, [])

  const initializeYolo = async () => {
    try {
      setState(prev => ({ ...prev, loading: true, error: null, progress: 20 }))
      
      // 调用Rust函数初始化YOLO模型  
      await invoke('initialize_yolo_model', {
        modelPath: '../models/best.onnx'
      })
      
      setState(prev => ({ ...prev, progress: 60 }))
      
      // 设置简化的异常检测类别配置
      const classes: ClassConfig[] = [
        {
          id: 0,
          name: '正常',
          confidence: 0.9,
          enabled: true
        },
        {
          id: 1,
          name: '异常',
          confidence: 0.9,
          enabled: true
        }
      ]

      setState(prev => ({ 
        ...prev, 
        classes, 
        loading: false,
        progress: 100
      }))
      
      // 添加成功消息
      addDetectionResult({
        timestamp: new Date().toLocaleTimeString(),
        className: 'System',
        confidence: 1.0,
        bbox: [0, 0, 0, 0],
        message: `✅ YOLO异常检测系统初始化成功`
      })
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: `初始化失败: ${error}`, 
        loading: false,
        progress: 0
      }))
    }
  }

  const addDetectionResult = (result: DetectionResult) => {
    setState(prev => ({
      ...prev,
      detectionResults: [...prev.detectionResults.slice(-19), result]
    }))
  }

  // 开始摄像头检测
  const startCamera = async () => {
    try {
      setState(prev => ({ ...prev, loading: true, error: null }))
      
      await invoke('start_camera_detection')
      
      setState(prev => ({ 
        ...prev, 
        inputSource: 'camera',
        isRunning: true,
        loading: false 
      }))
      
      addDetectionResult({
        timestamp: new Date().toLocaleTimeString(),
        className: 'System',
        confidence: 1.0,
        bbox: [0, 0, 0, 0],
        message: '📹 摄像头检测已启动'
      })
      
      // 开始帧更新循环
      startFrameUpdate()
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: `摄像头启动失败: ${error}`, 
        loading: false 
      }))
    }
  }

  // 选择视频文件
  const selectVideo = async () => {
    try {
      const file = await open({
        multiple: false,
        filters: [{ name: 'Video', extensions: ['mp4', 'avi', 'mov'] }]
      })

      if (!file) return

      const filePath = Array.isArray(file) ? file[0] : file
      setState(prev => ({ ...prev, loading: true }))
      
      await invoke('load_video_source', { path: filePath })
      
      setState(prev => ({ 
        ...prev, 
        inputSource: 'video',
        isRunning: true,
        loading: false 
      }))
      
      addDetectionResult({
        timestamp: new Date().toLocaleTimeString(),
        className: 'System',
        confidence: 1.0,
        bbox: [0, 0, 0, 0],
        message: `🎬 视频文件已加载: ${filePath}`
      })
      
      startFrameUpdate()
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: `视频加载失败: ${error}`, 
        loading: false 
      }))
    }
  }

  // 处理图片文件拖拽 - 引导用户使用'选择图片'按钮
  const handleImageSelect = async (_file: File, fileInfo: FileInfo) => {
    // 引导用户使用正确的文件选择方法
    toast({
      title: "请使用'选择图片'按钮",
      description: "为获取正确的文件路径，请点击下方的'选择图片'按钮来选择文件。",
      variant: "default"
    })
    
    // 添加系统消息到检测结果
    addDetectionResult({
      timestamp: new Date().toLocaleTimeString(),
      className: 'System',
      confidence: 1.0,
      bbox: [0, 0, 0, 0],
      message: `💡 检测到拖拽文件 ${fileInfo.name}，请使用'选择图片'按钮获取正确路径`
    })
  }


  // 传统文件选择方法（保持向后兼容）
  const selectImage = async () => {
    try {
const result = await open({
        multiple: false,
        filters: [{ name: 'Image', extensions: ['jpg', 'jpeg', 'png', 'bmp'] }]
      })

      if (!result) {
return
      }

      // Tauri v2 返回格式处理
      let filePath: string
      
      if (typeof result === 'string') {
        // 直接返回字符串路径
        filePath = result
} else if (Array.isArray(result) && (result as string[]).length > 0) {
        // 返回数组（虽然multiple为false，但为了兼容性）
        filePath = (result as string[])[0]
} else {
toast({
          title: "文件选择错误",
          description: "未能获取有效的文件路径",
          variant: "destructive"
        })
        return
      }

      // 路径格式检查
      
      // 检查文件路径是否已经是绝对路径
      let absolutePath: string
      try {
        if (filePath.startsWith('/') || (filePath.includes(':') && filePath.includes('\\'))) {
          // 已经是绝对路径
          absolutePath = filePath

        } else {
          // 相对路径，需要解析
absolutePath = await resolve(filePath)
}
      } catch (error) {
        console.error('Path resolution error:', error)
        // 如果解析失败，直接使用原路径
        absolutePath = filePath
      }

      // 跨平台路径处理 - 获取文件名
      const pathSeparator = absolutePath.includes('\\') ? '\\' : '/'
      const fileName = absolutePath.split(pathSeparator).pop() || 'unknown.jpg'
      
// 创建FileInfo
      const fileInfo: FileInfo = {
        name: fileName,
        size: 0, // 无法获取文件大小
        type: 'image/jpeg',
        lastModified: Date.now()
      }

      // 直接处理图片
      await processImageFromPath(absolutePath, fileInfo)
    } catch (error) {
      console.error('File selection error:', error)
      toast({
        title: "选择文件失败",
        description: `无法打开文件选择器: ${error}`,
        variant: "destructive"
      })
    }
  }

  // 从路径处理图片
  const processImageFromPath = async (filePath: string, fileInfo: FileInfo) => {
try {
      setState(prev => ({
        ...prev,
        processingImage: true,
        selectedImageFile: fileInfo,
        imageProcessingProgress: 0,
        error: null,
        currentFrame: null,
        detectionBoxes: [],
        detectionResults: prev.detectionResults.filter(result => result.className === 'System')
      }))

      toast({
        title: "开始处理图片",
        description: `正在处理 ${fileInfo.name}...`,
        variant: "default"
      })

      const progressInterval = setInterval(() => {
        setState(prev => ({
          ...prev,
          imageProcessingProgress: Math.min(prev.imageProcessingProgress + 10, 90)
        }))
      }, 200)
      
      const result = await invoke('process_single_image', {
        path: filePath,
        classConfigs: state.classes
      }) as any

      clearInterval(progressInterval)

      const detectionBoxes: DetectionBox[] = result.detections?.map((det: any) => ({
        className: det.confidence > 0.5 ? '异常' : '正常',
        confidence: det.confidence,
        bbox: det.bbox
      })) || []
      
      setState(prev => ({ 
        ...prev, 
        inputSource: 'image',
        currentFrame: result.imageData,
        processingImage: false,
        imageProcessingProgress: 100,
        detectionBoxes
      }))

      // 添加检测结果
      if (result.detections && result.detections.length > 0) {
        result.detections.forEach((det: any) => {
          const detectionType = det.confidence > 0.5 ? '异常' : '正常'
          addDetectionResult({
            timestamp: new Date().toLocaleTimeString(),
            className: detectionType,
            confidence: det.confidence,
            bbox: det.bbox,
            message: `检测结果: ${detectionType} (置信度: ${(det.confidence * 100).toFixed(1)}%)`
          })
        })
        
        toast({
          title: "检测完成",
          description: `发现 ${result.detections.length} 个检测对象`,
          variant: "success"
        })
      } else {
        addDetectionResult({
          timestamp: new Date().toLocaleTimeString(),
          className: '未检测出',
          confidence: 0,
          bbox: [0, 0, 0, 0],
          message: '🖼️ 图片处理完成，未检测出异常或正常状态'
        })
        
        toast({
          title: "处理完成",
          description: "图片已处理，未检测到异常或正常状态",
          variant: "default"
        })
      }
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        processingImage: false,
        error: `图片处理失败: ${error}` 
      }))
      
      toast({
        title: "处理失败",
        description: `图片处理时出现错误: ${error}`,
        variant: "destructive"
      })
    }
  }

  // 清除图片结果
  const clearImageResults = () => {
    setState(prev => ({
      ...prev,
      currentFrame: null,
      selectedImageFile: null,
      detectionBoxes: [],
      inputSource: prev.inputSource === 'image' ? null : prev.inputSource,
      processingImage: false,
      imageProcessingProgress: 0,
      error: null
    }))
    
    toast({
      title: "已清除",
      description: "图片检测结果已清除",
      variant: "default"
    })
  }

  // 停止检测
  const stopDetection = async () => {
    try {
      // 清理帧更新循环
      if (frameUpdateRef.current) {
        clearTimeout(frameUpdateRef.current)
        frameUpdateRef.current = null
      }

      await invoke('stop_detection')
      
      setState(prev => ({ 
        ...prev, 
        isRunning: false,
        inputSource: null,
        currentFrame: null
      }))

      addDetectionResult({
        timestamp: new Date().toLocaleTimeString(),
        className: 'System',
        confidence: 1.0,
        bbox: [0, 0, 0, 0],
        message: '⏹️ 检测已停止'
      })
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: `停止检测失败: ${error}` 
      }))
    }
  }

  // 帧更新循环
  const startFrameUpdate = () => {
    const updateFrame = async () => {
      if (!state.isRunning) return

      try {
        const frameResult = await invoke('get_next_frame', {
          classConfigs: state.classes.filter(cls => cls.enabled)
        }) as any
        
        if (frameResult && frameResult.success) {
          setState(prev => ({
            ...prev,
            currentFrame: frameResult.imageData
          }))

          // 处理新的检测结果
          if (frameResult.detections && frameResult.detections.length > 0) {
            frameResult.detections.forEach((det: any) => {
              const detectionType = det.confidence > 0.5 ? '异常' : '正常'
              addDetectionResult({
                timestamp: new Date().toLocaleTimeString(),
                className: detectionType,
                confidence: det.confidence,
                bbox: det.bbox,
                message: `🎯 检测结果: ${detectionType} (置信度: ${(det.confidence * 100).toFixed(1)}%)`
              })
            })
          } else {
            // 未检测出任何结果
            addDetectionResult({
              timestamp: new Date().toLocaleTimeString(),
              className: '未检测出',
              confidence: 0,
              bbox: [0, 0, 0, 0],
              message: '📊 未检测出异常或正常状态'
            })
          }
        }
      } catch (error) {
        console.error('Frame update error:', error)
      }
      
      // 继续下一帧
      if (state.isRunning) {
        frameUpdateRef.current = setTimeout(updateFrame, 33) // ~30fps
      }
    }

    updateFrame()
  }

  // 更新类别置信度
  const updateClassConfidence = (classId: number, confidence: number) => {
    setState(prev => ({
      ...prev,
      classes: prev.classes.map(cls =>
        cls.id === classId ? { ...cls, confidence } : cls
      )
    }))
  }

  // 切换类别启用状态
  const toggleClassEnabled = (classId: number) => {
    setState(prev => ({
      ...prev,
      classes: prev.classes.map(cls =>
        cls.id === classId ? { ...cls, enabled: !cls.enabled } : cls
      )
    }))
  }

  // 清空检测结果
  const clearResults = () => {
    setState(prev => ({ ...prev, detectionResults: [] }))
  }

  // 重置配置
  const resetConfiguration = async () => {
    try {
      await invoke('reset_configuration')
      await initializeYolo()
      
      addDetectionResult({
        timestamp: new Date().toLocaleTimeString(),
        className: 'System',
        confidence: 1.0,
        bbox: [0, 0, 0, 0],
        message: '🔄 配置已重置为默认值'
      })
    } catch (error) {
      setState(prev => ({ 
        ...prev, 
        error: `重置配置失败: ${error}` 
      }))
    }
  }

  // 加载界面
  if (state.loading) {
    return (
      <div className="flex items-center justify-center h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
        <Card className="w-96 p-6">
          <div className="text-center space-y-4">
            <div className="flex justify-center">
              <Zap className="w-12 h-12 text-blue-600 animate-pulse" />
            </div>
            <h2 className="text-2xl font-bold text-gray-800 dark:text-white">
              YOLO检测系统
            </h2>
            <p className="text-gray-600 dark:text-gray-300">正在初始化模型...</p>
            <Progress value={state.progress} className="w-full" />
            <p className="text-sm text-gray-500">{state.progress}%</p>
          </div>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex h-screen bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800">
      {/* 主要内容区域 */}
      <div className="flex-1 flex flex-col p-6">
        {/* 标题栏 */}
        <div className="mb-6">
          <h1 className="text-3xl font-bold text-gray-800 dark:text-white mb-2 flex items-center gap-3">
            <Zap className="w-8 h-8 text-blue-600" />
            YOLO实时检测系统
          </h1>
          <div className="flex items-center space-x-2">
            <Badge variant={state.isRunning ? "default" : "secondary"}>
              {state.isRunning ? "运行中" : "已停止"}
            </Badge>
            {state.inputSource && (
              <Badge variant="outline">
                {state.inputSource === 'camera' ? '📹 摄像头' : 
                 state.inputSource === 'video' ? '🎬 视频' : '🖼️ 图片'}
              </Badge>
            )}
            <Badge variant="outline" className="ml-auto">
              v2.0 - Magic UI
            </Badge>
          </div>
        </div>

        {/* 错误提示 */}
        {state.error && (
          <div className="mb-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-xl flex items-center shadow-lg">
            <AlertCircle className="w-5 h-5 text-red-500 mr-3" />
            <span className="text-red-700 dark:text-red-300">{state.error}</span>
          </div>
        )}

        {/* 视频/图片显示区域 */}
        <Card className="flex-1 mb-6 shadow-xl border-0 bg-white/80 dark:bg-gray-800/80 backdrop-blur-xl">
          <CardContent className="p-0 h-full">
            <div 
              ref={videoRef}
              className="w-full h-full bg-gradient-to-br from-gray-900 to-gray-700 rounded-xl flex items-center justify-center min-h-96 relative overflow-hidden"
            >
              {state.currentFrame ? (
                <>
                  {state.inputSource === 'image' ? (
                    <ImagePreview
                      src={`data:image/jpeg;base64,${state.currentFrame}`}
                      alt="检测结果"
                      fileName={state.selectedImageFile?.name}
                      fileSize={state.selectedImageFile ? `${(state.selectedImageFile.size / 1024).toFixed(1)} KB` : undefined}
                      detections={state.detectionBoxes}
                      onClose={clearImageResults}
                      onReset={clearImageResults}
                      className="w-full h-full"
                    />
                  ) : (
                    <>
                      <img 
                        src={`data:image/jpeg;base64,${state.currentFrame}`}
                        alt="检测画面"
                        className="max-w-full max-h-full object-contain rounded-lg"
                      />
                      {/* 实时状态指示器 */}
                      {state.isRunning && (
                        <div className="absolute top-4 left-4">
                          <div className="flex items-center gap-2 bg-black/70 text-white px-3 py-2 rounded-full backdrop-blur-sm">
                            <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse"></div>
                            <span className="text-sm font-medium">实时检测中</span>
                          </div>
                        </div>
                      )}
                    </>
                  )}
                </>
              ) : (
                <div className="w-full h-full p-6">
                  {state.inputSource === 'image' && state.processingImage ? (
                    <div className="w-full h-full flex items-center justify-center">
                      <div className="text-center space-y-4">
                        <Spinner size="large" variant="white" />
                        <div className="text-white space-y-2">
                          <p className="text-xl font-medium">正在处理图片...</p>
                          <p className="text-sm opacity-75">{state.selectedImageFile?.name}</p>
                          <div className="w-64 mx-auto">
                            <Progress value={state.imageProcessingProgress} className="bg-white/20" />
                            <p className="text-xs mt-2 opacity-60">{state.imageProcessingProgress}%</p>
                          </div>
                        </div>
                      </div>
                    </div>
                  ) : (
                    <FileUpload
                      onFileSelect={handleImageSelect}
                      onClear={clearImageResults}
                      accept="image/*"
                      disabled={state.loading || state.processingImage || state.isRunning}
                      loading={state.processingImage}
                      selectedFile={state.selectedImageFile}
                      error={state.error}
                      disableClick={true}
                      className="w-full h-full min-h-[300px] bg-transparent border-white/30 text-white"
                    />
                  )}
                </div>
              )}
            </div>
          </CardContent>
        </Card>

        {/* 控制按钮 */}
        <div className="flex flex-wrap gap-4 justify-center">
          <ShimmerButton
            onClick={startCamera}
            disabled={state.isRunning || state.loading}
            className="flex items-center gap-2 bg-blue-600 hover:bg-blue-700 text-white px-6 py-3"
          >
            <Camera className="w-5 h-5" />
            摄像头检测
          </ShimmerButton>
          
          <RippleButton
            onClick={selectVideo}
            disabled={state.isRunning || state.loading}
            className="flex items-center gap-2 bg-purple-600 hover:bg-purple-700 text-white px-6 py-3"
            rippleColor="#ffffff"
          >
            <Video className="w-5 h-5" />
            选择视频
          </RippleButton>
          
          <RippleButton
            onClick={selectImage}
            disabled={state.loading || state.processingImage}
            className="flex items-center gap-2 bg-green-600 hover:bg-green-700 text-white px-6 py-3"
            rippleColor="#ffffff"
          >
            {state.processingImage ? (
              <Spinner size="small" variant="white" />
            ) : (
              <ImageIcon className="w-5 h-5" />
            )}
            {state.processingImage ? '处理中...' : '选择图片'}
          </RippleButton>
          
          {state.inputSource === 'image' && (
            <RippleButton
              onClick={clearImageResults}
              disabled={state.loading || state.processingImage}
              className="flex items-center gap-2 bg-gray-600 hover:bg-gray-700 text-white px-4 py-3"
              rippleColor="#ffffff"
            >
              <RefreshCcw className="w-5 h-5" />
              清除图片
            </RippleButton>
          )}
          
          {state.isRunning && (
            <PulsatingButton
              onClick={stopDetection}
              className="flex items-center gap-2 bg-red-600 hover:bg-red-700 text-white px-6 py-3"
              pulseColor="#ef4444"
            >
              <Square className="w-5 h-5" />
              停止检测
            </PulsatingButton>
          )}

          <RippleButton
            onClick={resetConfiguration}
            disabled={state.loading}
            className="flex items-center gap-2 bg-gray-600 hover:bg-gray-700 text-white px-4 py-3"
            rippleColor="#ffffff"
          >
            <RotateCcw className="w-5 h-5" />
            重置
          </RippleButton>
        </div>
      </div>

      {/* 侧边栏 */}
      <div className="w-80 bg-white/90 dark:bg-gray-800/90 backdrop-blur-xl border-l border-gray-200 dark:border-gray-700 flex flex-col shadow-2xl">
        {/* 参数设置 */}
        <Card className="m-4 shadow-lg border-0 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm">
          <CardHeader className="pb-3">
            <CardTitle className="flex items-center gap-2 text-lg">
              <Settings className="w-5 h-5 text-blue-600" />
              参数设置
            </CardTitle>
          </CardHeader>
          <CardContent>
            <ScrollArea className="h-64">
              <div className="space-y-4">
                {state.classes.map((cls) => (
                  <div key={cls.id} className="space-y-3 p-3 rounded-lg bg-gray-50 dark:bg-gray-700/50">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <div className={`w-3 h-3 rounded-full ${
                          cls.name === '正常' ? 'bg-green-500' : 
                          cls.name === '异常' ? 'bg-red-500' : 'bg-gray-400'
                        }`}></div>
                        <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">{cls.name}</span>
                      </div>
                      <Switch
                        checked={cls.enabled}
                        onCheckedChange={() => toggleClassEnabled(cls.id)}
                      />
                    </div>
                    <div className="space-y-2">
                      <div className="flex justify-between text-xs text-gray-500">
                        <span>置信度阈值</span>
                        <Badge variant="secondary" className="text-xs">
                          {cls.confidence.toFixed(2)}
                        </Badge>
                      </div>
                      <Slider
                        value={[cls.confidence]}
                        onValueChange={([value]) => updateClassConfidence(cls.id, value)}
                        max={1}
                        min={0}
                        step={0.01}
                        disabled={!cls.enabled}
                        className="w-full"
                      />
                    </div>
                  </div>
                ))}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* 检测结果 */}
        <Card className="flex-1 m-4 shadow-lg border-0 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2 text-lg">
                <Activity className="w-5 h-5 text-green-600" />
                检测结果
                <Badge variant="outline" className="ml-2">
                  {state.detectionResults.length}
                </Badge>
              </CardTitle>
              <RippleButton
                onClick={clearResults}
                className="text-xs px-3 py-1 bg-gray-100 hover:bg-gray-200 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300"
                rippleColor="#6b7280"
              >
                清空
              </RippleButton>
            </div>
          </CardHeader>
          <CardContent className="pt-0">
            <ScrollArea className="h-full max-h-96">
              <div className="space-y-3">
                {state.detectionResults.length === 0 ? (
                  <div className="text-center py-12">
                    <CheckCircle className="w-12 h-12 mx-auto text-gray-400 mb-3" />
                    <p className="text-gray-500 text-sm">暂无检测结果</p>
                    <p className="text-gray-400 text-xs mt-1">开始检测后将显示结果</p>
                  </div>
                ) : (
                  state.detectionResults.slice().reverse().map((result, index) => {
                    const isSystemMessage = result.className === 'System'
                    const isImageResult = state.inputSource === 'image' && !isSystemMessage
                    
                    return (
                      <div
                        key={index}
                        className={`p-3 rounded-xl border shadow-sm hover:shadow-md transition-all duration-200 ${
                          isSystemMessage 
                            ? 'bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-gray-700/50 dark:to-gray-600/50 border-blue-100 dark:border-gray-600'
                            : result.className === '异常'
                            ? 'bg-gradient-to-r from-red-50 to-pink-50 dark:from-red-900/20 dark:to-red-800/20 border-red-200 dark:border-red-700'
                            : result.className === '正常'
                            ? 'bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-green-800/20 border-green-200 dark:border-green-700'
                            : 'bg-gradient-to-r from-gray-50 to-slate-50 dark:from-gray-700/50 dark:to-gray-600/50 border-gray-200 dark:border-gray-600'
                        }`}
                      >
                        <div className="flex items-center gap-2 mb-2">
                          {isSystemMessage ? (
                            <Zap className="w-4 h-4 text-blue-500" />
                          ) : result.className === '异常' ? (
                            <AlertCircle className="w-4 h-4 text-red-500" />
                          ) : result.className === '正常' ? (
                            <CheckCircle className="w-4 h-4 text-green-500" />
                          ) : (
                            <Activity className="w-4 h-4 text-gray-500" />
                          )}
                          <span className="font-semibold text-sm text-gray-800 dark:text-gray-200">
                            {result.className}
                          </span>
                          {!isSystemMessage && (
                            <Badge 
                              variant={result.className === '异常' ? 'destructive' : 'default'}
                              className="text-xs ml-auto"
                            >
                              {(result.confidence * 100).toFixed(1)}%
                            </Badge>
                          )}
                        </div>
                        
                        <p className="text-xs text-gray-600 dark:text-gray-300 leading-relaxed mb-2">
                          {result.message}
                        </p>
                        
                        {isImageResult && result.bbox && result.bbox.some(val => val > 0) && (
                          <div className="text-xs text-gray-500 dark:text-gray-400 bg-gray-100 dark:bg-gray-700/50 rounded px-2 py-1 mb-2">
                            <span className="font-medium">检测框:</span> 
                            [{result.bbox.map(val => val.toFixed(0)).join(', ')}]
                          </div>
                        )}
                        
                        <div className="flex items-center justify-between text-xs text-gray-400 mt-2">
                          <div className="flex items-center gap-1">
                            <span className="w-1 h-1 bg-gray-400 rounded-full"></span>
                            <span>{result.timestamp}</span>
                          </div>
                          {isImageResult && (
                            <Badge variant="outline" className="text-xs">
                              🖼️ 图片检测
                            </Badge>
                          )}
                        </div>
                      </div>
                    )
                  })
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        {/* 系统状态 */}
        <Card className="m-4 shadow-lg border-0 bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm">
          <CardContent className="pt-4 space-y-3">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-2">
                <div className={`w-2 h-2 rounded-full ${state.loading ? 'bg-yellow-500 animate-pulse' : 'bg-green-500'}`}></div>
                <span className="text-gray-600 dark:text-gray-300">
                  模型状态: {state.loading ? '初始化中' : '已就绪'}
                </span>
              </div>
              {state.classes.length > 0 && (
                <Badge variant="outline" className="text-xs">
                  {state.classes.filter(c => c.enabled).length}/{state.classes.length} 类别
                </Badge>
              )}
            </div>
            
            {/* 图片信息显示 */}
            {state.selectedImageFile && (
              <div className="pt-2 border-t border-gray-200 dark:border-gray-600">
                <div className="text-sm space-y-2">
                  <div className="flex items-center gap-2 text-gray-700 dark:text-gray-300">
                    <ImageIcon className="w-4 h-4 text-blue-500" />
                    <span className="font-medium">当前图片</span>
                  </div>
                  
                  <div className="text-xs text-gray-600 dark:text-gray-400 space-y-1 ml-6">
                    <div className="flex justify-between">
                      <span>文件名:</span>
                      <span className="font-mono text-right max-w-32 truncate" title={state.selectedImageFile.name}>
                        {state.selectedImageFile.name}
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>文件大小:</span>
                      <span className="font-mono">
                        {(state.selectedImageFile.size / 1024).toFixed(1)} KB
                      </span>
                    </div>
                    <div className="flex justify-between">
                      <span>文件类型:</span>
                      <span className="font-mono text-right">
                        {state.selectedImageFile.type}
                      </span>
                    </div>
                    {state.detectionBoxes.length > 0 && (
                      <div className="flex justify-between">
                        <span>检测数量:</span>
                        <Badge variant="default" className="text-xs h-5">
                          {state.detectionBoxes.length} 个对象
                        </Badge>
                      </div>
                    )}
                    {state.processingImage && (
                      <div className="flex justify-between items-center">
                        <span>处理进度:</span>
                        <div className="flex items-center gap-2">
                          <Progress value={state.imageProcessingProgress} className="w-16 h-2" />
                          <span className="font-mono text-xs">{state.imageProcessingProgress}%</span>
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* 隐藏的文件输入 */}
      <input
        ref={fileInputRef}
        type="file"
        accept="image/*,video/*"
        className="hidden"
      />
      
      {/* Toast 通知 */}
      <Toaster />
    </div>
  )
}

export default YoloApp