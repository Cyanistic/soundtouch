//!A wrapper around the SoundTouch C++ audio library.
//!
//!## High Level Example
//!```rust
//!use soundtouch_rs::SoundTouch;
//!let mut soundtouch = SoundTouch::new();
//!soundtouch
//!    .set_channels(2)
//!    .set_sample_rate(44100)
//!    .set_tempo(1.10);
//!
//!// use actual audio samples here
//!let samples = vec![0.0; 44100 * 2];
//!let output_samples = soundtouch.generate_audio(&samples);
//!
//!// do something with output_samples
//!
//!```
//!## Low Level Example
//!```rust
//!use soundtouch_rs::SoundTouch;
//!let mut soundtouch = SoundTouch::new();
//!soundtouch
//!    .set_channels(2)
//!    .set_sample_rate(44100)
//!    .set_tempo(1.10);
//!
//!// use actual audio samples here
//!let mut samples = vec![0.0; 44100 * 2];
//!
//!const BUF_SIZE: usize = 6720;
//!let mut new_samples: [f32; BUF_SIZE] = [0.0; BUF_SIZE];
//!let mut output_samples: Vec<f32> = Vec::with_capacity(samples.len());
//!soundtouch.put_samples(&samples, samples.len() / soundtouch.0.channels as usize);
//!let mut n_samples = 1;
//!while n_samples != 0 {
//!    n_samples = soundtouch.receive_samples(
//!        new_samples.as_mut_slice(),
//!        BUF_SIZE / soundtouch.0.channels as usize
//!        );
//!    output_samples.extend_from_slice(&new_samples);
//!}
//!soundtouch.flush();
//!
//!// do something with output_samples
//!
//!````
//!Both examples should produce the same output.
use ffi::{
    soundtouch_SoundTouch as SoundTouchSys, soundtouch_SoundTouch_putSamples as putSamples,
    soundtouch_SoundTouch_receiveSamples as receiveSamples, uint,
};
use std::ffi::{c_int, c_void};
mod bindings;
use bindings as ffi;
// use soundtouch_sys as ffi;

/// A list of settings that can be enabled or disabled.
pub enum Setting {
    /// Enable/disable anti-alias filter in pitch transposer (0 = disable)
    UseAaFilter,

    /// Pitch transposer anti-alias filter length (8 .. 128 taps, default = 32)
    AaFilterLength,

    /// Enable/disable quick seeking algorithm in tempo changer routine
    /// (enabling quick seeking lowers CPU utilization but causes a minor sound
    ///  quality compromising)
    UseQuickseek,

    /// Time-stretch algorithm single processing sequence length in milliseconds. This determines
    /// to how long sequences the original sound is chopped in the time-stretch algorithm.
    /// See "STTypes.h" or README for more information.
    SequenceMs,

    /// Time-stretch algorithm seeking window length in milliseconds for algorithm that finds the
    /// best possible overlapping location. This determines from how wide window the algorithm
    /// may look for an optimal joining location when mixing the sound sequences back together.
    /// See "STTypes.h" or README for more information.
    SeekwindowMs,

    /// Time-stretch algorithm overlap length in milliseconds. When the chopped sound sequences
    /// are mixed back together, to form a continuous sound stream, this parameter defines over
    /// how long period the two consecutive sequences are let to overlap each other.
    /// See "STTypes.h" or README for more information.
    OverlapMs,

    /// Call "getSetting" with this ID to query processing sequence size in samples.
    /// This value gives approximate value of how many input samples you'll need to
    /// feed into SoundTouch after initial buffering to get out a new batch of
    /// output samples.
    ///
    /// This value does not include initial buffering at beginning of a new processing
    /// stream, use INITIAL_LATENCY to get the initial buffering size.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    NominalInputSequence,

    /// Call "getSetting" with this ID to query nominal average processing output
    /// size in samples. This value tells approcimate value how many output samples
    /// SoundTouch outputs once it does DSP processing run for a batch of input samples.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    NominalOutputSequence,

    /// Call "getSetting" with this ID to query initial processing latency, i.e.
    /// approx. how many samples you'll need to enter to SoundTouch pipeline before
    /// you can expect to get first batch of ready output samples out.
    ///
    /// After the first output batch, you can then expect to get approx.
    /// NOMINAL_OUTPUT_SEQUENCE ready samples out for every
    /// NOMINAL_INPUT_SEQUENCE samples that you enter into SoundTouch.
    ///
    /// Example:
    ///     processing with parameter -tempo=5
    ///     => initial latency = 5509 samples
    ///        input sequence  = 4167 samples
    ///        output sequence = 3969 samples
    ///
    /// Accordingly, you can expect to feed in approx. 5509 samples at beginning of
    /// the stream, and then you'll get out the first 3969 samples. After that, for
    /// every approx. 4167 samples that you'll put in, you'll receive again approx.
    /// 3969 samples out.
    ///
    /// This also means that average latency during stream processing is
    /// INITIAL_LATENCY-OUTPUT_SEQUENCE/2, in the above example case 5509-3969/2
    /// = 3524 samples
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    InitialLatency,
}

impl Setting {
    fn as_c_int(&self) -> c_int {
        match self {
            Setting::UseAaFilter => 0,
            Setting::AaFilterLength => 1,
            Setting::UseQuickseek => 2,
            Setting::SequenceMs => 3,
            Setting::SeekwindowMs => 4,
            Setting::OverlapMs => 5,
            Setting::NominalInputSequence => 6,
            Setting::NominalOutputSequence => 7,
            Setting::InitialLatency => 8,
        }
    }
}

