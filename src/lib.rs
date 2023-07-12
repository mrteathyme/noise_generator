use rodio::Sample;
use rodio::Source;
use rand::prelude::*;
use std::f64::consts::*;

use rodio::{OutputStream, Sink};

#[derive(Clone)]
struct NoiseGenerator {
    sample_rate: u32,
    channels: u16,
    previous_low: f64,
    previous_high: f64,
    dynamic_gain: f64,
    max_amp: f64,
}

impl Iterator for NoiseGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let sample: f64 = 2.0 * rand::thread_rng().gen::<f64>() - 1.0;
        //Attenuate signals below 20hz to remove excessive gain in sub bass. ToDo: Rework filters
        //to be data structure that keeps track of own state, perhaps store a signal chain in
        //noisegenerator as a vector? ill figure it out.
        let sample = high_pass(self.previous_high, sample, self.sample_rate as f64, 20.0); 
        self.previous_high = sample;
        let sample = low_pass(self.previous_low, sample, self.sample_rate as f64, 40.0);
        self.previous_low = sample;
        if sample.abs() > self.max_amp {
            self.max_amp = 1.2*sample.abs();
            self.dynamic_gain = 1.0/self.max_amp;
        }
        let sample = sample * self.dynamic_gain;
        Some(sample as f32)
        //Some(rand::thread_rng().gen::<f64>() as f32)
    }
}

impl Source for NoiseGenerator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    fn channels(&self) -> u16 {
        self.channels
    }
}

pub fn test() {
    let mut noise = NoiseGenerator {sample_rate: 48000, channels: 2, previous_high: 0.0 , previous_low: 0.0, dynamic_gain: 1.0, max_amp: 0.0};
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    sink.append(noise.clone());
    loop {
        noise.next();
        println!("{:#?}", noise.dynamic_gain);
    }
    sink.sleep_until_end();
}

fn low_pass(old: f64, new: f64, sample_rate: f64, cutoff_frequency: f64) -> f64 {
    let radians_per_second = 2.0*PI*cutoff_frequency;
    let discrete_time = 1.0/sample_rate;
    //let alpha = discrete_time / (radians_per_second / discrete_time);
    let alpha = discrete_time / (discrete_time + (1.0 / cutoff_frequency));
    //(alpha * new + (1.0-alpha) * old)
    old + alpha * (new - old)
}

fn high_pass(old: f64, new: f64, sample_rate: f64, cutoff_frequency: f64) -> f64 {
    new - low_pass(old, new, sample_rate, cutoff_frequency)
}
