"use client"

import * as React from "react"
import { cn } from "@/lib/utils"
import { Upload, Image as ImageIcon, X, FileText } from "lucide-react"
import { Button } from "./button"

interface FileInfo {
  name: string
  size: number
  type: string
  lastModified: number
}

interface FileUploadProps {
  onFileSelect: (file: File, fileInfo: FileInfo) => void
  onClear?: () => void
  accept?: string
  disabled?: boolean
  loading?: boolean
  maxSize?: number // in bytes
  className?: string
  selectedFile?: FileInfo | null
  error?: string | null
  disableClick?: boolean // 禁用点击交互，只保留拖拽功能
}

const FileUpload = React.forwardRef<HTMLDivElement, FileUploadProps>(
  ({ 
    onFileSelect, 
    onClear,
    accept = "image/*", 
    disabled = false,
    loading = false,
    maxSize = 10 * 1024 * 1024, // 10MB
    className,
    selectedFile,
    error,
    disableClick = false,
    ...props 
  }, ref) => {
    const [isDragOver, setIsDragOver] = React.useState(false)
    const fileInputRef = React.useRef<HTMLInputElement>(null)

    const formatFileSize = (bytes: number) => {
      if (bytes === 0) return '0 Bytes'
      const k = 1024
      const sizes = ['Bytes', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    const handleFileSelect = (file: File) => {
      if (file.size > maxSize) {
        return
      }

      const fileInfo: FileInfo = {
        name: file.name,
        size: file.size,
        type: file.type,
        lastModified: file.lastModified
      }

      onFileSelect(file, fileInfo)
    }

    const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0]
      if (file) {
        handleFileSelect(file)
      }
    }

    const handleDrop = (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault()
      setIsDragOver(false)

      if (disabled || loading) return

      const files = event.dataTransfer.files
      if (files.length > 0) {
        handleFileSelect(files[0])
      }
    }

    const handleDragOver = (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault()
      if (!disabled && !loading) {
        setIsDragOver(true)
      }
    }

    const handleDragLeave = (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault()
      setIsDragOver(false)
    }

    const handleClick = () => {
      if (!disabled && !loading && !disableClick) {
        fileInputRef.current?.click()
      }
    }

    const handleClear = () => {
      if (fileInputRef.current) {
        fileInputRef.current.value = ''
      }
      onClear?.()
    }

    return (
      <div
        ref={ref}
        className={cn(
          "relative border-2 border-dashed rounded-lg p-6 transition-all duration-200",
          isDragOver && !disabled && !loading
            ? "border-blue-400 bg-blue-50 dark:bg-blue-950/20"
            : "border-gray-300 dark:border-gray-600",
          disabled || loading
            ? "opacity-50 cursor-not-allowed"
            : disableClick
            ? "cursor-default" // 禁用点击时使用默认光标
            : "cursor-pointer hover:border-gray-400 dark:hover:border-gray-500",
          error && "border-red-400 bg-red-50 dark:bg-red-950/20",
          className
        )}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        {...(!disableClick && { onClick: handleClick })}
        {...props}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept={accept}
          onChange={handleFileChange}
          disabled={disabled || loading}
          className="hidden"
        />

        {selectedFile ? (
          <div className="flex items-start justify-between">
            <div className="flex items-center gap-3">
              {selectedFile.type.startsWith('image/') ? (
                <ImageIcon className="w-8 h-8 text-blue-600" />
              ) : (
                <FileText className="w-8 h-8 text-gray-600" />
              )}
              <div>
                <p className="font-medium text-gray-900 dark:text-gray-100">
                  {selectedFile.name}
                </p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  {formatFileSize(selectedFile.size)} • {new Date(selectedFile.lastModified).toLocaleString()}
                </p>
              </div>
            </div>
            {onClear && (
              <Button
                variant="ghost"
                size="sm"
                onClick={(e) => {
                  e.stopPropagation()
                  handleClear()
                }}
                className="text-gray-400 hover:text-gray-600"
              >
                <X className="w-4 h-4" />
              </Button>
            )}
          </div>
        ) : (
          <div className="text-center">
            <div className="flex justify-center mb-4">
              {loading ? (
                <div className="w-12 h-12 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />
              ) : (
                <Upload className={cn(
                  "w-12 h-12",
                  isDragOver ? "text-blue-600" : "text-gray-400"
                )} />
              )}
            </div>
            <div className="space-y-2">
              <p className="text-lg font-medium text-gray-900 dark:text-gray-100">
                {loading ? "处理中..." : disableClick ? "视频/图片显示" : isDragOver ? "释放文件获取提示" : "拖拽图片获取提示"}
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {loading ? "正在处理您的图片..." : disableClick ? "请使用下方按钮选择文件" : "为获取正确路径，请使用下方'选择图片'按钮"}
              </p>
              {!disableClick && (
                <p className="text-xs text-blue-600 dark:text-blue-400 mt-2">
                  💡 拖拽文件后会提示您使用正确的选择方式
                </p>
              )}
            </div>
          </div>
        )}

        {error && (
          <div className="mt-4 p-3 bg-red-100 dark:bg-red-900/20 border border-red-200 dark:border-red-700 rounded-md">
            <p className="text-sm text-red-700 dark:text-red-300">{error}</p>
          </div>
        )}
      </div>
    )
  }
)

FileUpload.displayName = "FileUpload"

export { FileUpload, type FileInfo }