import React from 'react'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

interface DetectionResult {
  detections: Detection[]
  frame_data?: string
  timestamp: string
}

interface Detection {
  class_id: number
  class_name: string
  confidence: number
  bbox: [number, number, number, number] // [x, y, width, height]
}

interface DetectionVisualizationProps {
  result: DetectionResult | null
}

export const DetectionVisualization: React.FC<DetectionVisualizationProps> = ({ result }) => {
  if (!result) {
    return (
      <Card className="p-6 text-center">
        <p className="text-muted-foreground">暂无检测结果</p>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {/* 检测结果图像 */}
      {result.frame_data && (
        <Card className="p-4">
          <div className="relative">
            <img 
              src={`data:image/jpeg;base64,${result.frame_data}`}
              alt="检测结果"
              className="w-full rounded-lg border"
              style={{ maxHeight: '500px', objectFit: 'contain' }}
            />
            
            {/* 检测框叠加层 */}
            <div className="absolute inset-0">
              {result.detections.map((detection, index) => {
                const [x, y, width, height] = detection.bbox
                const confidenceColor = detection.confidence > 0.8 ? 'green' : 
                                      detection.confidence > 0.6 ? 'orange' : 'red'
                
                return (
                  <div
                    key={index}
                    className="absolute border-2 pointer-events-none"
                    style={{
                      left: `${x}px`,
                      top: `${y}px`,
                      width: `${width}px`,
                      height: `${height}px`,
                      borderColor: confidenceColor,
                    }}
                  >
                    {/* 检测标签 */}
                    <div 
                      className="absolute -top-8 left-0 px-2 py-1 text-xs text-white rounded"
                      style={{ backgroundColor: confidenceColor }}
                    >
                      {detection.class_name} ({(detection.confidence * 100).toFixed(1)}%)
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        </Card>
      )}

      {/* 检测结果列表 */}
      <Card className="p-4">
        <h3 className="font-semibold mb-3">检测结果详情</h3>
        
        {result.detections.length === 0 ? (
          <p className="text-muted-foreground text-center py-4">
            未检测到任何对象
          </p>
        ) : (
          <div className="space-y-2">
            {result.detections.map((detection, index) => (
              <div 
                key={index}
                className="flex items-center justify-between p-3 bg-muted rounded-lg"
              >
                <div className="flex items-center space-x-3">
                  <Badge 
                    variant={detection.class_name === '异常' ? 'destructive' : 'secondary'}
                  >
                    {detection.class_name}
                  </Badge>
                  <span className="text-sm">
                    位置: ({detection.bbox[0].toFixed(0)}, {detection.bbox[1].toFixed(0)})
                  </span>
                </div>
                <div className="flex items-center space-x-2">
                  <span 
                    className={`text-sm font-medium ${
                      detection.confidence > 0.8 ? 'text-green-600' :
                      detection.confidence > 0.6 ? 'text-orange-600' : 'text-red-600'
                    }`}
                  >
                    {(detection.confidence * 100).toFixed(1)}%
                  </span>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* 时间戳 */}
        <div className="mt-4 pt-4 border-t">
          <p className="text-xs text-muted-foreground">
            检测时间: {new Date(result.timestamp).toLocaleString()}
          </p>
        </div>
      </Card>
    </div>
  )
}