use std::path::PathBuf;
use anyhow::{Result, anyhow};

use soundtouch::SoundTouch;

#[test]
fn test() {
    change_tempo_wav(&PathBuf::from("./tests/beer.wav"), 1.5, false).unwrap();
}

fn change_tempo_wav(path: &PathBuf, rate: f64, change_pitch: bool) -> Result<()>{
    let mut reader = hound::WavReader::open(path)?;
    let mut spec = hound::WavSpec{
    .. reader.spec()
    };
    let mut encoder = hound::WavWriter::create(format!("{}({}).wav", path.parent().ok_or(anyhow!("No parent path"))?.join(path.file_stem().ok_or(anyhow!("Invalid file"))?).display(), rate), spec)?;
    
    let samples = reader.samples::<i16>().map(|x| x.unwrap() as f32).collect::<Vec<f32>>();
    let out_data: Vec<f32>;

    if change_pitch{
        spec.sample_rate = (spec.sample_rate as f64 * rate) as u32;
        out_data = samples;
    }else{
        let mut soundtouch = SoundTouch::new();
        soundtouch.set_tempo(rate)
            .set_sample_rate(spec.sample_rate)
            .set_channels(spec.channels as u32);
        out_data =  soundtouch.generate_audio(&samples);
    }

    for sample in out_data{
        encoder.write_sample(sample as i16)?;
    }
    encoder.finalize()?;
    Ok(())
}
