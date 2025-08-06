use ffi::{uint, SoundTouch as SoundTouchSys};
use soundtouch_ffi as ffi;
use core::ffi::{c_int, c_void};

#[cfg(feature = "alloc")]
use ffi::{SoundTouch_putSamples as putSamples, SoundTouch_receiveSamples as receiveSamples};
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A list of settings that can be enabled or disabled.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Setting {
    /// Enable/disable anti-alias filter in pitch transposer (0 = disable)
    UseAaFilter = 0,

    /// Pitch transposer anti-alias filter length (8 .. 128 taps, default = 32)
    AaFilterLength = 1,

    /// Enable/disable quick seeking algorithm in tempo changer routine
    /// (enabling quick seeking lowers CPU utilization but causes a minor sound
    ///  quality compromising)
    UseQuickseek = 2,

    /// Time-stretch algorithm single processing sequence length in milliseconds. This determines
    /// to how long sequences the original sound is chopped in the time-stretch algorithm.
    /// See [STTypes.h](https://codeberg.org/soundtouch/soundtouch/src/branch/master/include/STTypes.h) or [README](https://www.surina.net/soundtouch/readme.html) for more information.
    SequenceMs = 3,

    /// Time-stretch algorithm seeking window length in milliseconds for algorithm that finds the
    /// best possible overlapping location. This determines from how wide window the algorithm
    /// may look for an optimal joining location when mixing the sound sequences back together.
    /// See [STTypes.h](https://codeberg.org/soundtouch/soundtouch/src/branch/master/include/STTypes.h) or [README](https://www.surina.net/soundtouch/readme.html) for more information.
    SeekwindowMs = 4,

    /// Time-stretch algorithm overlap length in milliseconds. When the chopped sound sequences
    /// are mixed back together, to form a continuous sound stream, this parameter defines over
    /// how long period the two consecutive sequences are let to overlap each other.
    /// See [STTypes.h](https://codeberg.org/soundtouch/soundtouch/src/branch/master/include/STTypes.h) or [README](https://www.surina.net/soundtouch/readme.html) for more information.
    OverlapMs = 5,

    /// Call [`get_setting`] with this ID to query processing sequence size in samples.
    /// This value gives approximate value of how many input samples you'll need to
    /// feed into SoundTouch after initial buffering to get out a new batch of
    /// output samples.
    ///
    /// This value does not include initial buffering at beginning of a new processing
    /// stream, use `INITIAL_LATENCY` to get the initial buffering size.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. setSetting ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    ///
    ///   [`get_setting`]: SoundTouch::get_setting
    NominalInputSequence = 6,

    /// Call [`get_setting`] with this ID to query nominal average processing output
    /// size in samples. This value tells approcimate value how many output samples
    /// SoundTouch outputs once it does DSP processing run for a batch of input samples.
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. [`set_setting`] ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    ///
    /// [`get_setting`]: SoundTouch::get_setting
    /// [`set_setting`]: SoundTouch::set_setting
    NominalOutputSequence = 7,

    /// Call [`get_setting`] with this ID to query initial processing latency, i.e.
    /// approx. how many samples you'll need to enter to SoundTouch pipeline before
    /// you can expect to get first batch of ready output samples out.
    ///
    /// After the first output batch, you can then expect to get approx.
    /// `NOMINAL_OUTPUT_SEQUENCE` ready samples out for every
    /// `NOMINAL_INPUT_SEQUENCE` samples that you enter into SoundTouch.
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
    /// `INITIAL_LATENCY-OUTPUT_SEQUENCE/2`, in the above example case 5509-3969/2
    /// = 3524 samples
    ///
    /// Notices:
    /// - This is read-only parameter, i.e. [`set_setting`] ignores this parameter
    /// - This parameter value is not pub constant but change depending on
    ///   tempo/pitch/rate/samplerate settings.
    ///
    ///   [`get_setting`]: SoundTouch::get_setting
    ///   [`set_setting`]: SoundTouch::set_setting
    InitialLatency = 8,
}

