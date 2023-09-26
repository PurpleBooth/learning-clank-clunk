//! # clank-clunk
//!
//! A library for making noise.
#![warn(
    rust_2018_idioms,
    unused,
    rust_2021_compatibility,
    nonstandard_style,
    future_incompatible,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::unwrap_used,
    clippy::missing_assert_message,
    clippy::todo,
    clippy::allow_attributes_without_reason,
    clippy::panic,
    clippy::panicking_unwrap,
    clippy::panic_in_result_fn
)]

use std::num::FpCategory::Zero;
use std::time::Duration;

use rodio::dynamic_mixer::mixer;
use rodio::source::SineWave;
use rodio::{OutputStream, Sample, Sink, Source, source};
use rodio::queue::queue;
use staff::{midi, Chord};

fn split_adsr<S: Source<Item = A> + Sized, A: Sample + Sized + Clone>(
    source: S,
    attack_duration: Duration,
    decay_duration: Duration,
    sustain_duration: Duration,
    release: Duration,
) -> Vec<impl Source<Item = A> + Clone> {
    let data = source
        .take_duration(attack_duration + decay_duration + sustain_duration + release)
        .buffered();

    let attack_samples = data
        .clone()
        .skip_duration(Duration::ZERO)
        .take_duration(attack_duration);
    let decay_samples = data
        .clone()
        .skip_duration(attack_duration)
        .take_duration(decay_duration);
    let sustain_samples = data
        .clone()
        .skip_duration(attack_duration + decay_duration)
        .take_duration(sustain_duration);
    let release_samples = data
        .skip_duration(attack_duration + decay_duration + sustain_duration)
        .take_duration(release);

    vec![
        attack_samples,
        decay_samples,
        sustain_samples,
        release_samples,
    ]
}

#[allow(clippy::cast_possible_truncation)]
fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().expect("sdfaetbsrtbs");
    let sink = Sink::try_new(&stream_handle).expect("aaaa");
    let chord = Chord::major(midi!(C, 4));
    let (input, output) = mixer(1, 44100);

    for note in chord {
        let frequency = note.frequency() as f32;
        let source = SineWave::new(frequency);
        input.add(source);
    }

    let attack_duration = Duration::from_secs_f32(0.1);
    let decay_duration = Duration::from_secs_f32(0.1);
    let sustain_duration = Duration::from_secs_f32(1.0);
    let release_duration = Duration::from_secs_f32(5.0);
    let mut split = split_adsr(
        output,
        attack_duration,
        decay_duration,
        sustain_duration,
        release_duration,
    );

    let (inqueue, outqueue) = queue(false);

    let attack = split.pop().expect("a better error messge").amplify(1.1);

    inqueue.append(attack.clone().fade_in(attack_duration));
    let decay = split.pop().expect("a better error messge");
    inqueue.append(attack.take_crossfade_with(decay, decay_duration));
    let sustain = split.pop().expect("a better error messge");
    inqueue.append(sustain.clone());
    inqueue.append(sustain.take_crossfade_with(source::Zero::<f32>::new(1, 44100), release_duration));
    sink.append(outqueue);

    sink.sleep_until_end();
}
