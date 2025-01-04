use gtk4::ProgressBar;
use hound::WavReader;
use image::{GenericImageView, GrayImage, ImageBuffer, Luma};
use rayon::prelude::*;
use std::{error::Error, sync::Mutex};
use tract_onnx::prelude::*;
use std::time::Instant;

pub fn compute_signal(
    filepath: &str,
    debug: &bool,
    sync: &bool,
    use_model: &bool,
    progress_bar: &ProgressBar,
) -> String {
    // Start timer
    let start = Instant::now();

    println!("Debug: {}, Sync: {}, Use model: {}", debug, sync, use_model);

    // Update progress bar
    progress_bar.set_fraction(0.1);
    progress_bar.set_text(Some("Loading WAV file..."));

    /*
        Loading wav files with hound
    */
    let mut reader = WavReader::open(filepath).unwrap();
    let spec = reader.spec();
    if *debug {
        println!("Wav file: {}", filepath);
        println!("Sample rate: {}", spec.sample_rate);
        println!("Channels: {}", spec.channels);
        println!("Sample format: {:?}", spec.sample_format);
    }

    let target_sample_rate = 20800;

    let mut samples: Vec<f32> = Vec::new();
    if spec.sample_format == hound::SampleFormat::Float {
        let samples_float = match reader
            .samples::<f32>()
            .collect::<Result<Vec<f32>, hound::Error>>()
        {
            Ok(samples) => samples,
            Err(e) => {
                eprintln!("Error reading samples: {}", e);
                return String::from("Error reading samples");
            }
        };
        let channels = spec.channels as usize;
        for i in (0..samples_float.len()).step_by(channels) {
            samples.push(samples_float[i]);
        }
    } else if spec.sample_format == hound::SampleFormat::Int {
        let samples_int = match reader
            .samples::<i32>()
            .collect::<Result<Vec<i32>, hound::Error>>()
        {
            Ok(samples) => samples,
            Err(e) => {
                eprintln!("Error reading samples: {}", e);
                return String::from("Error reading samples");
            }
        };
        let channels = spec.channels as usize;
        for i in (0..samples_int.len()).step_by(channels) {
            samples.push(samples_int[i] as f32);
        }
    }

    // Update progress bar
    progress_bar.set_fraction(0.3);
    progress_bar.set_text(Some("Processing samples..."));

    println!("Samples: {}", samples.len());
    for sample in samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");

    // Resampling
    let ratio = target_sample_rate as f64 / spec.sample_rate as f64;
    let resampled_samples = resample_signal(samples, ratio);

    // Update progress bar
    progress_bar.set_fraction(0.5);
    progress_bar.set_text(Some("Resampling..."));

    println!("Resampled samples: {}", resampled_samples.len());
    for sample in resampled_samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");

    let frequency = target_sample_rate as f32;

    let filtered_signal = low_pass_filter(&resampled_samples, 5000.0, frequency);

    // Update progress bar
    progress_bar.set_fraction(0.7);
    progress_bar.set_text(Some("Filtering signal..."));

    println!("Demodulating...");
    let am_signal = envelope_detection(&filtered_signal, 10, 2.0);

    // Update progress bar
    progress_bar.set_fraction(0.8);
    progress_bar.set_text(Some("Demodulating..."));

    // APT Signal sync
    let path: String = if *sync {
        println!("Syncing...");
        let frame_width = (frequency * 0.5) as usize;

        // Sync pattern for APT signal
        // [..WW..WW..WW..WW..WW..WW..WW........]
        let sync_pattern = vec![
            -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0,
            -1.0, -1.0, -1.0, -1.0, -1.0,
        ];
        let synced_signal = sync_apt(&am_signal, frame_width, &sync_pattern);

        match generate_image(&synced_signal, frequency, 5) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error generating image: {}", e);
                return String::from("Error generating image");
            }
        }
    } else {
        match generate_image(&am_signal, frequency, 5) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error generating image: {}", e);
                return String::from("Error generating image");
            }
        }
    };

    // Update progress bar
    progress_bar.set_fraction(0.9);
    progress_bar.set_text(Some("Generating image..."));

    if *use_model {
        println!("Enhancing image...");
        let model_path = "model.onnx";
        let enhanced_image_path = enhance_image_with_model(&path, model_path).unwrap();
        progress_bar.set_fraction(1.0);
        progress_bar.set_text(Some("Enhancement complete"));

        // Stop timer
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);

        enhanced_image_path
    } else {
        progress_bar.set_fraction(1.0);
        progress_bar.set_text(Some("Processing complete"));

        // Stop timer
        let duration = start.elapsed();
        println!("Time elapsed: {:?}", duration);
        
        path
    }
}

