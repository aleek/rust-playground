use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::env;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
use speech_to_text::SpeechToText;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <model_path>", args[0]);
        std::process::exit(1);
    }

    let model_path = &args[1];
    let sample_rate = 48000.0;

    // Create recognizer
    let stt = Arc::new(Mutex::new(SpeechToText::new(model_path, sample_rate).expect("Failed to create recognizer")));

    // Set up audio input
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No input device available");
    let config = device.default_input_config().expect("Failed to get default input config");

    // We'll downsample if needed, but expect 16kHz mono or stereo, i16
    let stt_clone = stt.clone();
    let err_fn = |err| eprintln!("Stream error: {}", err);

    println!("Sample rate: {}", config.sample_rate().0);
    println!("Sample format: {:?}", config.sample_format());
    println!("Channels: {}", config.channels());
    println!("Buffer size: {:?}", config.buffer_size());

    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => build_input_stream::<i16>(&device, &config.into(), stt_clone, err_fn),
        cpal::SampleFormat::U16 => build_input_stream::<u16>(&device, &config.into(), stt_clone, err_fn),
        cpal::SampleFormat::F32 => build_input_stream_f32_mono(&device, &config.into(), stt_clone, err_fn),
        _ => panic!("Unsupported sample format"),
    };

    stream.play().expect("Failed to start stream");

    println!("Listening... Press Ctrl+C to stop.");
    loop {
        thread::sleep(Duration::from_millis(500));
        //let mut recognizer = stt.lock().unwrap();

        // if let Some(partial) = recognizer.get_partial() {
        //     if !partial.is_empty() {
        //         println!("Partial: {}", partial);
        //     }
        // }

        //recognizer.get_result_wait();

        /*
        if let Some(final_text) = recognizer.get_final_result() {
            if !final_text.is_empty() {
                println!("Final: {}", final_text);
            }
        }
        */
    }
}

fn build_input_stream_f32_mono(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    stt: Arc<Mutex<SpeechToText>>,
    err_fn: fn(cpal::StreamError),
) -> cpal::Stream {
    device.build_input_stream(
        config,
        move |data: &[f32], _| {
            let mut mono_samples = Vec::with_capacity(data.len());
            for sample in data {
                let s: i16 = f32::to_sample(*sample);
                mono_samples.push(s);
            }
            let mut recognizer = stt.lock().unwrap();
            recognizer.push_audio_mono(&mono_samples);
        },
        err_fn,
        None,
    ).expect("Failed to build input stream")
}

    //T: cpal::Sample + Send + 'static,
fn build_input_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    stt: Arc<Mutex<SpeechToText>>,
    err_fn: fn(cpal::StreamError),
) -> cpal::Stream
where
    T: cpal::SizedSample + cpal::FromSample<i16>, i16: cpal::FromSample<T>,
{
    let channels = config.channels as usize;
    device.build_input_stream(
        config,
        move |data: &[T], _| {
            // Convert input to i16 mono
            let mut mono_samples = Vec::with_capacity(data.len() / channels);
            for sample in data {
                let s = T::to_sample(*sample);
                mono_samples.push(s);
            }
                /*
            for frame in data.chunks(channels) {
                let l = frame[0].to_signed_sample();
                let sample = if channels > 1 {
                    let r = frame[1].to_signed_sample();
                    l.add_amp(amp)
                    ((l + r) / 2) as i16
                } else {
                    l
                };
                mono_samples.push(sample);
                mono_samples.push(frame[0].to_signed_sample());
            }
                */

            let mut recognizer = stt.lock().unwrap();
            recognizer.push_audio_mono(&mono_samples);
        },
        err_fn,
        None,
    ).expect("Failed to build input stream")
}