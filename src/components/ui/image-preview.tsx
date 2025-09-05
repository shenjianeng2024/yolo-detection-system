"use client"

import * as React from "react"
import { cn } from "@/lib/utils"
import { Badge } from "./badge"
import { Button } from "./button"
import { Card, CardContent } from "./card"
import { X, Download, ZoomIn, RotateCcw } from "lucide-react"

interface DetectionBox {
  className: string
  confidence: number
  bbox: [number, number, number, number]
}

interface ImagePreviewProps {
  src: string
  alt?: string
  fileName?: string
  fileSize?: string
  detections?: DetectionBox[]
  onClose?: () => void
  onReset?: () => void
  className?: string
  showControls?: boolean
}

const ImagePreview = React.forwardRef<HTMLDivElement, ImagePreviewProps>(
  ({ 
    src, 
    alt = "预览图片", 
    fileName, 
    fileSize,
    detections = [],
    onClose,
    onReset,
    className,
    showControls = true,
    ...props 
  }, ref) => {
    const [isLoaded, setIsLoaded] = React.useState(false)
    const [isZoomed, setIsZoomed] = React.useState(false)
    const imageRef = React.useRef<HTMLImageElement>(null)

    const handleImageLoad = () => {
      setIsLoaded(true)
    }

    const handleDownload = () => {
      const link = document.createElement('a')
      link.href = src
      link.download = fileName || 'detection-result.jpg'
      document.body.appendChild(link)
      link.click()
      document.body.removeChild(link)
    }

    const toggleZoom = () => {
      setIsZoomed(!isZoomed)
    }

    return (
      <Card ref={ref} className={cn("relative overflow-hidden", className)} {...props}>
        {/* 控制按钮栏 */}
        {showControls && (
          <div className="absolute top-2 right-2 z-10 flex gap-2">
            <Button
              variant="secondary"
              size="sm"
              onClick={toggleZoom}
              className="bg-black/70 hover:bg-black/80 text-white border-none"
            >
              <ZoomIn className="w-4 h-4" />
            </Button>
            <Button
              variant="secondary"
              size="sm"
              onClick={handleDownload}
              className="bg-black/70 hover:bg-black/80 text-white border-none"
            >
              <Download className="w-4 h-4" />
            </Button>
            {onReset && (
              <Button
                variant="secondary"
                size="sm"
                onClick={onReset}
                className="bg-black/70 hover:bg-black/80 text-white border-none"
              >
                <RotateCcw className="w-4 h-4" />
              </Button>
            )}
            {onClose && (
              <Button
                variant="secondary"
                size="sm"
                onClick={onClose}
                className="bg-black/70 hover:bg-black/80 text-white border-none"
              >
                <X className="w-4 h-4" />
              </Button>
            )}
          </div>
        )}

        {/* 文件信息栏 */}
        {(fileName || fileSize) && (
          <div className="absolute bottom-2 left-2 z-10 flex gap-2">
            {fileName && (
              <Badge variant="secondary" className="bg-black/70 text-white border-none">
                {fileName}
              </Badge>
            )}
            {fileSize && (
              <Badge variant="outline" className="bg-black/70 text-white border-white/30">
                {fileSize}
              </Badge>
            )}
          </div>
        )}

        {/* 检测结果统计 */}
        {detections.length > 0 && (
          <div className="absolute top-2 left-2 z-10">
            <Badge variant="default" className="bg-green-600 hover:bg-green-700">
              检测到 {detections.length} 个对象
            </Badge>
          </div>
        )}

        <CardContent className="p-0">
          <div className={cn(
            "relative w-full h-full min-h-[300px] flex items-center justify-center bg-gradient-to-br from-gray-900 to-gray-700",
            isZoomed && "overflow-auto"
          )}>
            {!isLoaded && (
              <div className="absolute inset-0 flex items-center justify-center">
                <div className="w-8 h-8 border-2 border-white border-t-transparent rounded-full animate-spin" />
              </div>
            )}
            
            <img
              ref={imageRef}
              src={src}
              alt={alt}
              onLoad={handleImageLoad}
              className={cn(
                "transition-all duration-300 rounded-lg",
                isZoomed 
                  ? "max-w-none cursor-zoom-out" 
                  : "max-w-full max-h-full object-contain cursor-zoom-in",
                !isLoaded && "opacity-0"
              )}
              onClick={toggleZoom}
            />

            {/* 检测框叠加层 */}
            {isLoaded && detections.length > 0 && imageRef.current && (
              <div className="absolute inset-0 pointer-events-none">
                {detections.map((detection, index) => {
                  const [x, y, width, height] = detection.bbox
                  const imgRect = imageRef.current?.getBoundingClientRect()
                  if (!imgRect) return null

                  // 计算检测框在显示图片中的位置
                  const scaleX = imgRect.width / (imageRef.current?.naturalWidth || 1)
                  const scaleY = imgRect.height / (imageRef.current?.naturalHeight || 1)
                  
                  return (
                    <div
                      key={index}
                      className="absolute border-2 border-red-500 bg-red-500/10"
                      style={{
                        left: `${x * scaleX}px`,
                        top: `${y * scaleY}px`,
                        width: `${width * scaleX}px`,
                        height: `${height * scaleY}px`,
                      }}
                    >
                      <Badge 
                        variant="destructive" 
                        className="absolute -top-6 left-0 text-xs"
                      >
                        {detection.className} {(detection.confidence * 100).toFixed(1)}%
                      </Badge>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    )
  }
)

ImagePreview.displayName = "ImagePreview"

export { ImagePreview, type DetectionBox }