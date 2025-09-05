use anyhow::Result;
use image::{ImageBuffer, Rgb, RgbImage};
use ndarray::Array4;

pub fn preprocess_image(img: &RgbImage, target_width: usize, target_height: usize) -> Result<Array4<f32>> {
    // 1. 调整图像大小（保持宽高比）
    let resized = resize_with_padding(img, target_width as u32, target_height as u32);
    
    // 2. 转换为float32并规范化到[0,1]
    let mut input_array = Array4::<f32>::zeros((1, 3, target_height, target_width));
    
    for y in 0..target_height {
        for x in 0..target_width {
            let pixel = resized.get_pixel(x as u32, y as u32);
            
            // RGB -> 规范化 [0, 1]
            input_array[[0, 0, y, x]] = pixel[0] as f32 / 255.0;
            input_array[[0, 1, y, x]] = pixel[1] as f32 / 255.0;
            input_array[[0, 2, y, x]] = pixel[2] as f32 / 255.0;
        }
    }
    
    Ok(input_array)
}

fn resize_with_padding(img: &RgbImage, target_width: u32, target_height: u32) -> RgbImage {
    let (orig_width, orig_height) = img.dimensions();
    
    // 计算缩放比例
    let scale = (target_width as f32 / orig_width as f32)
        .min(target_height as f32 / orig_height as f32);
    
    let new_width = (orig_width as f32 * scale) as u32;
    let new_height = (orig_height as f32 * scale) as u32;
    
    // 调整图像大小
    let resized = image::imageops::resize(
        img,
        new_width,
        new_height,
        image::imageops::FilterType::Lanczos3
    );
    
    // 创建目标图像并居中放置
    let mut result = ImageBuffer::new(target_width, target_height);
    
    // 填充灰色背景（114, 114, 114） - YOLOv8标准
    for pixel in result.pixels_mut() {
        *pixel = Rgb([114, 114, 114]);
    }
    
    // 计算居中位置
    let offset_x = (target_width - new_width) / 2;
    let offset_y = (target_height - new_height) / 2;
    
    // 复制调整大小后的图像到中心位置
    for y in 0..new_height {
        for x in 0..new_width {
            if let Some(pixel) = resized.get_pixel_checked(x, y) {
                result.put_pixel(offset_x + x, offset_y + y, *pixel);
            }
        }
    }
    
    result
}