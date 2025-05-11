## soundtouch

[![Crates.io](https://img.shields.io/crates/v/soundtouch.svg)](https://crates.io/crates/soundtouch)
[![Documentation](https://docs.rs/soundtouch/badge.svg)](https://docs.rs/soundtouch)

A safe utility wrapper around the SoundTouch C++ audio library. The API is very similar to the original C++ API. For dynamic and static linking configuration see [soundtouch-ffi's linking behavior](https://github.com/Cyanistic/soundtouch-ffi?tab=readme-ov-file#default-linking-behavior).

Most of the documentation is copied from the [SoundTouch repository](https://codeberg.org/soundtouch/soundtouch).
## High Level Example

```rust
use soundtouch::{SoundTouch, Setting};

let mut soundtouch = SoundTouch::new();
soundtouch
    .set_channels(2)
    .set_sample_rate(44100)
    .set_tempo(1.10)
    // Recommended setting to speed up processing
    .set_setting(Setting::UseQuickseek, 1);

// use actual audio samples here
let samples = vec![0.0; 44100 * 2];
let output_samples = soundtouch.generate_audio(&samples);

// do something with output_samples

```
## Low Level Example

```rust
use soundtouch::{SoundTouch, Setting};

let mut soundtouch = SoundTouch::new();
soundtouch
    .set_channels(2)
    .set_sample_rate(44100)
    .set_tempo(1.10)    
    // Recommended setting to speed up processing
    .set_setting(Setting::UseQuickseek, 1);

// use actual audio samples here
let mut samples = vec![0.0; 44100 * 2];

const BUF_SIZE: usize = 6720;
let mut new_samples: [f32; BUF_SIZE] = [0.0; BUF_SIZE];
let mut output_samples: Vec<f32> = Vec::with_capacity(samples.len());
soundtouch.put_samples(&samples, samples.len() / 2);
let mut n_samples = 1;
while n_samples != 0 {
    n_samples = soundtouch.receive_samples(
        new_samples.as_mut_slice(),
        BUF_SIZE / 2
        );
    output_samples.extend_from_slice(&new_samples);
}
soundtouch.flush();

// do something with output_samples

````
Both examples should produce the same output.