/// Main class for tempo/pitch/rate adjusting routines.
///
/// Notes:
/// - Initialize the SoundTouch object instance by setting up the sound stream
///   parameters with functions [`set_sample_rate`] and [`set_channels`], then set
///   desired tempo/pitch/rate settings with the corresponding functions.
///
/// - The SoundTouch class behaves like a first-in-first-out pipeline: The
///   samples that are to be processed are fed into one of the pipe by calling
///   function [`put_samples`], while the ready processed samples can be read
///   from the other end of the pipeline with function [`receive_samples`].
///   This crate provides a utility function [`generate_audio`] that does this.
///
/// - The SoundTouch processing classes require certain sized `batches` of
///   samples in order to process the sound. For this reason the classes buffer
///   incoming samples until there are enough of samples available for
///   processing, then they carry out the processing step and consequently
///   make the processed samples available for outputting.
///
/// - For the above reason, the processing routines introduce a certain
///   `latency` between the input and output, so that the samples input to
///   SoundTouch may not be immediately available in the output, and neither
///   the amount of outputtable samples may not immediately be in direct
///   relationship with the amount of previously input samples.
///
/// - The tempo/pitch/rate control parameters can be altered during processing.
///   Please notice though that they aren't currently protected by semaphores,
///   so in multi-thread application external semaphore protection may be
///   required.
///
/// - This class utilizes classes `TDStretch` for tempo change (without modifying
///   pitch) and `RateTransposer` for changing the playback rate (that is, both
///   tempo and pitch in the same ratio) of the sound. The third available control
///   `pitch` (change pitch but maintain tempo) is produced by a combination of
///   combining the two other controls.
///
/// [`set_sample_rate`]: SoundTouch::set_sample_rate
/// [`set_channels`]: SoundTouch::set_channels
/// [`put_samples`]: SoundTouch::put_samples
/// [`receive_samples`]: SoundTouch::receive_samples
/// [`generate_audio`]: SoundTouch::generate_audio
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

    /// Set the sample rate.
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

    /// Sets new pitch control value. Original pitch = 1.0, smaller values
    /// represent lower pitches, larger values higher pitch.
    pub fn set_pitch(&mut self, pitch: f64) -> &mut Self {
        unsafe {
            self.0.setPitch(pitch);
        }
        self
    }

    /// Sets new rate control value. Normal rate = 1.0, smaller values
    /// represent slower rate, larger faster rates.
    pub fn set_rate(&mut self, rate: f64) -> &mut Self {
        unsafe {
            self.0.setRate(rate);
        }
        self
    }

    /// Sets new tempo control value as a difference in percents compared
    /// to the original tempo (-50 .. +100 %).
    pub fn set_tempo_change(&mut self, new_tempo: f64) -> &mut Self {
        unsafe {
            self.0.setTempoChange(new_tempo);
        }
        self
    }

    /// Sets new rate control value as a difference in percents compared
    /// to the original rate (-50 .. +100 %).
    pub fn set_rate_change(&mut self, new_rate: f64) -> &mut Self {
        unsafe {
            self.0.setRateChange(new_rate);
        }
        self
    }

    /// Sets pitch change in octaves compared to the original pitch
    /// `(-1.00 .. +1.00)`.
    pub fn set_pitch_octaves(&mut self, pitch_octaves: f64) -> &mut Self {
        unsafe {
            self.0.setPitchOctaves(pitch_octaves);
        }
        self
    }

    /// Sets pitch change in semi-tones compared to the original pitch
    /// (-12 .. +12).
    pub fn set_pitch_semitones(&mut self, pitch_semitones: i32) -> &mut Self {
        unsafe {
            self.0.setPitchSemiTones(pitch_semitones);
        }
        self
    }

    /// Changes a setting controlling the processing system behaviour. See the
    /// [`Setting`] enum for available settings.
    ///
    ///[`Setting`]: Setting
    pub fn set_setting(&mut self, setting: Setting, value: i32) -> &mut Self {
        unsafe {
            self.0.setSetting(setting as c_int, value as c_int);
        }
        self
    }

    /// **NOT FROM SOUNDTOUCH**
    ///
    /// Generates audio samples from given input samples using the settings set in the SoundTouch struct
    /// and returns them in a vector.
    ///
    /// This is equivalent to calling [`put_samples`] and [`receive_samples`]
    /// until the unprocessed sample buffer is empty.
    ///
    /// Do not use [`put_samples`] or [`receive_samples`] with this function.
    ///
    /// [`put_samples`]: SoundTouch::put_samples
    /// [`receive_samples`]: SoundTouch::receive_samples
    #[cfg(feature = "alloc")]
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


    /// Adds `num_samples` pcs of samples from the `samples` memory position into
    /// the input of the object. Notice that sample rate **must** be set before
    /// calling this function, otherwise throws a runtime_error exception.
    ///
    /// Note: `num_samples` should contain the number of samples per channel.
    /// Ex: If `samples.len()` is `6720` and there are `2` channels, then
    /// `num_samples` should be `3360`.
    pub fn put_samples(&mut self, samples: &[f32], num_samples: usize) {
        unsafe {
            ffi::SoundTouch_putSamples(
                &mut self.0 as *mut _ as *mut c_void,
                samples.as_ptr(),
                num_samples as uint,
            );
        }
    }

    /// Output samples from beginning of the sample buffer. Copies requested samples to
    /// output buffer and removes them from the sample buffer. If there are less than
    /// `max_samples` samples in the buffer, returns all that available.
    pub fn receive_samples(&mut self, samples: &mut [f32], max_samples: usize) -> usize {
        unsafe {
            ffi::SoundTouch_receiveSamples(
                &mut self.0 as *mut _ as *mut c_void,
                samples.as_mut_ptr(),
                max_samples as uint,
            ) as usize
        }
    }

    /// Adjusts book-keeping so that given number of samples are removed from beginning of the
    /// sample buffer without copying them anywhere.
    pub fn receive_samples_no_in(&mut self, max_samples: usize) -> usize {
        unsafe {
            ffi::SoundTouch_receiveSamples1(
                &mut self.0 as *mut _ as *mut c_void,
                max_samples as uint,
            ) as usize
        }
    }

    /// Returns number of samples currently unprocessed.
    pub fn num_unprocessed_samples(&self) -> usize {
        unsafe {
            ffi::SoundTouch_numUnprocessedSamples(&self.0 as *const _ as *mut c_void) as usize
        }
    }

    /// Clears all the samples in the object's output and internal processing
    /// buffers.
    pub fn clear(&mut self) {
        unsafe {
            ffi::SoundTouch_clear(&mut self.0 as *mut _ as *mut c_void);
        }
    }

    /// Flushes the last samples from the processing pipeline to the output.
    /// Clears also the internal processing buffers.
    //
    /// Note: This function is meant for extracting the last samples of a sound
    /// stream. This function may introduce additional blank samples in the end
    /// of the sound stream, and thus it's not recommended to call this function
    /// in the middle of a sound stream.
    pub fn flush(&mut self) {
        unsafe {
            ffi::SoundTouch_flush(&mut self.0);
        }
    }

    /// Returns number of channels.
    pub fn num_channels(&self) -> u32 {
        self.0.channels
    }

    /// Gets a setting controlling the processing system behaviour. See the
    /// [`Setting`] enum for available settings.
    ///
    ///[`Setting`]: Setting
    pub fn get_setting(&self, setting: Setting) -> i32 {
        unsafe { self.0.getSetting(setting as c_int) }
    }

    /// Get ratio between input and output audio durations, useful for calculating
    /// processed output duration: if you'll process a stream of `N` samples, then
    /// you can expect to get out `N * `[`get_input_output_sample_ratio`] samples.
    ///
    /// This ratio will give accurate target duration ratio for a full audio track,
    /// given that the the whole track is processed with same processing parameters.
    ///
    /// If this ratio is applied to calculate intermediate offsets inside a processing
    /// stream, then this ratio is approximate and can deviate +- some tens of milliseconds
    /// from ideal offset, yet by end of the audio stream the duration ratio will become
    /// exact.
    ///
    /// Example: if processing with parameters `-tempo=15 -pitch=-3`, the function
    /// will return value `0.8695652..`. Now, if processing an audio stream whose duration
    /// is exactly one million audio samples, then you can expect the processed
    /// output duration  be `0.869565 * 1000000 = 869565` samples.
    ///
    /// [`get_input_output_sample_ratio`]: SoundTouch::get_input_output_sample_ratio
    pub fn get_input_output_sample_ratio(&mut self) -> f64 {
        unsafe { self.0.getInputOutputSampleRatio() }
    }

    /// Returns the SoundTouch library version Id.
    pub fn get_version_id() -> u32 {
        unsafe { ffi::SoundTouch_getVersionId() }
    }

    /// Returns SoundTouch library version string.
    pub fn get_version_string() -> &'static str {
        unsafe {
            let ptr = ffi::SoundTouch_getVersionString();
            let c_str = core::ffi::CStr::from_ptr(ptr);
            c_str.to_str().unwrap()
        }
    }

    /// Returns nonzero if there aren't any `ready` samples.
    pub fn is_empty(&mut self) -> i32 {
        unsafe { ffi::FIFOSampleBuffer_isEmpty(&mut self.0 as *mut _ as *mut c_void) as i32 }
    }

    /// Get number of `ready` samples that can be received with
    /// function [`receive_samples`].
    ///
    /// [`receive_samples`]: SoundTouch::receive_samples
    pub fn num_samples(&mut self) -> i32 {
        unsafe { ffi::FIFOSampleBuffer_numSamples(&mut self.0 as *mut _ as *mut c_void) as i32 }
    }
}

#[cfg(not(windows))]
impl Drop for SoundTouch {
    fn drop(&mut self) {
        unsafe { ffi::SoundTouch_SoundTouch_destructor(&mut self.0) };
    }
}
