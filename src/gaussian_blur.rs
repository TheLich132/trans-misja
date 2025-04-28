use crate::settings::FunctionsSettings;

use image::imageops;
use image::{ImageBuffer, Rgb};

pub fn selective_gaussian_blur(
    image_path: &str,
    settings: &FunctionsSettings,
) -> Result<String, String> {
    // Load the image and convert to RGB8
    let img = image::open(image_path).map_err(|e| e.to_string())?;
    let img = img.to_rgb8();
    let (width, height) = img.dimensions();
    print!("Image dimensions: {}x{}\n", width, height);

    // Create the blurred version of the image
    let blurred = imageops::blur(&img, settings.blur_sigma);

    // Thresholds for region detection:
    let brightness_threshold: f32 = settings.brightness_threshold; // areas with low average intensity (signal loss)
    let noise_threshold: f32 = settings.noise_threshold; // areas with high local variability (noise)

    // Create an output image buffer
    let mut output = ImageBuffer::new(width, height);

    // Process each pixel
    for y in 0..height {
        for x in 0..width {
            let current = img.get_pixel(x, y);
            let (mean, std_dev) = neighborhood_stats(&img, x, y, width, height);

            // If the region is dark (signal loss) or noisy, use the blurred pixel.
            if mean < brightness_threshold || std_dev > noise_threshold {
                output.put_pixel(x, y, *blurred.get_pixel(x, y));
            } else {
                output.put_pixel(x, y, *current);
            }
        }
    }

    // Sharp the image to enhance edges
    let sharpened =
        imageops::unsharpen(&output, settings.sharpen_sigma, settings.sharpen_threshold);
    // Save the sharpened image
    output = sharpened;

    // Save the resulting image
    let output_path = format!("selective_blur_{}", image_path);
    println!("Saving output image to: {}", output_path);
    output.save(&output_path).map_err(|e| e.to_string())?;
    Ok(output_path)
}

/// Helper function to compute the average intensity and standard deviation
/// of a pixel's 3x3 neighborhood.
fn neighborhood_stats(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> (f32, f32) {
    let mut sum = 0.0;
    let mut count = 0;
    let mut intensities = Vec::new();

    // Consider a 3x3 neighborhood.
    let x_start = x.saturating_sub(1);
    let x_end = (x + 1).min(width - 1);
    let y_start = y.saturating_sub(1);
    let y_end = (y + 1).min(height - 1);

    for j in y_start..=y_end {
        for i in x_start..=x_end {
            let pixel = img.get_pixel(i, j);
            // Compute a simple intensity as the average of R, G and B.
            let intensity = (pixel[0] as f32 + pixel[1] as f32 + pixel[2] as f32) / 3.0;
            sum += intensity;
            intensities.push(intensity);
            count += 1;
        }
    }

    let mean = sum / count as f32;
    let mut variance_sum = 0.0;
    for intensity in &intensities {
        variance_sum += (intensity - mean).powi(2);
    }

    let std_dev = (variance_sum / count as f32).sqrt();
    (mean, std_dev)
}
