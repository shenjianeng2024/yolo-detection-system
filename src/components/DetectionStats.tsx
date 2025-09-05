import React from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { AlertTriangle, CheckCircle, Activity, Clock } from 'lucide-react'

interface DetectionResult {
  detections: Detection[]
  timestamp: string
}

interface Detection {
  class_id: number
  class_name: string
  confidence: number
  bbox: [number, number, number, number]
}

interface DetectionStatsProps {
  results: DetectionResult[]
  currentResult: DetectionResult | null
}

export const DetectionStats: React.FC<DetectionStatsProps> = ({ 
  results, 
  currentResult 
}) => {
  // 计算统计信息
  const totalDetections = results.reduce((sum, result) => sum + result.detections.length, 0)
  const abnormalCount = results.reduce((sum, result) => 
    sum + result.detections.filter(d => d.class_name === '异常').length, 0
  )
  const normalCount = results.reduce((sum, result) => 
    sum + result.detections.filter(d => d.class_name === '正常').length, 0
  )
  
  const currentAbnormal = currentResult?.detections.filter(d => d.class_name === '异常').length || 0
  const currentNormal = currentResult?.detections.filter(d => d.class_name === '正常').length || 0
  const currentTotal = currentResult?.detections.length || 0

  // 计算平均置信度
  const allDetections = results.flatMap(r => r.detections)
  const avgConfidence = allDetections.length > 0 
    ? allDetections.reduce((sum, d) => sum + d.confidence, 0) / allDetections.length 
    : 0

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      {/* 当前检测结果 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">当前检测</CardTitle>
          <Activity className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{currentTotal}</div>
          <p className="text-xs text-muted-foreground">
            {currentAbnormal > 0 && (
              <Badge variant="destructive" className="mr-1">
                异常 {currentAbnormal}
              </Badge>
            )}
            {currentNormal > 0 && (
              <Badge variant="secondary">
                正常 {currentNormal}
              </Badge>
            )}
          </p>
        </CardContent>
      </Card>

      {/* 异常检测总数 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">异常检测</CardTitle>
          <AlertTriangle className="h-4 w-4 text-destructive" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-destructive">{abnormalCount}</div>
          <p className="text-xs text-muted-foreground">
            累计异常检测数量
          </p>
        </CardContent>
      </Card>

      {/* 正常检测总数 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">正常检测</CardTitle>
          <CheckCircle className="h-4 w-4 text-green-600" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-green-600">{normalCount}</div>
          <p className="text-xs text-muted-foreground">
            累计正常数量 / 总计: {totalDetections}
          </p>
        </CardContent>
      </Card>

      {/* 平均置信度 */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">平均置信度</CardTitle>
          <Clock className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {avgConfidence > 0 ? (avgConfidence * 100).toFixed(1) : 0}%
          </div>
          <p className="text-xs text-muted-foreground">
            基于 {results.length} 次检测
          </p>
        </CardContent>
      </Card>
    </div>
  )
}