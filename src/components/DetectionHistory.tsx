import React from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Eye, Download, Trash2 } from 'lucide-react'

interface DetectionResult {
  detections: Detection[]
  frame_data?: string
  timestamp: string
}

interface Detection {
  class_id: number
  class_name: string
  confidence: number
  bbox: [number, number, number, number]
}

interface DetectionHistoryProps {
  results: DetectionResult[]
  onSelectResult: (result: DetectionResult) => void
  onClearHistory: () => void
}

export const DetectionHistory: React.FC<DetectionHistoryProps> = ({
  results,
  onSelectResult,
  onClearHistory
}) => {
  const handleExportResults = () => {
    const exportData = {
      timestamp: new Date().toISOString(),
      total_results: results.length,
      results: results.map(result => ({
        timestamp: result.timestamp,
        detections_count: result.detections.length,
        detections: result.detections.map(d => ({
          class_name: d.class_name,
          confidence: d.confidence,
          bbox: d.bbox
        }))
      }))
    }

    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json'
    })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `yolo_detection_results_${new Date().toISOString().split('T')[0]}.json`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  return (
    <Card className="h-full">
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">检测历史</CardTitle>
          <div className="flex space-x-2">
            {results.length > 0 && (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleExportResults}
                  className="text-xs"
                >
                  <Download className="w-3 h-3 mr-1" />
                  导出
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={onClearHistory}
                  className="text-xs text-destructive"
                >
                  <Trash2 className="w-3 h-3 mr-1" />
                  清空
                </Button>
              </>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        {results.length === 0 ? (
          <div className="p-6 text-center text-muted-foreground">
            暂无检测历史
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <div className="p-4 space-y-3">
              {results.slice().reverse().map((result, index) => {
                const reverseIndex = results.length - 1 - index
                const abnormalCount = result.detections.filter(d => d.class_name === '异常').length
                const normalCount = result.detections.filter(d => d.class_name === '正常').length
                const totalCount = result.detections.length
                
                return (
                  <div
                    key={reverseIndex}
                    className="p-3 border rounded-lg hover:bg-muted cursor-pointer transition-colors"
                    onClick={() => onSelectResult(result)}
                  >
                    <div className="flex items-center justify-between mb-2">
                      <div className="flex items-center space-x-2">
                        <span className="text-sm font-medium">
                          检测 #{reverseIndex + 1}
                        </span>
                        <Badge variant="outline" className="text-xs">
                          共 {totalCount} 个
                        </Badge>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="h-6 w-6 p-0"
                        onClick={(e) => {
                          e.stopPropagation()
                          onSelectResult(result)
                        }}
                      >
                        <Eye className="w-3 h-3" />
                      </Button>
                    </div>
                    
                    <div className="flex items-center space-x-2 mb-2">
                      {abnormalCount > 0 && (
                        <Badge variant="destructive" className="text-xs">
                          异常 {abnormalCount}
                        </Badge>
                      )}
                      {normalCount > 0 && (
                        <Badge variant="secondary" className="text-xs">
                          正常 {normalCount}
                        </Badge>
                      )}
                    </div>
                    
                    <p className="text-xs text-muted-foreground">
                      {new Date(result.timestamp).toLocaleString()}
                    </p>
                  </div>
                )
              })}
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  )
}