fn resample_signal(samples: Vec<f32>, ratio: f64) -> Vec<f32> {
    let target_len = (samples.len() as f64 * ratio) as usize;
    (0..target_len)
        .filter_map(|i| {
            let index = (i as f64 / ratio) as usize;
            if index + 1 >= samples.len() {
                None
            } else {
                let x = (i as f64 / ratio) - index as f64;
                let y = samples[index] + x as f32 * (samples[index + 1] - samples[index]);
                Some(y)
            }
        })
        .collect()
}

fn low_pass_filter(samples: &[f32], cutoff_freq: f32, sample_rate: f32) -> Vec<f32> {
    assert!(
        cutoff_freq > 0.0 && cutoff_freq < sample_rate / 2.0,
        "Invalid cutoff frequency"
    );
    assert!(sample_rate > 0.0, "Sample rate must be positive");

    let rc = 1.0 / (cutoff_freq * 2.0 * std::f32::consts::PI);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let filtered_samples: Vec<f32> = samples
        .iter()
        .scan(samples[0], |prev, &sample| {
            let filtered = *prev + alpha * (sample - *prev);
            *prev = filtered;
            Some(filtered)
        })
        .collect();

    filtered_samples
}

fn normalize_image(image: &mut GrayImage) {
    let max_value = *image.iter().max().unwrap();
    let min_value = *image.iter().min().unwrap();

    for pixel in image.iter_mut() {
        *pixel = (((*pixel as f32 - min_value as f32) / (max_value as f32 - min_value as f32))
            * 255.0) as u8;
    }
}

fn find_sync_position(signal: &[f32], sync_pattern: &[f32]) -> usize {
    let sync_len = sync_pattern.len();
    let signal_len = signal.len();

    if sync_len == 0 || signal_len == 0 || sync_len > signal_len {
        return 0; // Return 0 if input is invalid
    }

    let mut best_offset = 0;
    let mut best_score = f32::MIN;

    for offset in 0..=(signal_len - sync_len) {
        let (score, signal_energy, pattern_energy) = (0..sync_len).fold(
            (0.0, 0.0, 0.0),
            |(score, signal_energy, pattern_energy), i| {
                (
                    score + signal[offset + i] * sync_pattern[i],
                    signal_energy + signal[offset + i] * signal[offset + i],
                    pattern_energy + sync_pattern[i] * sync_pattern[i],
                )
            },
        );

        let normalized_score = score / (signal_energy.sqrt() * pattern_energy.sqrt());
        if normalized_score > best_score {
            best_score = normalized_score;
            best_offset = offset;
        }
    }

    best_offset
}

fn sync_apt(signal: &[f32], frame_width: usize, sync_pattern: &[f32]) -> Vec<f32> {
    let mut synced = Vec::with_capacity(signal.len());
    let rows = signal.len() / frame_width;
    const ADDITIONAL_OFFSET: usize = 120; // Adjust this value as needed

    for r in 0..rows {
        let row_start = r * frame_width;
        let row_end = row_start + frame_width.min(signal.len() - row_start);
        let row_slice = &signal[row_start..row_end];

        // Find best correlation offset using find_sync_position
        let best_offset = find_sync_position(row_slice, sync_pattern);

        // Fine-tune the alignment by checking a small range around the best offset
        let fine_tune_range = 5;
        let mut fine_tuned_offset = best_offset;
        let mut best_fine_tuned_score = f32::MIN;
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for offset in (best_offset.saturating_sub(fine_tune_range))
            ..=(best_offset + fine_tune_range).min(row_slice.len() - sync_pattern.len())
        {
            let (score, signal_energy, pattern_energy) = (0..sync_pattern.len()).fold(
                (0.0, 0.0, 0.0),
                |(score, signal_energy, pattern_energy), i| {
                    (
                        score + row_slice[offset + i] * sync_pattern[i],
                        signal_energy + row_slice[offset + i] * row_slice[offset + i],
                        pattern_energy + sync_pattern[i] * sync_pattern[i],
                    )
                },
            );

            let normalized_score = score / (signal_energy.sqrt() * pattern_energy.sqrt());
            if normalized_score > best_fine_tuned_score {
                best_fine_tuned_score = normalized_score;
                fine_tuned_offset = offset;
            }

            // Calculate weighted sum for fine-tuning
            weighted_sum += offset as f32 * normalized_score;
            weight_total += normalized_score;
        }

        // Calculate weighted average offset
        if weight_total > 0.0 {
            fine_tuned_offset = (weighted_sum / weight_total).round() as usize;
        }

        // Add additional offset to ensure the row starts with sync A bar
        fine_tuned_offset = fine_tuned_offset
            .saturating_sub(ADDITIONAL_OFFSET)
            .min(row_slice.len());

        // Circular shift from fine-tuned offset
        println!(
            "Best offset: {}, Fine-tuned offset: {}",
            best_offset, fine_tuned_offset
        );
        synced.extend_from_slice(&row_slice[fine_tuned_offset..]);
        synced.extend_from_slice(&row_slice[..fine_tuned_offset]);
    }

    synced
}

