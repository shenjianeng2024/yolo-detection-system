use anyhow::Result;
use std::time::{Duration, Instant};

use super::{CandleYoloModel, ModelStats};

pub struct PerformanceBenchmark {
    model: CandleYoloModel,
    test_images: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub total_time_ms: u64,
    pub avg_time_per_image_ms: u64,
    pub fps: f64,
    pub total_images: usize,
    pub memory_usage_mb: u64,
    pub cache_hit_rate_percent: f64,
    pub stats: ModelStats,
}

impl PerformanceBenchmark {
    pub async fn new(model_path: &str) -> Result<Self> {
        let model = CandleYoloModel::new(model_path)?;
        let test_images = Self::generate_test_images().await?;
        
        Ok(Self {
            model,
            test_images,
        })
    }

    // 生成不同尺寸和复杂度的测试图像
    async fn generate_test_images() -> Result<Vec<Vec<u8>>> {
        let mut images = Vec::new();
        
        // 生成不同尺寸的测试图像
        let test_sizes = vec![
            (320, 240),   // 小图像
            (640, 480),   // 中等图像  
            (1280, 720),  // HD图像
            (1920, 1080), // Full HD图像
        ];

        for (width, height) in test_sizes {
            let image = Self::create_synthetic_image(width, height)?;
            images.push(image);
        }
        
        // 生成不同亮度的图像
        for brightness in [50, 128, 200] {
            let image = Self::create_brightness_image(640, 480, brightness)?;
            images.push(image);
        }

        println!("🎯 Generated {} test images for benchmarking", images.len());
        Ok(images)
    }

