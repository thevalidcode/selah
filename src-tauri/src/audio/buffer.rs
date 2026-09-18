//! Thread-safe sample buffer shared by the capture callback (producer) and
//! the speech worker (consumer).
//!
//! The buffer is CAP'd: if the consumer falls behind, the oldest samples are
//! dropped rather than letting memory grow without bound.

use std::collections::VecDeque;
use std::sync::Mutex;

pub struct AudioBuffer {
    inner: Mutex<Inner>,
    capacity: usize,
}

struct Inner {
    samples: VecDeque<f32>,
}

impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(Inner {
                samples: VecDeque::with_capacity(capacity.min(64 * 1024)),
            }),
            capacity: capacity.max(1),
        }
    }

    /// Appends samples. Drops oldest samples when over capacity.
    pub fn push(&self, samples: &[f32]) {
        let mut inner = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return, // poisoned buffer: skip rather than panic in audio callback
        };
        for &s in samples {
            if inner.samples.len() >= self.capacity {
                inner.samples.pop_front();
            }
            inner.samples.push_back(s);
        }
    }

    /// Removes and returns every buffered sample.
    pub fn drain(&self) -> Vec<f32> {
        let mut inner = match self.inner.lock() {
            Ok(g) => g,
            Err(_) => return Vec::new(),
        };
        inner.samples.drain(..).collect()
    }

    pub fn sample_count(&self) -> usize {
        self.inner.lock().map(|g| g.samples.len()).unwrap_or(0)
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.inner.lock() {
            g.samples.clear();
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_drain_roundtrip() {
        let buffer = AudioBuffer::new(4096);
        buffer.push(&[1.0, 2.0, 3.0]);
        let got = buffer.drain();
        assert_eq!(got, vec![1.0, 2.0, 3.0]);
        assert_eq!(buffer.sample_count(), 0);
    }

    #[test]
    fn caps_at_capacity() {
        let buffer = AudioBuffer::new(4);
        buffer.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(buffer.sample_count(), 4);
        assert_eq!(buffer.drain(), vec![3.0, 4.0, 5.0, 6.0]);
    }
}