/// A wrapper around the SoundTouch struct.
#[derive(Debug)]
pub struct SoundTouch(SoundTouchSys);

unsafe impl Send for SoundTouch {}

impl Default for SoundTouch {
    fn default() -> Self {
        Self(unsafe { SoundTouchSys::new() })
    }
}

impl SoundTouch {
    /// Crate a new SoundTouch instance.
    pub fn new() -> Self {
        Self(unsafe { SoundTouchSys::new() })
    }

    /// Set the number of channels.
    /// - 1 = mono
    /// - 2 = stereo
    pub fn set_channels(&mut self, num_channels: u32) -> &mut Self {
        unsafe {
            self.0.setChannels(num_channels);
        }
        self
    }

    /// Set the sample rate of the input audio samples.
    pub fn set_sample_rate(&mut self, sample_rate: u32) -> &mut Self {
        unsafe {
            self.0.setSampleRate(sample_rate);
        }
        self
    }

    /// Set the tempo of the audio to generate.
    pub fn set_tempo(&mut self, tempo: f64) -> &mut Self {
        unsafe {
            self.0.setTempo(tempo);
        }
        self
    }

    /// Set the pitch of the audio to generate.
    pub fn set_pitch(&mut self, pitch: f64) -> &mut Self {
        unsafe {
            self.0.setPitch(pitch);
        }
        self
    }

    /// Set the rate of the audio to generate.
    pub fn set_rate(&mut self, rate: f64) -> &mut Self {
        unsafe {
            self.0.setRate(rate);
        }
        self
    }

    /// Set the change of tempo from the input audio.
    pub fn set_tempo_change(&mut self, tempo_change: f64) -> &mut Self {
        unsafe {
            self.0.setTempoChange(tempo_change);
        }
        self
    }

    /// Set the pitch octaves of the audio to generate.
    pub fn set_pitch_octaves(&mut self, pitch_octaves: f64) -> &mut Self {
        unsafe {
            self.0.setPitchOctaves(pitch_octaves);
        }
        self
    }

    /// Set the pitch semitones of the audio to generate.
    pub fn set_pitch_semitones(&mut self, pitch_semitones: i32) -> &mut Self {
        unsafe {
            self.0.setPitchSemiTones(pitch_semitones);
        }
        self
    }

    /// Enable a setting from the Setting enum.
    pub fn set_setting(&mut self, setting: Setting, value: i64) -> &mut Self {
        unsafe {
            self.0.setSetting(setting.as_c_int(), value as c_int);
        }
        self
    }

    /// **NOT FROM SOUNDTOUCH**
    ///
    /// Generates audio samples from given input samples using the settings set in the SoundTouch struct
    /// and returns them in a vector.
    /// Do not use [`put_samples`] or [`receive_samples`] with this function.
    ///
    /// [`put_samples`]: SoundTouch::put_samples
    /// [`receive_samples`]: SoundTouch::receive_samples
    pub fn generate_audio(&mut self, samples: &[f32]) -> Vec<f32> {
        const BUF_SIZE: usize = 6720;
        let mut new_samples: [f32; BUF_SIZE] = [0.0; BUF_SIZE];
        let mut out_data: Vec<f32> = Vec::with_capacity(samples.len());
        unsafe {
            let ptr: *mut c_void = &mut self.0 as *mut _ as *mut c_void;
            putSamples(
                ptr,
                samples.as_ptr(),
                samples.len() as u32 / self.0.channels,
            );
            let mut n_samples: u32 = 1;
            while n_samples != 0 {
                n_samples = receiveSamples(
                    ptr,
                    new_samples.as_mut_ptr(),
                    BUF_SIZE as u32 / self.0.channels,
                );
                out_data.extend_from_slice(&new_samples);
            }
            self.0.flush();
        }
        out_data
    }

    /// Adds 'numSamples' pcs of samples from the 'samples' memory position into
    /// the input of the object. Notice that sample rate _has_to_ be set before
    /// calling this function, otherwise throws a runtime_error exception.
    pub fn put_samples(&mut self, samples: &[f32], num_samples: usize) {
        unsafe {
            ffi::soundtouch_SoundTouch_putSamples(
                &mut self.0 as *mut _ as *mut c_void,
                samples.as_ptr(),
                num_samples as uint,
            );
        }
    }

    /// Output samples from beginning of the sample buffer. Copies requested samples to
    /// output buffer and removes them from the sample buffer. If there are less than
    /// 'numsample' samples in the buffer, returns all that available.
    pub fn receive_samples(&mut self, samples: &mut [f32], max_samples: usize) -> usize {
        unsafe {
            ffi::soundtouch_SoundTouch_receiveSamples(
                &mut self.0 as *mut _ as *mut c_void,
                samples.as_mut_ptr(),
                max_samples as uint,
            ) as usize
        }
    }

    /// Returns number of samples currently unprocessed.
    pub fn num_unprocessed_samples(&self) -> usize {
        unsafe {
            ffi::soundtouch_SoundTouch_numUnprocessedSamples(&self.0 as *const _ as *mut c_void)
                as usize
        }
    }

    /// Clears all the samples in the object's output and internal processing.
    pub fn clear(&mut self) {
        unsafe {
            ffi::soundtouch_SoundTouch_clear(&mut self.0 as *mut _ as *mut c_void);
        }
    }

    /// This function is meant for extracting the last samples of a sound
    /// stream. This function may introduce additional blank samples in the end
    /// of the sound stream, and thus it's not recommended to call this function
    /// in the middle of a sound stream.
    pub fn flush(&mut self) {
        unsafe {
            ffi::soundtouch_SoundTouch_flush(&mut self.0);
        }
    }

}