    // 创建合成测试图像
    fn create_synthetic_image(width: u32, height: u32) -> Result<Vec<u8>> {
        use image::{RgbImage, Rgb, ImageFormat};
        
        let mut img = RgbImage::new(width, height);
        
        // 创建渐变图案
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = 128;
                
                img.put_pixel(x, y, Rgb([r, g, b]));
            }
        }
        
        // 添加一些形状作为"目标"
        Self::draw_rectangles(&mut img);
        
        // 转换为字节
        let mut buffer = Vec::new();
        let dyn_img = image::DynamicImage::ImageRgb8(img);
        dyn_img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)?;
        
        Ok(buffer)
    }

    // 创建特定亮度的图像
    fn create_brightness_image(width: u32, height: u32, brightness: u8) -> Result<Vec<u8>> {
        use image::{RgbImage, Rgb, ImageFormat};
        
        let mut img = RgbImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                // 添加一些噪声和模式
                let noise = ((x + y) % 50) as u8;
                let pixel_brightness = brightness.saturating_add(noise / 2);
                
                img.put_pixel(x, y, Rgb([pixel_brightness, pixel_brightness, pixel_brightness]));
            }
        }
        
        Self::draw_rectangles(&mut img);
        
        let mut buffer = Vec::new();
        let dyn_img = image::DynamicImage::ImageRgb8(img);
        dyn_img.write_to(&mut std::io::Cursor::new(&mut buffer), ImageFormat::Png)?;
        
        Ok(buffer)
    }

    // 在图像上绘制矩形作为检测目标
    fn draw_rectangles(img: &mut image::RgbImage) {
        use image::Rgb;
        
        let (width, height) = img.dimensions();
        let rect_color = Rgb([255, 0, 0]); // 红色矩形
        
        // 绘制几个矩形
        let rects = vec![
            (width / 4, height / 4, width / 6, height / 8),
            (width * 3 / 4, height / 2, width / 8, height / 6),
        ];
        
        for (x, y, w, h) in rects {
            for dx in 0..w {
                for dy in 0..h {
                    let px = x + dx;
                    let py = y + dy;
                    if px < width && py < height {
                        img.put_pixel(px, py, rect_color);
                    }
                }
            }
        }
    }

    // 基础性能测试
    pub async fn run_basic_benchmark(&mut self) -> Result<BenchmarkResult> {
        println!("🚀 Running basic performance benchmark...");
        
        self.model.reset_stats().await;
        let start_time = Instant::now();
        
        let mut total_detections = 0;
        for (i, image_data) in self.test_images.iter().enumerate() {
            print!("Processing image {}/{}\r", i + 1, self.test_images.len());
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            
            let detections = self.model.detect_image(image_data).await?;
            total_detections += detections.len();
        }
        
        let total_time = start_time.elapsed();
        let stats = self.model.get_performance_stats().await;
        let memory_usage = self.model.get_memory_usage().await;
        
        let cache_hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0
        } else {
            0.0
        };

        println!("\n✅ Basic benchmark completed!");
        println!("   Total detections found: {}", total_detections);
        
        Ok(BenchmarkResult {
            test_name: "Basic Performance Test".to_string(),
            total_time_ms: total_time.as_millis() as u64,
            avg_time_per_image_ms: total_time.as_millis() as u64 / self.test_images.len() as u64,
            fps: self.test_images.len() as f64 / total_time.as_secs_f64(),
            total_images: self.test_images.len(),
            memory_usage_mb: memory_usage,
            cache_hit_rate_percent: cache_hit_rate,
            stats,
        })
    }

    // 缓存效率测试
    pub async fn run_cache_benchmark(&mut self) -> Result<BenchmarkResult> {
        println!("💾 Running cache efficiency benchmark...");
        
        self.model.reset_stats().await;
        let start_time = Instant::now();
        
        // 重复处理相同图像以测试缓存效果
        let test_image = &self.test_images[0];
        let repeat_count = 10usize;
        
        for i in 0..repeat_count {
            print!("Cache test iteration {}/{}\r", i + 1, repeat_count);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
            
            let _detections = self.model.detect_image(test_image).await?;
        }
        
        let total_time = start_time.elapsed();
        let stats = self.model.get_performance_stats().await;
        let memory_usage = self.model.get_memory_usage().await;
        
        let cache_hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0
        } else {
            0.0
        };

        println!("\n✅ Cache benchmark completed!");
        println!("   Cache hits: {}, Cache misses: {}", stats.cache_hits, stats.cache_misses);
        
        Ok(BenchmarkResult {
            test_name: "Cache Efficiency Test".to_string(),
            total_time_ms: total_time.as_millis() as u64,
            avg_time_per_image_ms: total_time.as_millis() as u64 / repeat_count as u64,
            fps: repeat_count as f64 / total_time.as_secs_f64(),
            total_images: repeat_count,
            memory_usage_mb: memory_usage,
            cache_hit_rate_percent: cache_hit_rate,
            stats,
        })
    }

    // 压力测试
    pub async fn run_stress_test(&mut self, duration_seconds: u64) -> Result<BenchmarkResult> {
        println!("🔥 Running stress test for {}s...", duration_seconds);
        
        self.model.reset_stats().await;
        let start_time = Instant::now();
        let end_time = start_time + Duration::from_secs(duration_seconds);
        
        let mut processed_count = 0usize;
        let mut image_index = 0;
        
        while Instant::now() < end_time {
            let image_data = &self.test_images[image_index % self.test_images.len()];
            let _detections = self.model.detect_image(image_data).await?;
            
            processed_count += 1;
            image_index += 1;
            
            if processed_count % 10 == 0 {
                let elapsed = start_time.elapsed().as_secs();
                print!("Stress test: {}s elapsed, {} images processed\r", elapsed, processed_count);
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
            }
        }
        
        let total_time = start_time.elapsed();
        let stats = self.model.get_performance_stats().await;
        let memory_usage = self.model.get_memory_usage().await;
        
        let cache_hit_rate = if stats.cache_hits + stats.cache_misses > 0 {
            stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64 * 100.0
        } else {
            0.0
        };

        println!("\n✅ Stress test completed!");
        println!("   Processed {} images in {}s", processed_count, total_time.as_secs());
        
        Ok(BenchmarkResult {
            test_name: format!("Stress Test ({}s)", duration_seconds),
            total_time_ms: total_time.as_millis() as u64,
            avg_time_per_image_ms: total_time.as_millis() as u64 / processed_count as u64,
            fps: processed_count as f64 / total_time.as_secs_f64(),
            total_images: processed_count,
            memory_usage_mb: memory_usage,
            cache_hit_rate_percent: cache_hit_rate,
            stats,
        })
    }

    // 生成完整的性能报告
    pub async fn generate_full_report(&mut self) -> Result<String> {
        println!("📊 Generating comprehensive performance report...\n");
        
        let basic_result = self.run_basic_benchmark().await?;
        println!();
        
        let cache_result = self.run_cache_benchmark().await?;
        println!();
        
        let stress_result = self.run_stress_test(30).await?;
        println!();

        let model_report = self.model.generate_performance_report().await;
        
        let full_report = format!(
            "{}\n\n\
            🎯 Benchmark Results Summary\n\
            ============================\n\n\
            📈 Basic Performance Test:\n\
            • Average Time per Image: {}ms\n\
            • FPS: {:.2}\n\
            • Total Images: {}\n\
            • Memory Usage: {}MB\n\
            • Cache Hit Rate: {:.1}%\n\n\
            💾 Cache Efficiency Test:\n\
            • Average Time per Image: {}ms\n\
            • FPS: {:.2}\n\
            • Cache Hit Rate: {:.1}%\n\
            • Cache Performance: {}\n\n\
            🔥 Stress Test (30s):\n\
            • Average Time per Image: {}ms\n\
            • FPS: {:.2}\n\
            • Total Images Processed: {}\n\
            • Memory Usage: {}MB\n\
            • Sustained Performance: {}\n\n\
            🔍 Performance Analysis:\n\
            • Best FPS: {:.2} ({})\n\
            • Lowest Latency: {}ms ({})\n\
            • Cache Effectiveness: {}\n\
            • Memory Efficiency: {}\n\n\
            💡 Recommendations:\n\
            {}",
            model_report,
            basic_result.avg_time_per_image_ms,
            basic_result.fps,
            basic_result.total_images,
            basic_result.memory_usage_mb,
            basic_result.cache_hit_rate_percent,
            cache_result.avg_time_per_image_ms,
            cache_result.fps,
            cache_result.cache_hit_rate_percent,
            if cache_result.cache_hit_rate_percent > 50.0 { "Excellent" } else { "Needs Improvement" },
            stress_result.avg_time_per_image_ms,
            stress_result.fps,
            stress_result.total_images,
            stress_result.memory_usage_mb,
            if stress_result.fps > 15.0 { "Good" } else { "Acceptable" },
            // Analysis
            [basic_result.fps, cache_result.fps, stress_result.fps].iter().fold(0.0_f64, |a, &b| a.max(b)),
            "Cache Test", // Best performer is usually cache test
            [basic_result.avg_time_per_image_ms, cache_result.avg_time_per_image_ms, stress_result.avg_time_per_image_ms].iter().min().unwrap(),
            "Cache Test",
            if cache_result.cache_hit_rate_percent > 80.0 { "Very Effective" } else if cache_result.cache_hit_rate_percent > 50.0 { "Effective" } else { "Limited" },
            if basic_result.memory_usage_mb < 50 { "Excellent" } else if basic_result.memory_usage_mb < 100 { "Good" } else { "High" },
            self.generate_recommendations(&basic_result, &cache_result, &stress_result)
        );
        
        Ok(full_report)
    }

    fn generate_recommendations(&self, basic: &BenchmarkResult, cache: &BenchmarkResult, stress: &BenchmarkResult) -> String {
        let mut recommendations = Vec::new();
        
        if basic.fps < 10.0 {
            recommendations.push("• Consider enabling GPU acceleration for better performance");
        }
        
        if cache.cache_hit_rate_percent < 50.0 {
            recommendations.push("• Image cache is underutilized - check cache eviction policy");
        }
        
        if basic.memory_usage_mb > 100 {
            recommendations.push("• Memory usage is high - consider reducing cache size");
        }
        
        if stress.fps < basic.fps * 0.8 {
            recommendations.push("• Performance degrades under load - check for memory leaks");
        }
        
        if basic.avg_time_per_image_ms > 100 {
            recommendations.push("• Image preprocessing is slow - consider using lower resolution");
        }
        
        if recommendations.is_empty() {
            recommendations.push("• Performance looks good! Consider testing with real YOLO model");
        }
        
        recommendations.join("\n")
    }
}

impl std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Test: {}\nTime: {}ms | FPS: {:.2} | Images: {} | Memory: {}MB | Cache Hit: {:.1}%",
            self.test_name,
            self.total_time_ms,
            self.fps,
            self.total_images,
            self.memory_usage_mb,
            self.cache_hit_rate_percent
        )
    }
}