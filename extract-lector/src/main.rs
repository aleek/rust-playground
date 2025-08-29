use std::env;
use std::fs::File;
use std::io::{Read, Write};
use anyhow::Result;

/// Skip first n elements from a vector
fn skip_first_n(v: &[i16], len: usize) -> Vec<i16> {
    if len >= v.len() {
        Vec::new()
    } else {
        v[len..].to_vec()
    }
}

/// Wczytaj plik PCM jako wektor i16
fn read_pcm_i16(path: &str) -> Result<Vec<i16>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let samples: Vec<i16> = buffer
        .chunks_exact(2)
        .map(|b| i16::from_le_bytes([b[0], b[1]]))
        .collect();
    Ok(samples)
}

/// Zapisz wynikowy wektor i16 do pliku PCM
fn write_pcm_i16(path: &str, samples: &[i16]) -> Result<()> {
    let mut file = File::create(path)?;
    for &sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

/// Znajdź najlepsze przesunięcie A względem C (maksymalna korelacja)
/// Computational complexity: O(samplerate*number_of_samples)
fn find_best_lag(a: &[i16], c: &[i16], max_lag: usize) -> isize {
    let mut best_lag = 0;
    let mut best_corr = f64::MIN;

    for lag in -(max_lag as isize)..=(max_lag as isize) {
        let corr: f64 = a.iter().enumerate()
            .filter_map(|(i, &a_val)| {
                let j = i as isize + lag;
                if j >= 0 && (j as usize) < c.len() {
                    Some(a_val as f64 * c[j as usize] as f64)
                } else {
                    None
                }
            })
            .sum();

        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    best_lag
}

/// Przesuń sygnał A względem C o `lag` próbek
fn shift_signal(a: &[i16], lag: isize, target_len: usize) -> Vec<i16> {
    if lag > 0 {
        let lag = lag as usize;
        let mut shifted = vec![0i16; lag];
        shifted.extend_from_slice(&a[..target_len.saturating_sub(lag)]);
        shifted.truncate(target_len);
        shifted
    } else {
        let lag = (-lag) as usize;
        let start = lag.min(a.len());
        let mut shifted = a[start..].to_vec();
        shifted.resize(target_len, 0);
        shifted
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        eprintln!("Usage: {} <orig audio file> <lector audio file mixed> <output file>", args[0]);
        std::process::exit(1);
    }

    // Load data
    println!("Loading original audio file...");
    let orig_audio_samples = read_pcm_i16(&args[1])?;
    println!("Loading mixed audio file...");
    let mixed_audio_samples = read_pcm_i16(&args[2])?;
    println!("Loading done");

    // Find best match in range +/- 1 second (48000 samples)
    let max_lag = 48000;
    //let lag = find_best_lag(&a, &c, max_lag);
    let lag = 10055; // lector is later
    println!("Found lag: {} samples ({} ms)", lag, lag as f64 * 1000.0 / 48000.0);

    // Shift A
    //let a_aligned = shift_signal(&orig_audio_samples, lag, mixed_audio_samples.len());
    let a_aligned = skip_first_n(&orig_audio_samples, lag as usize);
    let c_aligned = &mixed_audio_samples[..a_aligned.len()];

    // Calculate attenuation coefficient alpha
    let dot_ac: f64 = a_aligned.iter().zip(c_aligned).map(|(&a, &c)| a as f64 * c as f64).sum();
    let dot_aa: f64 = a_aligned.iter().map(|&a| (a as f64).powi(2)).sum();
    let alpha = dot_ac / dot_aa;
    println!("Alpha coefficient: {:.4}", alpha);

    // Subtract A from C to get B
    let b: Vec<i16> = c_aligned.iter().zip(a_aligned.iter())
        .map(|(&c_val, &a_val)| {
            //let b_f = c_val as f64 - (1.0 - alpha) * (a_val as f64);
            //let b_f = c_val as f64 - a_val as f64;
            c_val/2 - a_val/2
            //b_f.clamp(i16::MIN as f64, i16::MAX as f64).round() as i16
        })
        .collect();

    write_pcm_i16(&args[3], &b)?;
    println!("Result saved to {}", args[3]);

    let d: Vec<i16> = c_aligned.iter().zip(a_aligned.iter())
        .map(|(&c_val, &a_val)| {
            //let b_f = c_val as f64 - (1.0 - alpha) * (a_val as f64);
            //let b_f = a_val as f64 - c_val as f64;
            //b_f.clamp(i16::MIN as f64, i16::MAX as f64).round() as i16
            a_val/2 + c_val/2
        })
        .collect();

    write_pcm_i16(&args[4], &d)?;
    println!("Result saved to {}", args[4]);
    Ok(())
}
