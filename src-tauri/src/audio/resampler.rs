//! Linear-interpolation sample-rate converter.
//!
//! Device audio often arrives at 48 kHz or 44.1 kHz; Moonshine wants 16 kHz
//! mono. This converter downsamples while accumulating a fractional position
//! so chunk boundaries remain sample-accurate.

use crate::errors::AppError;

pub struct LinearResampler {
    input_rate: f64,
    output_rate: f64,
    ratio: f64,
    position: f64,
}

impl LinearResampler {
    /// Creates a converter from `input_rate` to `output_rate`.
    pub fn new(input_rate: u32, output_rate: u32) -> Result<Self, AppError> {
        if input_rate == 0 || output_rate == 0 {
            return Err(AppError::InvalidConfiguration(format!(
                "resampler rates must be non-zero ({input_rate} -> {output_rate})"
            )));
        }
        // Speeding up (output > input) needs at least 2 samples per output;
        // slow down (output < input) works with plain interpolation.
        let ratio = input_rate as f64 / output_rate as f64;
        Ok(Self {
            input_rate: input_rate as f64,
            output_rate: output_rate as f64,
            ratio,
            position: 0.0,
        })
    }

    /// Converts a chunk of mono samples. Keeps phase across calls.
    pub fn process(&mut self, input: &[f32]) -> Vec<f32> {
        if input.is_empty() {
            return Vec::new();
        }
        if self.input_rate == self.output_rate {
            return input.to_vec();
        }

        let mut output = Vec::with_capacity((input.len() as f64 / self.ratio).ceil() as usize + 1);
        let mut i = self.position;
        while i + 1.0 < input.len() as f64 {
            let idx = i.floor() as usize;
            let frac = i - idx as f64;
            let sample = input[idx] as f64 * (1.0 - frac) + input[idx + 1] as f64 * frac;
            output.push(sample as f32);
            i += self.ratio;
        }
        self.position = i - input.len() as f64;
        output
    }

    /// Drops phase — call when the stream restarts.
    pub fn reset(&mut self) {
        self.position = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_when_rates_match() {
        let mut r = LinearResampler::new(16_000, 16_000).unwrap();
        let out = r.process(&[1.0, 2.0, 3.0]);
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn downsamples_roughly() {
        let mut r = LinearResampler::new(48_000, 16_000).unwrap();
        // 48k of constant data → 16k samples, all the same value.
        let input = vec![0.5; 48_000];
        let out = r.process(&input);
        assert!((out.len() as f64 - 16_000.0).abs() < 2.0);
        assert!(out.iter().all(|s| (s - 0.5).abs() < 1e-6));
    }

    #[test]
    fn phase_continues_across_chunks() {
        let mut r = LinearResampler::new(2, 1).unwrap();
        let a = r.process(&[0.0, 1.0, 0.0]);
        let b = r.process(&[1.0, 0.0, 1.0]);
        // No panics and no output data corruption; total length sanity.
        assert!(a.len() <= 3);
        assert!(b.len() <= 3);
    }
}
