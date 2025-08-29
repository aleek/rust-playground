use std::io::Write;

use vosk::{AcceptWaveformError, CompleteResult, DecodingState, Model, Recognizer};

pub struct SpeechToText {
    recognizer: Recognizer,
    last_decoding_state: DecodingState,
    partial_text: String,
    // Optionally store model if needed for lifetime
}

impl SpeechToText {
    /// Create a new SpeechToText recognizer
    pub fn new(model_path: &str, sample_rate: f32) -> Result<Self,std::io::Error> {
        let model = Model::new(model_path).ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Model not found"))?;
        let mut recognizer = Recognizer::new(&model, sample_rate).ok_or(std::io::Error::new(std::io::ErrorKind::Other, "Recognizer not found"))?;
        recognizer.set_max_alternatives(0);
        recognizer.set_words(true);
        recognizer.set_partial_words(true);
        Ok(Self { recognizer, last_decoding_state: DecodingState::Running, partial_text: String::new() })
    }

    /// Push a vector of interleaved stereo samples (i16: L, R, L, R, ...)
    pub fn push_audio(&mut self, stereo_samples: &[i16]) -> Result<DecodingState, AcceptWaveformError> {
        // Downmix stereo to mono by averaging L and R
        let mut mono_samples = Vec::with_capacity(stereo_samples.len() / 2);
        for chunk in stereo_samples.chunks(2) {
            if chunk.len() == 2 {
                let l = chunk[0] as i32;
                let r = chunk[1] as i32;
                let avg = ((l + r) / 2) as i16;
                mono_samples.push(avg);
            }
        }
        let decoding_state = self.recognizer.accept_waveform(&mono_samples)?;
        self.last_decoding_state = decoding_state;
        Ok(decoding_state)
    }

    /// Push a vector of mono samples (i16)
    pub fn push_audio_mono(&mut self, mono_samples: &[i16]) -> Result<DecodingState, AcceptWaveformError> {
        let decoding_state = self.recognizer.accept_waveform(mono_samples)?;
        match decoding_state {
            DecodingState::Running => {
                let partial = self.recognizer.partial_result();
                let str = partial.partial.to_string();
                if str != self.partial_text {
                    self.partial_text = str;
                    print!("\rpartial:{}", partial.partial);
                    std::io::stdout().flush().unwrap();
                }
            }
            DecodingState::Finalized => {
                match self.recognizer.result() {
                    CompleteResult::Single(result) => {
                        println!("\rresult: {:?}", result.text);
                    }
                    CompleteResult::Multiple(result) => {
                        result.alternatives.clone().sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
                        if result.alternatives.len() == 0 {
                            ()
                        }

                        let text = result.alternatives[0].text;
                        println!("text: {:?} confidence: {:?} alternatives: {:?}", text, result.alternatives[0].confidence, result.alternatives.len());
                    }
                }
            }
            DecodingState::Failed => {
                println!("Failed");
            }
        }
        self.last_decoding_state = decoding_state;
        Ok(decoding_state)
    }

    /// Get the latest partial result (words recognized so far)
    pub fn get_partial(&mut self) -> Option<String> {
        let partial = self.recognizer.partial_result();
        println!("partial: {:?}", partial);
        let val = partial.partial; // .get("partial")?.as_str()?;
        if val.is_empty() { None } else { Some(val.to_string()) }
    }

    pub fn get_result_wait(&mut self) {
        let result = self.recognizer.result();

        match result {
            CompleteResult::Single(result) => {
                println!("result: {:?}", result.text);
            }
            CompleteResult::Multiple(result) => {
                result.alternatives.clone().sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
                if result.alternatives.len() == 0 {
                    return;
                }

                let text = result.alternatives[0].text;
                println!("text: {:?} confidence: {:?} alternatives: {:?}", text, result.alternatives[0].confidence, result.alternatives.len());
            }
        }
    }

    /// Get the final result (words recognized in completed utterance)
    pub fn get_final_result(&mut self) -> Option<String> {
        let final_result = self.recognizer.final_result();
        println!("final_result: {:?}", final_result);
        match final_result {
            CompleteResult::Single(result) => {
                let val = result.text;
                if val.is_empty() { None } else { Some(val.to_string()) }
            }
            CompleteResult::Multiple(result) => {
                if result.alternatives.is_empty() { None } else {
                    let val = result.alternatives[0].text;
                    if val.is_empty() { None } else { Some(val.to_string()) }
                }
            }
        }
    }
}
