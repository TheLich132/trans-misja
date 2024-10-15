use crate::ui::GLOBALS;
use hound::WavReader;
use image::{GrayImage, ImageBuffer, Luma};
use plotly::{Plot, Scatter};

pub fn compute_signal(filepath: &str, globals: &GLOBALS) -> String {
    /*
        Loading wav files with hound
    */
    let mut reader = WavReader::open(filepath).unwrap();
    let spec = reader.spec();
    if globals.debug {
        println!("Wav file: {}", filepath);
        println!("Sample rate: {}", spec.sample_rate);
        println!("Channels: {}", spec.channels);
        println!("Sample format: {:?}", spec.sample_format);
    }

    let target_sample_rate = 20800;

    // let samples: Vec<f32> = reader.samples::<f32>().collect::<Result<Vec<f32>, hound::Error>>()?.iter().enumerate().filter_map(|(i, sample)| {
    //     if i % spec.channels as usize == 0 {
    //         Some(*sample)
    //     } else {
    //         None
    //     }
    // }).collect();

    // let samples: Vec<f32> = match spec.sample_format {
    //     hound::SampleFormat::Float => reader.samples::<f32>().collect::<Result<Vec<f32>, hound::Error>>().unwrap(),
    //     hound::SampleFormat::Int => reader.samples::<i16>().collect::<Result<Vec<i16>, hound::Error>>().unwrap(),
    // }.iter().enumerate().filter_map(|(i, sample)| {
    //     if i % spec.channels as usize == 0 {
    //         Some(*sample as f32)
    //     } else {
    //         None
    //     }
    // }).collect();

    let mut samples: Vec<f32> = Vec::new();
    if spec.sample_format == hound::SampleFormat::Float {
        let samples_float = reader.samples::<f32>().collect::<Result<Vec<f32>, hound::Error>>().unwrap();
        let channels = spec.channels as usize;
        let mut i = 0;
        while i < samples_float.len() {
            if i % channels == 0 {
                samples.push(samples_float[i]);
            }
            i += 1;
        }
    } else if spec.sample_format == hound::SampleFormat::Int {
        let samples_int = reader.samples::<i16>().collect::<Result<Vec<i16>, hound::Error>>().unwrap();
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

    let ratio = target_sample_rate as f64 / spec.sample_rate as f64;

    let mut resampled_samples: Vec<f32> = Vec::new();
    for i in 0..(samples.len() as f64 * ratio) as usize {
        let index = (i as f64 / ratio) as usize;
        if index < samples.len() {
            let x = (i as f64 / ratio) as f64 - index as f64;
            let y: f32 = samples[index] as f32 + x as f32 * ((samples[index + 1] as f32) - (samples[index] as f32));
            resampled_samples.push(y);
        }
    }

    println!("Resampled samples: {}", resampled_samples.len());
    for sample in resampled_samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");

    let frequency = target_sample_rate as f32;
    
    println!("Demodulating...");
    let am_signal = envelope_detection(&resampled_samples);

    // **************
    // TODO: Delete this later
    // **************
    // let am_signal = am_demodulation2(&resampled_samples, frequency);
    // println!("Hilbert...");
    // let am_signal = hilbert(&resampled_samples.into_iter().map(|x| x as f64).collect::<Vec<f64>>());
    // println!("Normalized...");
    // let am_envelope: Vec<f32> = am_signal.iter().map(|x| x.norm() as f32).collect();
    // let synced_signal = sync(&am_signal);

    let path = generate_image(&am_signal, frequency);

    return path;
}

fn sync(signal: &Vec<f32>) -> Vec<f32> {
    // . = -1, W = 1
    let sample_sync_frame: [f32; 36] = [-1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0];

    let mut plot = Plot::new();

    let mut sync_signal = Vec::new();
    for i in 0..(signal.len() - sample_sync_frame.len() + 1) {
        let mut sum = 0.0;
        for j in 0..sample_sync_frame.len() {
            sum += signal[i + j] * sample_sync_frame[j];
        }
        sync_signal.push(sum);
    }

    let trace = Scatter::new(
        (0..10000).collect::<Vec<usize>>(),
        sync_signal.iter().take(10000).map(|x| *x as f64).collect::<Vec<f64>>(),
    );
    plot.add_trace(trace);
    plot.write_html("plot.html");

    sync_signal
}


// **************
// TODO: Delete this later
// **************
// fn am_demodulation(signal: &Vec<f32>, frequency: f32) -> Vec<f32> {
//     let mut am_signal = Vec::new();
//     for i in 1..signal.len() {
//         am_signal.push(
//             (f32::sqrt(
//                 f32::powi(signal[i], 2) + f32::powi(signal[i - 1], 2)
//                     - 2.0 * signal[i] * signal[i - 1] * f32::cos(frequency),
//             )) / f32::sin(frequency),
//         )
//     }
//     am_signal
// }

// **************
// TODO: Delete this later
// **************
// fn am_demodulation2(signal: &Vec<f32>, frequency: f32) -> Vec<f32> {
//     let lo_freq = frequency * 0.9;
//     let lo_signal: Vec<f32> = (0..signal.len()).map(|x| (2.0 * std::f32::consts::PI * lo_freq * x as f32).sin()).collect();

//     let product: Vec<f32> = signal.iter().zip(lo_signal.iter()).map(|(x, y)| x * y).collect();

//     let mut filtered_product = Vec::with_capacity(product.len());
//     let filter_coeff = [1.0, -0.5, 0.25];
//     for i in 0..product.len() {
//         let mut sum = 0.0;
//         for j in 0..filter_coeff.len() {
//             sum += product[(i + j) % product.len()] * filter_coeff[j];
//         }
//         filtered_product.push(sum);
//     }

//     filtered_product
// }

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

    img.save("image.png").unwrap();

    String::from("image.png")
}