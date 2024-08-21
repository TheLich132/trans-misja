use hound::WavReader;

pub fn load_wav_file(filepath: &str) {
    /*
        Loading wav files with hound
    */
    let mut reader = WavReader::open(filepath).unwrap();
    let spec = reader.spec();
    println!("Sample rate: {}", spec.sample_rate);
    println!("Channels: {}", spec.channels);
    println!("Sample format: {:?}", spec.sample_format);

    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();
    for sample in samples.iter().take(100) {
        print!("{}, ", sample);
    }
    println!("(...)");
}