fn envelope_detection(signal: &[f32], window_size: usize, scaling_factor: f32) -> Vec<f32> {
    let mut envelope: Vec<f32> = Vec::with_capacity(signal.len());
    for i in 0..signal.len() {
        let mut max: f32 = 0.0; // specify the type of max explicitly
        let end = (i + window_size).min(signal.len());
        for sample in signal.iter().take(end).skip(i) {
            max = max.max(sample.abs());
        }
        envelope.push(max * scaling_factor);
    }
    envelope
}

fn generate_image(
    signal: &[f32],
    frequency: f32,
    reduction_factor: u32,
) -> Result<String, Box<dyn Error>> {
    const SCALE_FACTOR: f32 = 32.0;
    const MAX_LUMINANCE: f32 = 255.0;

    let frame_width = (frequency * 0.5) as u32;
    println!("Frame width: {}", frame_width);
    let w = frame_width;
    let h = (signal.len() / frame_width as usize) as u32;
    println!("Width: {}, Height: {}", w, h);

    let mut img: GrayImage = ImageBuffer::new(w, h);

    let mut px = 0;
    let mut py = 0;

    for &sample in signal.iter() {
        let mut lum = sample / SCALE_FACTOR - SCALE_FACTOR;
        lum = lum.clamp(0.0, MAX_LUMINANCE);
        img.put_pixel(px, py, Luma([lum as u8]));
        px += 1;
        if px >= w {
            px = 0;
            py += 1;
        }

        if py >= h {
            break;
        }
    }

    // Reduce image width by the reduction factor
    let new_w = w / reduction_factor;
    let mut img_resized: GrayImage = ImageBuffer::new(new_w, h);

    for px in 0..new_w {
        for py in 0..h {
            let orig_px = px * reduction_factor;
            img_resized.put_pixel(px, py, *img.get_pixel(orig_px, py));
        }
    }
    img = img_resized;

    normalize_image(&mut img);

    img.save("image.png")?;

    Ok(String::from("image.png"))
}

fn enhance_image_with_model(
    image_path: &str,
    model_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Load the ONNX model
    let model = tract_onnx::onnx()
        .model_for_path(model_path)?
        .with_input_fact(
            0,
            InferenceFact::dt_shape(f32::datum_type(), tvec![1, 1, 256, 256]),
        )?
        .into_optimized()?
        .into_runnable()?;

    // Load and preprocess the image
    let image = image::open(image_path)?.to_luma8();
    let (width, height) = image.dimensions();
    let patch_size = 256;
    let step_size = patch_size;

    // Create a blank output image
    let output_image = Mutex::new(GrayImage::new(width, height));

    // Process patches in parallel
    (0..height)
        .into_par_iter()
        .step_by(step_size)
        .for_each(|i| {
            (0..width).into_par_iter().step_by(step_size).for_each(|j| {
                println!("Processing patch at ({}, {})", j, i);
                // Extract the patch from the image
                let patch = image
                    .view(
                        j,
                        i,
                        (patch_size as u32).min(width - j),
                        (patch_size as u32).min(height - i),
                    )
                    .to_image();

                // Pad the patch to the full patch size if necessary
                let mut padded_patch = GrayImage::new(patch_size as u32, patch_size as u32);
                for y in 0..patch.height() {
                    for x in 0..patch.width() {
                        padded_patch.put_pixel(x, y, *patch.get_pixel(x, y));
                    }
                }

                // Convert the patch to a tensor
                let tensor: Tensor = tract_ndarray::Array4::from_shape_fn(
                    (1, 1, patch_size, patch_size),
                    |(_, _, y, x)| padded_patch.get_pixel(x as u32, y as u32).0[0] as f32 / 255.0,
                )
                .into();

                // Run the model
                let result = model.run(tvec!(tensor.into())).unwrap();
                let output_patch: tract_ndarray::ArrayView4<f32> = result[0]
                    .to_array_view::<f32>()
                    .unwrap()
                    .into_dimensionality()
                    .unwrap();

                // Copy the output patch to the output image
                let mut output_image = output_image.lock().unwrap();
                for y in 0..patch.height() {
                    for x in 0..patch.width() {
                        let value =
                            (output_patch[[0, 0, y as usize, x as usize]] * 255.0).round() as u8;
                        output_image.put_pixel(j + x, i + y, Luma([value]));
                    }
                }
            });
        });

    output_image
        .lock()
        .unwrap()
        .save("enhanced_image.png")
        .unwrap();

    Ok(String::from("enhanced_image.png"))
}
