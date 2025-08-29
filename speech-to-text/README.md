# Speech-to-Text by Cursor

A Rust library for real-time speech recognition using Vosk.

## Examples

### Process Audio File

Process a raw PCM 16-bit audio file with 48kHz sampling rate:

```bash
# Build the example
cargo build --example process_audio_file

# Run with your audio file
cargo run --example process_audio_file path/to/your/audio.pcm
```

**Note:** You need to update the model path in `examples/process_audio_file.rs` to point to your Vosk model directory.

### Microphone to Console

Real-time speech recognition from microphone input:

```bash
cargo run --example mic_to_console
```

## Audio Format Requirements

For the `process_audio_file` example:
- Raw PCM 16-bit audio
- 48kHz sampling rate
- Mono or stereo (stereo will be downmixed to mono)

## Dependencies

- Vosk speech recognition engine
- CPAL for audio capture (in mic example)