import React, { useState, useRef, useEffect } from 'react'
import { Camera, Video, Image, Play, Square, Settings, AlertTriangle, Monitor, CheckCircle, XCircle } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Slider } from '@/components/ui/slider'
import { Checkbox } from '@/components/ui/checkbox'
import { Textarea } from '@/components/ui/textarea'
import { ShimmerButton } from '@/components/ui/shimmer-button'
import { Ripple } from '@/components/ui/ripple'
import { AnimatedGridPattern } from '@/components/ui/animated-grid-pattern'
import { invoke } from '@tauri-apps/api/core'
import './globals.css'

interface DetectionResult {
  class_name: string
  confidence: number
  bbox: [number, number, number, number] // [x, y, width, height]
}

interface DetectionState {
  is_running: boolean
  current_source?: string
  source_type?: string
  results: DetectionResult[]
}

// 简化的二分类系统 - 只有正常和异常
const classificationTypes = {
  normal: '正常',
  abnormal: '异常'
}

interface ConfidenceThresholds {
  [className: string]: number
}

interface ClassSelections {
  [className: string]: boolean
}

function App() {
  const [detectionState, setDetectionState] = useState<DetectionState>({
    is_running: false,
    results: []
  })
  const [selectedSource, setSelectedSource] = useState<'camera' | 'video' | 'image' | null>(null)
  
  // 简化的置信度阈值 - 只有正常和异常
  const [confThresholds, setConfThresholds] = useState<ConfidenceThresholds>({
    '正常': 0.5,
    '异常': 0.7
  })
  
  // 类别选择状态 - 默认全选
  const [classSelections, setClassSelections] = useState<ClassSelections>({
    '正常': true,
    '异常': true
  })
  
  const [detectionResults, setDetectionResults] = useState('')
  const [currentFile, setCurrentFile] = useState<string>('')

  // 轮询检测状态
  useEffect(() => {
    const pollDetectionState = async () => {
      try {
        const state = await invoke<DetectionState>('get_detection_state')
        setDetectionState(state)
        
        // 检测结果处理 - 转换为正常/异常分类
        if (state.results.length > 0) {
          const resultText = state.results.map(result => {
            // 模拟二分类逻辑：根据类别名称判断是否异常
            const isAbnormal = ['person', 'car', 'truck', 'fire hydrant', 'stop sign'].includes(result.class_name)
            const classification = isAbnormal ? '异常' : '正常'
            return `检测结果: ${classification} (置信度: ${result.confidence.toFixed(2)})!\n`
          }).join('')
          setDetectionResults(prev => prev + resultText)
        }
      } catch (error) {
        console.error('Failed to get detection state:', error)
      }
    }

    const interval = setInterval(pollDetectionState, 1000)
    return () => clearInterval(interval)
  }, [])

  // 摄像头处理
  const handleStartCamera = async () => {
    try {
      await invoke<string>('stop_detection')
      const result = await invoke<string>('start_camera')
      setSelectedSource('camera')
      setCurrentFile('')
      await handleStartDetection()
      console.log(result)
    } catch (error) {
      console.error('Failed to start camera:', error)
    }
  }

  // 视频选择
  const handleSelectVideo = async () => {
    try {
      await invoke<string>('stop_detection')
      const result = await invoke<string>('select_video_file')
      setSelectedSource('video')
      setCurrentFile(result)
      await handleStartDetection()
      console.log(result)
    } catch (error) {
      console.error('Failed to select video:', error)
    }
  }

  // 图像选择
  const handleSelectImage = async () => {
    try {
      await invoke<string>('stop_detection')
      const result = await invoke<string>('select_image_file')
      setSelectedSource('image')
      setCurrentFile(result)
      await processImage()
      console.log(result)
    } catch (error) {
      console.error('Failed to select image:', error)
    }
  }

  const processImage = async () => {
    try {
      const result = await invoke<string>('start_detection')
      console.log(result)
    } catch (error) {
      console.error('Failed to process image:', error)
      setDetectionResults(prev => prev + `错误: ${error}\n`)
    }
  }

  const handleStartDetection = async () => {
    try {
      const result = await invoke<string>('start_detection')
      console.log(result)
    } catch (error) {
      console.error('Failed to start detection:', error)
      setDetectionResults(prev => prev + `错误: ${error}\n`)
    }
  }

  const handleStopDetection = async () => {
    try {
      const result = await invoke<string>('stop_detection')
      console.log(result)
    } catch (error) {
      console.error('Failed to stop detection:', error)
    }
  }

  const updateConfidenceThreshold = (className: string, value: number) => {
    setConfThresholds(prev => ({ ...prev, [className]: value }))
  }

  const toggleClassSelection = (className: string, checked: boolean) => {
    setClassSelections(prev => ({ ...prev, [className]: checked }))
  }

  // 统计正常和异常的检测数量
  const getDetectionStats = () => {
    const normalCount = detectionState.results.filter(result => 
      !['person', 'car', 'truck', 'fire hydrant', 'stop sign'].includes(result.class_name)
    ).length
    const abnormalCount = detectionState.results.length - normalCount
    return { normal: normalCount, abnormal: abnormalCount }
  }

  const stats = getDetectionStats()

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-900 via-gray-800 to-slate-900 relative overflow-hidden">
      {/* 动画背景网格 */}
      <AnimatedGridPattern
        numSquares={30}
        maxOpacity={0.1}
        duration={3}
        repeatDelay={1}
        className="inset-x-0 inset-y-[-30%] h-[200%] skew-y-12 [mask-image:radial-gradient(500px_circle_at_center,white,transparent)]"
      />
      
      <div className="flex h-screen relative z-10">
        {/* 左侧 - 视频显示区域 */}
        <div className="flex-1 p-6">
          <Card className="h-full bg-black/20 border-gray-600/30 shadow-2xl backdrop-blur-sm relative overflow-hidden">
            {/* 检测状态下的涟漪效果 */}
            {detectionState.is_running && (
              <Ripple 
                className="opacity-30"
                mainCircleSize={200}
                mainCircleOpacity={0.2}
                numCircles={6}
              />
            )}
            
            <CardHeader className="pb-4 relative z-10">
              <CardTitle className="flex items-center gap-3 text-white">
                <Monitor className="w-6 h-6 text-blue-400" />
                智能检测系统
                {detectionState.is_running && (
                  <div className="ml-auto flex items-center gap-4">
                    {/* 实时统计显示 */}
                    <div className="flex items-center gap-4 text-sm">
                      <div className="flex items-center gap-2 bg-green-500/20 px-3 py-1 rounded-full border border-green-500/30">
                        <CheckCircle className="w-4 h-4 text-green-400" />
                        <span className="text-green-400 font-medium">正常: {stats.normal}</span>
                      </div>
                      <div className="flex items-center gap-2 bg-red-500/20 px-3 py-1 rounded-full border border-red-500/30">
                        <XCircle className="w-4 h-4 text-red-400" />
                        <span className="text-red-400 font-medium">异常: {stats.abnormal}</span>
                      </div>
                    </div>
                    <div className="flex items-center gap-2 bg-blue-500/20 px-3 py-1 rounded-full border border-blue-500/30">
                      <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
                      <span className="text-sm text-blue-400 font-medium">检测中</span>
                    </div>
                  </div>
                )}
              </CardTitle>
            </CardHeader>
            <CardContent className="h-full pb-6">
              <div className="w-full h-full bg-black rounded-xl flex items-center justify-center relative overflow-hidden border border-gray-700/50 shadow-inner" style={{minHeight: '600px', minWidth: '800px'}}>
                {selectedSource ? (
                  <div className="w-full h-full flex flex-col items-center justify-center text-white">
                    {selectedSource === 'camera' ? (
                      <>
                        <Camera className="w-24 h-24 mb-6 text-blue-400 opacity-60" />
                        <p className="text-2xl mb-3 font-medium">摄像头检测</p>
                        <p className="text-lg text-gray-400">实时智能分析</p>
                      </>
                    ) : selectedSource === 'video' ? (
                      <>
                        <Video className="w-24 h-24 mb-6 text-purple-400 opacity-60" />
                        <p className="text-2xl mb-3 font-medium">视频分析</p>
                        <p className="text-lg text-gray-400 text-center px-6">
                          {currentFile ? currentFile.split('/').pop() : '视频文件分析'}
                        </p>
                      </>
                    ) : (
                      <>
                        <Image className="w-24 h-24 mb-6 text-green-400 opacity-60" />
                        <p className="text-2xl mb-3 font-medium">图像检测</p>
                        <p className="text-lg text-gray-400 text-center px-6">
                          {currentFile ? currentFile.split('/').pop() : '静态图像分析'}
                        </p>
                      </>
                    )}
                  </div>
                ) : (
                  <div className="text-gray-400 text-center">
                    <Monitor className="w-20 h-20 mx-auto mb-6 opacity-30" />
                    <p className="text-xl mb-2">智能检测系统</p>
                    <p className="text-base">选择输入源开始检测</p>
                  </div>
                )}
                
                {/* 检测结果叠加 - 简化显示 */}
                {detectionState.results.length > 0 && (
                  <div className="absolute top-6 left-6 space-y-3 max-w-xs">
                    {detectionState.results.slice(0, 5).map((result, index) => {
                      const isAbnormal = ['person', 'car', 'truck', 'fire hydrant', 'stop sign'].includes(result.class_name)
                      return (
                        <div key={index} className={`px-4 py-2 rounded-lg font-medium flex items-center gap-2 backdrop-blur-sm shadow-lg ${
                          isAbnormal 
                            ? 'bg-red-500/90 text-white' 
                            : 'bg-green-500/90 text-white'
                        }`}>
                          {isAbnormal ? (
                            <XCircle className="w-4 h-4" />
                          ) : (
                            <CheckCircle className="w-4 h-4" />
                          )}
                          {isAbnormal ? '异常' : '正常'}: {(result.confidence * 100).toFixed(1)}%
                        </div>
                      )
                    })}
                    {detectionState.results.length > 5 && (
                      <div className="text-gray-300 text-sm bg-black/50 px-3 py-1 rounded backdrop-blur-sm">
                        +{detectionState.results.length - 5} 更多结果...
                      </div>
                    )}
                  </div>
                )}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* 右侧 - 控制面板 */}
        <div className="w-96 p-6 space-y-6 overflow-y-auto">
          {/* 输入源选择 */}
          <Card className="bg-black/20 border-gray-600/30 shadow-xl backdrop-blur-sm">
            <CardHeader>
              <CardTitle className="text-white flex items-center gap-2">
                <Settings className="w-5 h-5 text-blue-400" />
                输入源选择
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <ShimmerButton 
                onClick={handleStartCamera}
                className={`w-full justify-start transition-all duration-200 ${
                  selectedSource === 'camera' 
                    ? 'bg-blue-600/90 border-blue-400' 
                    : 'bg-gray-800/50 border-gray-600 hover:bg-blue-600/20'
                }`}
                background={selectedSource === 'camera' ? 'rgba(37, 99, 235, 0.9)' : 'rgba(31, 41, 55, 0.8)'}
                shimmerColor="#60a5fa"
              >
                <Camera className="w-5 h-5 mr-3" />
                摄像头检测
              </ShimmerButton>
              <ShimmerButton 
                onClick={handleSelectVideo}
                className={`w-full justify-start transition-all duration-200 ${
                  selectedSource === 'video' 
                    ? 'bg-purple-600/90 border-purple-400' 
                    : 'bg-gray-800/50 border-gray-600 hover:bg-purple-600/20'
                }`}
                background={selectedSource === 'video' ? 'rgba(147, 51, 234, 0.9)' : 'rgba(31, 41, 55, 0.8)'}
                shimmerColor="#a855f7"
              >
                <Video className="w-5 h-5 mr-3" />
                视频分析
              </ShimmerButton>
              <ShimmerButton 
                onClick={handleSelectImage}
                className={`w-full justify-start transition-all duration-200 ${
                  selectedSource === 'image' 
                    ? 'bg-green-600/90 border-green-400' 
                    : 'bg-gray-800/50 border-gray-600 hover:bg-green-600/20'
                }`}
                background={selectedSource === 'image' ? 'rgba(34, 197, 94, 0.9)' : 'rgba(31, 41, 55, 0.8)'}
                shimmerColor="#34d399"
              >
                <Image className="w-5 h-5 mr-3" />
                图像检测
              </ShimmerButton>
            </CardContent>
          </Card>

          {/* 控制按钮 */}
          <Card className="bg-black/20 border-gray-600/30 shadow-xl backdrop-blur-sm">
            <CardContent className="pt-6">
              <div className="flex gap-3">
                <ShimmerButton 
                  onClick={handleStartDetection}
                  disabled={!selectedSource || detectionState.is_running}
                  className="flex-1 disabled:opacity-50 disabled:cursor-not-allowed"
                  background="rgba(34, 197, 94, 0.9)"
                  shimmerColor="#10b981"
                  shimmerDuration="2s"
                >
                  <Play className="w-4 h-4 mr-2" />
                  开始检测
                </ShimmerButton>
                <ShimmerButton 
                  onClick={handleStopDetection}
                  disabled={!detectionState.is_running}
                  className="flex-1 disabled:opacity-50 disabled:cursor-not-allowed"
                  background="rgba(239, 68, 68, 0.9)"
                  shimmerColor="#f87171"
                  shimmerDuration="2s"
                >
                  <Square className="w-4 h-4 mr-2" />
                  停止检测
                </ShimmerButton>
              </div>
            </CardContent>
          </Card>

          {/* 简化的置信度阈值设置 - 只有正常和异常 */}
          <Card className="bg-black/20 border-gray-600/30 shadow-xl backdrop-blur-sm">
            <CardHeader>
              <CardTitle className="text-white text-lg flex items-center gap-2">
                <Settings className="w-5 h-5 text-blue-400" />
                检测阈值设置
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* 正常检测阈值 */}
              <div className="space-y-3 p-4 bg-green-500/10 rounded-lg border border-green-500/30">
                <div className="flex justify-between items-center">
                  <div className="flex items-center gap-2">
                    <CheckCircle className="w-4 h-4 text-green-400" />
                    <label className="text-sm font-medium text-green-300">正常检测阈值</label>
                  </div>
                  <span className="text-sm text-green-400 font-mono bg-green-900/30 px-2 py-1 rounded">
                    {confThresholds['正常']?.toFixed(2)}
                  </span>
                </div>
                <Slider
                  value={[confThresholds['正常'] || 0.5]}
                  onValueChange={(value) => updateConfidenceThreshold('正常', value[0])}
                  max={1}
                  min={0}
                  step={0.01}
                  className="w-full"
                />
                <p className="text-xs text-green-400/70">低于此值的检测结果将被忽略</p>
              </div>

              {/* 异常检测阈值 */}
              <div className="space-y-3 p-4 bg-red-500/10 rounded-lg border border-red-500/30">
                <div className="flex justify-between items-center">
                  <div className="flex items-center gap-2">
                    <XCircle className="w-4 h-4 text-red-400" />
                    <label className="text-sm font-medium text-red-300">异常检测阈值</label>
                  </div>
                  <span className="text-sm text-red-400 font-mono bg-red-900/30 px-2 py-1 rounded">
                    {confThresholds['异常']?.toFixed(2)}
                  </span>
                </div>
                <Slider
                  value={[confThresholds['异常'] || 0.7]}
                  onValueChange={(value) => updateConfidenceThreshold('异常', value[0])}
                  max={1}
                  min={0}
                  step={0.01}
                  className="w-full"
                />
                <p className="text-xs text-red-400/70">高于此值将触发异常警报</p>
              </div>
            </CardContent>
          </Card>

          {/* 简化的检测类别选择 */}
          <Card className="bg-black/20 border-gray-600/30 shadow-xl backdrop-blur-sm">
            <CardHeader>
              <CardTitle className="text-white text-lg">检测类别</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center space-x-3 p-3 hover:bg-green-500/10 rounded-lg transition-colors duration-200 border border-green-500/20">
                <Checkbox 
                  id="正常"
                  checked={classSelections['正常'] || false}
                  onCheckedChange={(checked) => toggleClassSelection('正常', !!checked)}
                  className="border-green-400 data-[state=checked]:bg-green-600"
                />
                <CheckCircle className="w-4 h-4 text-green-400" />
                <label 
                  htmlFor="正常"
                  className="text-sm font-medium text-green-300 cursor-pointer flex-1"
                >
                  正常状态检测
                </label>
              </div>
              <div className="flex items-center space-x-3 p-3 hover:bg-red-500/10 rounded-lg transition-colors duration-200 border border-red-500/20">
                <Checkbox 
                  id="异常"
                  checked={classSelections['异常'] || false}
                  onCheckedChange={(checked) => toggleClassSelection('异常', !!checked)}
                  className="border-red-400 data-[state=checked]:bg-red-600"
                />
                <XCircle className="w-4 h-4 text-red-400" />
                <label 
                  htmlFor="异常"
                  className="text-sm font-medium text-red-300 cursor-pointer flex-1"
                >
                  异常状态检测
                </label>
              </div>
            </CardContent>
          </Card>

          {/* 检测结果 */}
          <Card className="bg-black/20 border-gray-600/30 shadow-xl backdrop-blur-sm">
            <CardHeader>
              <CardTitle className="text-white text-lg flex items-center gap-2">
                <AlertTriangle className="w-5 h-5 text-yellow-400" />
                检测日志
              </CardTitle>
            </CardHeader>
            <CardContent>
              <Textarea
                value={detectionResults}
                readOnly
                placeholder="检测结果将在此显示..."
                className="min-h-32 resize-none bg-black/50 border-gray-600 text-gray-300 font-mono text-sm"
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}

export default App