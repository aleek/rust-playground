use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use speech_to_text::SpeechToText;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <model_path> <audio_file>", args[0]);
        std::process::exit(1);
    }

    let model_path = &args[1];
    let filename = &args[2];

    // Create SpeechToText instance
    let mut stt = SpeechToText::new(model_path, 48000.0)?;

    // Open and read the audio file
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    // Buffer to hold 48000 samples (48000 * 2 bytes per i16 sample)
    let mut buffer = vec![0u8; 48000 * 2];
    let mut samples = vec![0i16; 48000];

    println!("Processing audio file: {}", filename);
    println!("Reading 48000 samples at a time...");

    loop {
        // Read raw bytes from file
        let bytes_read = reader.read(&mut buffer)?;

        // If no bytes read, we've reached end of file
        if bytes_read == 0 {
            break;
        }

        // Convert bytes to i16 samples
        for i in 0..(bytes_read / 2) {
            if i < samples.len() {
                let byte1 = buffer[i * 2] as u16;
                let byte2 = buffer[i * 2 + 1] as u16;
                samples[i] = ((byte2 << 8) | byte1) as i16;
            }
        }

        // Process the samples through SpeechToText
        let actual_samples = &samples[..(bytes_read / 2)];
        match stt.push_audio_mono(actual_samples) {
            Ok(decoding_state) => {
                match decoding_state {
                    vosk::DecodingState::Running => {
                        // Partial results are handled in push_audio_mono
                    }
                    vosk::DecodingState::Finalized => {
                        // Final results are handled in push_audio_mono
                    }
                    vosk::DecodingState::Failed => {
                        eprintln!("Decoding failed");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing audio: {:?}", e);
            }
        }
    }

    println!("\nAudio processing complete!");
    Ok(())
}