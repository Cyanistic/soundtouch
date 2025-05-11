use core::ptr::null_mut;

use ffi::{BPMDetect as BPMDetectSys, BPMDetect_BPMDetect_destructor};
use core::ffi::c_int;
use soundtouch_ffi as ffi;

/// Beats-per-minute (BPM) detection routine.
///
/// The beat detection algorithm works as follows:
/// - Use function [`input_samples`] to input a chunks of samples to the class for
///   analysis. It's a good idea to enter a large sound file or stream in smallish
///   chunks of around few kilosamples in order not to extinguish too much RAM memory.
/// - Input sound data is decimated to approx 500 Hz to reduce calculation burden,
///   which is basically ok as low (bass) frequencies mostly determine the beat rate.
///   Simple averaging is used for anti-alias filtering because the resulting signal
///   quality isn't of that high importance.
/// - Decimated sound data is enveloped, i.e. the amplitude shape is detected by
///   taking absolute value that's smoothed by sliding average. Signal levels that
///   are below a couple of times the general RMS amplitude level are cut away to
///   leave only notable peaks there.
/// - Repeating sound patterns (e.g. beats) are detected by calculating short-term
///   autocorrelation function of the enveloped signal.
/// - After whole sound data file has been analyzed as above, the bpm level is
///   detected by function [`get_bpm`] that finds the highest peak of the autocorrelation
///   function, calculates it's precise location and converts this reading to bpm's.
///
///  [`get_bpm`]: BPMDetect::get_bpm
///  [`input_samples`]: BPMDetect::input_samples
pub struct BPMDetect(BPMDetectSys);

unsafe impl Send for BPMDetect {}

impl Default for BPMDetect {
    fn default() -> Self {
        Self(unsafe { BPMDetectSys::new(2, 44100) })
    }
}

impl BPMDetect {
    /// Creates a new BPMDetect instance with the given channels and sample rate.
    pub fn new(num_channels: u32, sample_rate: u32) -> Self {
        Self(unsafe { BPMDetectSys::new(num_channels as c_int, sample_rate as c_int) })
    }

    /// Inputs a block of samples for analyzing: Envelopes the samples and then
    /// updates the autocorrelation estimation. When whole song data has been input
    /// in smaller blocks using this function, read the resulting bpm with [`get_bpm`]
    /// function.
    ///
    /// Notice that data in `samples` array can be disrupted in processing.
    ///
    /// [`get_bpm`]: BPMDetect::get_bpm
    pub fn input_samples(&mut self, samples: &[f32]) {
        unsafe {
            self.0
                .inputSamples(samples.as_ptr(), samples.len() as c_int / self.0.channels)
        }
    }

    /// Analyzes the results and returns the BPM rate. Use this function to read result
    /// after whole song data has been input to the class by consecutive calls of
    /// [`input_samples`] function.
    ///
    /// [`input_samples`]: BPMDetect::input_samples
    pub fn get_bpm(&mut self) -> f32 {
        unsafe { ffi::BPMDetect_getBpm(&mut self.0) }
    }

    /// Get beat position arrays. Note: The array includes also really low beat detection values
    /// in absence of clear strong beats. Consumer may wish to filter low values away.
    /// - `pos` receive array of beat positions
    /// - `values` receive array of beat detection strengths
    /// - `max_num` indicates max.size of `pos` and `values` array.  
    ///
    /// You can query a suitable array sized by calling the [`query_size`] function.
    /// Returns the number of beats in the arrays.
    ///
    /// [`query_size`]: BPMDetect::query_size
    pub fn get_beats(&mut self, pos: &mut [f32], values: &mut [f32], max_num: i32) -> i32 {
        unsafe {
            self.0
                .getBeats(pos.as_mut_ptr(), values.as_mut_ptr(), max_num)
        }
    }

    /// Queries a suitable array sized for [`get_beats`].
    ///
    /// [`get_beats`]: BPMDetect::get_beats
    pub fn query_size(&mut self, max_num: i32) -> i32 {
        unsafe { self.0.getBeats(null_mut(), null_mut(), max_num) }
    }

    /// Detects individual beat positions.
    pub fn update_beat_pos(&mut self, process_samples: i32) {
        unsafe { self.0.updateBeatPos(process_samples) }
    }

    /// Removes constant bias from xcorr data.
    pub fn remove_bias(&mut self) {
        unsafe { self.0.removeBias() }
    }

    /// Calculates amplitude envelope for the buffer of samples.
    /// Result is output to `samples`.
    #[cfg(not(all(target_env="gnu", target_os="windows")))]
    pub fn calc_envelope(&mut self, samples: &mut [f32]) {
        unsafe {
            self.0
                .calcEnvelope(samples.as_mut_ptr(), samples.len() as c_int)
        }
    }

    /// Decimates samples to approx. 500 Hz.
    ///
    /// Returns the number of output samples.
    pub fn decimate(&mut self, dest: &mut [f32], src: &[f32], numsamples: i32) -> i32 {
        unsafe { self.0.decimate(dest.as_mut_ptr(), src.as_ptr(), numsamples) }
    }

    /// Updates auto-correlation function for given number of decimated samples that
    /// are read from the internal `buffer' pipe (samples aren't removed from the pipe
    /// though).
    pub fn update_x_corr(&mut self, process_samples: i32) {
        unsafe { self.0.updateXCorr(process_samples) }
    }
}

#[cfg(not(windows))]
impl Drop for BPMDetect {
    fn drop(&mut self) {
        unsafe { BPMDetect_BPMDetect_destructor(&mut self.0) }
    }
}
