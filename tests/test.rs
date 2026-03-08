use anyhow::{anyhow, Result};
use std::path::PathBuf;

use soundtouch::SoundTouch;

#[test]
fn test() {
    change_tempo_wav(&PathBuf::from("./tests/beer.wav"), 1.5, false).unwrap();
}

#[test]
fn test_num_samples_decreases_after_receive() {
    let mut soundtouch = SoundTouch::new();
    soundtouch
        .set_channels(2)
        .set_sample_rate(44100)
        .set_tempo(1.0);

    // Feed in enough samples to produce output
    let input_samples = vec![0.5f32; 44100 * 2]; // 1 second stereo
    soundtouch.put_samples(&input_samples, input_samples.len() / 2);
    soundtouch.flush();

    // Should have samples available now
    let initial_count = soundtouch.num_samples();
    assert!(
        initial_count > 0,
        "Expected num_samples > 0 after put_samples + flush, got {}",
        initial_count
    );

    // Receive some samples
    let mut output_buf = vec![0.0f32; 1000];
    let received = soundtouch.receive_samples(&mut output_buf, 500);
    assert!(received > 0, "Expected to receive some samples");

    // num_samples should have decreased
    let after_count = soundtouch.num_samples();
    assert!(
        after_count < initial_count,
        "Expected num_samples to decrease after receive_samples: before={}, after={}",
        initial_count,
        after_count
    );
}

fn change_tempo_wav(path: &PathBuf, rate: f64, change_pitch: bool) -> Result<()> {
    let mut reader = hound::WavReader::open(path)?;
    let mut spec = hound::WavSpec { ..reader.spec() };
    let mut encoder = hound::WavWriter::create(
        format!(
            "{}({}).wav",
            path.parent()
                .ok_or(anyhow!("No parent path"))?
                .join(path.file_stem().ok_or(anyhow!("Invalid file"))?)
                .display(),
            rate
        ),
        spec,
    )?;

    let samples = reader
        .samples::<i16>()
        .map(|x| x.unwrap() as f32)
        .collect::<Vec<f32>>();
    let out_data: Vec<f32>;

    if change_pitch {
        spec.sample_rate = (spec.sample_rate as f64 * rate) as u32;
        out_data = samples;
    } else {
        let mut soundtouch = SoundTouch::new();
        soundtouch
            .set_tempo(rate)
            .set_sample_rate(spec.sample_rate)
            .set_channels(spec.channels as u32);
        out_data = soundtouch.generate_audio(&samples);
    }

    for sample in out_data {
        encoder.write_sample(sample as i16)?;
    }
    encoder.finalize()?;
    Ok(())
}
