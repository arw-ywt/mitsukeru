// src/clock.rs

/// A monotonic time source for solver timeout and performance measurement.
pub trait Clock: Send + Sync {
    /// Returns the current monotonic timestamp in microseconds.
    fn now_micros(&self) -> u64;
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

#[cfg(feature = "std")]
impl Clock for SystemClock {
    fn now_micros(&self) -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::sync::LazyLock;
            use std::time::Instant;
            static START: LazyLock<Instant> = LazyLock::new(Instant::now);
            u64::try_from(START.elapsed().as_micros()).unwrap_or(u64::MAX)
        }
        #[cfg(target_arch = "wasm32")]
        {
            use std::sync::LazyLock;
            use web_time::Instant;
            static START: LazyLock<Instant> = LazyLock::new(Instant::now);
            u64::try_from(START.elapsed().as_micros()).unwrap_or(u64::MAX)
        }
    }
}

#[cfg(feature = "std")]
pub static DEFAULT_CLOCK: SystemClock = SystemClock;

/// Calculates elapsed milliseconds between two microsecond timestamps without loss of precision.
#[must_use]
pub fn elapsed_ms(start_micros: u64, end_micros: u64) -> f64 {
    let diff = end_micros.saturating_sub(start_micros);
    let ms_whole = u32::try_from(diff / 1000).unwrap_or(u32::MAX);
    let ms_rem = u32::try_from(diff % 1000).unwrap_or(0);
    f64::from(ms_whole) + (f64::from(ms_rem) / 1000.0)
}
