use hound::WavReader;
use image::{GrayImage, ImageBuffer, Luma};

pub fn compute_signal(filepath: &str, debug: &bool, sync: &bool) -> String {
    println!("Debug: {}, Sync: {}", debug, sync);

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
        let samples_float = reader
            .samples::<f32>()
            .collect::<Result<Vec<f32>, hound::Error>>()
            .unwrap();
        let channels = spec.channels as usize;
        let mut i = 0;
        while i < samples_float.len() {
            if i % channels == 0 {
                samples.push(samples_float[i]);
            }
            i += 1;
        }
    } else if spec.sample_format == hound::SampleFormat::Int {
        let samples_int = reader
            .samples::<i16>()
            .collect::<Result<Vec<i16>, hound::Error>>()
            .unwrap();
        let channels = spec.channels as usize;
        let mut i = 0;
        while i < samples_int.len() {
            if i % channels == 0 {
                samples.push(samples_int[i] as f32);
            }
            i += 1;
        }
    }

    println!("Samples: {}", samples.len());
    for sample in samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");

    // Resampling
    let ratio = target_sample_rate as f64 / spec.sample_rate as f64;
    let resampled_samples = resample_signal(samples, ratio);

    println!("Resampled samples: {}", resampled_samples.len());
    for sample in resampled_samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");

    let frequency = target_sample_rate as f32;

    let filtered_signal = low_pass_filter(&resampled_samples, 5000.0, frequency);

    println!("Demodulating...");
    let am_signal = envelope_detection(&filtered_signal);

    // APT Signal sync
    if *sync {
        println!("Syncing...");
        let frame_width = (frequency * 0.5) as usize;
        
        // Sync pattern for APT signal
        // [..WW..WW..WW..WW..WW..WW..WW........]
        let sync_pattern = vec![
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, 1.0, 1.0,
            -1.0, -1.0, -1.0, -1.0,
            -1.0, -1.0, -1.0, -1.0,
        ];
        let synced_signal = sync_apt(&am_signal, frame_width, &sync_pattern);

        let path = generate_image(&synced_signal, frequency);
        return path;
    } else {
        let path = generate_image(&am_signal, frequency);
        return path;
    }
}

fn resample_signal(samples: Vec<f32>, ratio: f64) -> Vec<f32> {
    let mut resampled_samples: Vec<f32> = Vec::new();
    for i in 0..(samples.len() as f64 * ratio) as usize {
        let index = (i as f64 / ratio) as usize;
        if index < samples.len() {
            let x = (i as f64 / ratio) as f64 - index as f64;
            let y: f32 = samples[index] as f32
                + x as f32 * ((samples[index + 1] as f32) - (samples[index] as f32));
            resampled_samples.push(y);
        }
    }
    resampled_samples
}

fn low_pass_filter(samples: &Vec<f32>, cutoff_freq: f32, sample_rate: f32) -> Vec<f32> {
    let mut filtered_samples = Vec::with_capacity(samples.len());
    let rc = 1.0 / (cutoff_freq * 2.0 * std::f32::consts::PI);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);
    let mut prev = samples[0];

    for &sample in samples.iter() {
        let filtered = prev + alpha * (sample - prev);
        filtered_samples.push(filtered);
        prev = filtered;
    }

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
    let mut best_offset = 0;
    let mut best_score = f32::MIN;

    for offset in 0..=(signal_len - sync_len) {
        let mut score = 0.0;
        let mut signal_energy = 0.0;
        let mut pattern_energy = 0.0;

        for i in 0..sync_len {
            score += signal[offset + i] * sync_pattern[i];
            signal_energy += signal[offset + i] * signal[offset + i];
            pattern_energy += sync_pattern[i] * sync_pattern[i];
        }

        let normalized_score = score / (signal_energy.sqrt() * pattern_energy.sqrt());
        if normalized_score > best_score {
            best_score = normalized_score;
            best_offset = offset;
        }
    }

    best_offset
}

fn sync_apt(signal: &Vec<f32>, frame_width: usize, sync_pattern: &[f32]) -> Vec<f32> {
    let mut synced = Vec::with_capacity(signal.len());
    let rows = signal.len() / frame_width;
    let additional_offset = 120; // Adjust this value as needed

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

        for offset in (best_offset.saturating_sub(fine_tune_range))..=(best_offset + fine_tune_range).min(row_slice.len() - sync_pattern.len()) {
            let mut score = 0.0;
            let mut signal_energy = 0.0;
            let mut pattern_energy = 0.0;

            for i in 0..sync_pattern.len() {
                score += row_slice[offset + i] * sync_pattern[i];
                signal_energy += row_slice[offset + i] * row_slice[offset + i];
                pattern_energy += sync_pattern[i] * sync_pattern[i];
            }

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
        fine_tuned_offset = fine_tuned_offset.saturating_sub(additional_offset).min(row_slice.len());

        // Circular shift from fine-tuned offset
        println!("Best offset: {}, Fine-tuned offset: {}", best_offset, fine_tuned_offset);
        for i in fine_tuned_offset..row_slice.len() {
            synced.push(row_slice[i]);
        }
        for i in 0..fine_tuned_offset {
            synced.push(row_slice[i]);
        }
    }

    synced
}

fn envelope_detection(signal: &Vec<f32>) -> Vec<f32> {
    let mut envelope: Vec<f32> = Vec::new();
    let window_size = 10; // adjust this value to change the smoothing amount
    let scaling_factor: f32 = 2.0; // adjust this value to change the brightness
    for i in 0..signal.len() {
        let mut max: f32 = 0.0; // specify the type of max explicitly
        for j in 0..window_size {
            if i + j < signal.len() {
                max = max.max(signal[i + j].abs());
            }
        }
        envelope.push(max * scaling_factor);
    }
    envelope
}

fn generate_image(signal: &Vec<f32>, frequency: f32) -> String {
    let frame_width = (frequency * 0.5) as u32;
    println!("Frame width: {}", frame_width);
    let w = frame_width;
    let h = (signal.len() / frame_width as usize) as u32;
    println!("Width: {}, Height: {}", w, h);

    let mut img: GrayImage = ImageBuffer::new(w, h);

    let mut px = 0;
    let mut py = 0;

    for p in 0..signal.len() {
        let mut lum = signal[p] / 32. - 32.;
        if lum < 0. {
            lum = 0.;
        }
        if lum > 255. {
            lum = 255.;
        }
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

    // Reduce image width by 5
    let new_w = w / 5;
    let mut img_resized: GrayImage = ImageBuffer::new(new_w, h);

    for px in 0..new_w {
        for py in 0..h {
            let orig_px = px * 5;
            img_resized.put_pixel(px, py, *img.get_pixel(orig_px, py));
        }
    }
    img = img_resized;

    normalize_image(&mut img);

    img.save("image.png").unwrap();

    String::from("image.png")
}
