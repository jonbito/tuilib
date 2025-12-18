//! Debouncing and throttling utilities for event rate limiting.
//!
//! This module provides utilities to control the rate of event processing,
//! preventing rapid duplicate events from overwhelming the UI.

use std::time::{Duration, Instant};

/// A debouncer that delays processing until events stop arriving.
///
/// Debouncing is useful for events that fire rapidly but only the final
/// state matters (like resize events or search input).
///
/// # How It Works
///
/// The debouncer only allows an event to proceed if a specified delay
/// has passed since the last event. Each new event resets the timer.
///
/// # Examples
///
/// ```rust
/// use tuilib::event::Debouncer;
/// use std::time::Duration;
///
/// let mut debouncer = Debouncer::new(Duration::from_millis(100));
///
/// // First call - allowed immediately
/// assert!(debouncer.should_process());
///
/// // Immediate second call - blocked (still within delay)
/// assert!(!debouncer.should_process());
///
/// // After waiting the delay duration, calls are allowed again
/// std::thread::sleep(Duration::from_millis(110));
/// assert!(debouncer.should_process());
/// ```
#[derive(Debug, Clone)]
pub struct Debouncer {
    delay: Duration,
    last_event: Option<Instant>,
}

impl Debouncer {
    /// Creates a new debouncer with the specified delay.
    ///
    /// # Arguments
    ///
    /// * `delay` - Minimum time between processed events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::Debouncer;
    /// use std::time::Duration;
    ///
    /// let debouncer = Debouncer::new(Duration::from_millis(50));
    /// ```
    pub fn new(delay: Duration) -> Self {
        Self {
            delay,
            last_event: None,
        }
    }

    /// Checks if enough time has passed to process a new event.
    ///
    /// If this returns `true`, it also updates the internal timestamp,
    /// so subsequent calls within the delay window will return `false`.
    ///
    /// # Returns
    ///
    /// `true` if the event should be processed, `false` if it should be skipped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::Debouncer;
    /// use std::time::Duration;
    ///
    /// let mut debouncer = Debouncer::new(Duration::from_millis(100));
    ///
    /// if debouncer.should_process() {
    ///     // Process the event
    /// }
    /// ```
    pub fn should_process(&mut self) -> bool {
        let now = Instant::now();

        match self.last_event {
            Some(last) if now.duration_since(last) < self.delay => false,
            _ => {
                self.last_event = Some(now);
                true
            }
        }
    }

    /// Resets the debouncer state.
    ///
    /// After calling this, the next `should_process()` will return `true`.
    pub fn reset(&mut self) {
        self.last_event = None;
    }

    /// Returns the debounce delay duration.
    pub fn delay(&self) -> Duration {
        self.delay
    }

    /// Sets a new debounce delay.
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = delay;
    }

    /// Returns the time since the last processed event.
    ///
    /// Returns `None` if no event has been processed yet.
    pub fn time_since_last(&self) -> Option<Duration> {
        self.last_event.map(|last| last.elapsed())
    }

    /// Returns the remaining time until the next event can be processed.
    ///
    /// Returns `Duration::ZERO` if an event can be processed immediately.
    pub fn remaining(&self) -> Duration {
        match self.last_event {
            Some(last) => {
                let elapsed = last.elapsed();
                if elapsed >= self.delay {
                    Duration::ZERO
                } else {
                    self.delay - elapsed
                }
            }
            None => Duration::ZERO,
        }
    }
}

impl Default for Debouncer {
    fn default() -> Self {
        Self::new(Duration::from_millis(50))
    }
}

/// A throttle that limits event processing to a maximum rate.
///
/// Unlike debouncing, throttling allows the first event immediately
/// and then enforces a minimum interval between subsequent events.
///
/// # How It Works
///
/// The throttle allows events at a fixed rate. The first event is always
/// processed immediately, and then subsequent events are only processed
/// if the minimum interval has passed.
///
/// # Examples
///
/// ```rust
/// use tuilib::event::Throttle;
/// use std::time::Duration;
///
/// let mut throttle = Throttle::new(Duration::from_millis(100));
///
/// // First call - allowed immediately
/// assert!(throttle.should_process());
///
/// // Immediate second call - blocked
/// assert!(!throttle.should_process());
///
/// // After waiting, calls are allowed again
/// std::thread::sleep(Duration::from_millis(110));
/// assert!(throttle.should_process());
/// ```
#[derive(Debug, Clone)]
pub struct Throttle {
    interval: Duration,
    last_allowed: Option<Instant>,
}

impl Throttle {
    /// Creates a new throttle with the specified interval.
    ///
    /// # Arguments
    ///
    /// * `interval` - Minimum time between processed events
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::Throttle;
    /// use std::time::Duration;
    ///
    /// // Allow at most 10 events per second
    /// let throttle = Throttle::new(Duration::from_millis(100));
    /// ```
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_allowed: None,
        }
    }

    /// Creates a throttle from a rate (events per second).
    ///
    /// # Arguments
    ///
    /// * `events_per_second` - Maximum number of events to allow per second
    ///
    /// # Panics
    ///
    /// Panics if `events_per_second` is 0.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tuilib::event::Throttle;
    ///
    /// // Allow at most 60 events per second
    /// let throttle = Throttle::from_rate(60);
    /// ```
    pub fn from_rate(events_per_second: u32) -> Self {
        assert!(events_per_second > 0, "events_per_second must be > 0");
        let interval = Duration::from_secs(1) / events_per_second;
        Self::new(interval)
    }

    /// Checks if enough time has passed to process a new event.
    ///
    /// If this returns `true`, it also updates the internal timestamp.
    ///
    /// # Returns
    ///
    /// `true` if the event should be processed, `false` if it should be skipped.
    pub fn should_process(&mut self) -> bool {
        let now = Instant::now();

        match self.last_allowed {
            Some(last) if now.duration_since(last) < self.interval => false,
            _ => {
                self.last_allowed = Some(now);
                true
            }
        }
    }

    /// Resets the throttle state.
    ///
    /// After calling this, the next `should_process()` will return `true`.
    pub fn reset(&mut self) {
        self.last_allowed = None;
    }

    /// Returns the throttle interval.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Sets a new throttle interval.
    pub fn set_interval(&mut self, interval: Duration) {
        self.interval = interval;
    }

    /// Returns the time since the last allowed event.
    ///
    /// Returns `None` if no event has been allowed yet.
    pub fn time_since_last(&self) -> Option<Duration> {
        self.last_allowed.map(|last| last.elapsed())
    }

    /// Returns the remaining time until the next event can be processed.
    ///
    /// Returns `Duration::ZERO` if an event can be processed immediately.
    pub fn remaining(&self) -> Duration {
        match self.last_allowed {
            Some(last) => {
                let elapsed = last.elapsed();
                if elapsed >= self.interval {
                    Duration::ZERO
                } else {
                    self.interval - elapsed
                }
            }
            None => Duration::ZERO,
        }
    }

    /// Returns the maximum rate as events per second.
    pub fn rate(&self) -> f64 {
        1.0 / self.interval.as_secs_f64()
    }
}

impl Default for Throttle {
    fn default() -> Self {
        Self::from_rate(60) // 60 Hz default
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_debouncer_creation() {
        let debouncer = Debouncer::new(Duration::from_millis(100));
        assert_eq!(debouncer.delay(), Duration::from_millis(100));
    }

    #[test]
    fn test_debouncer_default() {
        let debouncer = Debouncer::default();
        assert_eq!(debouncer.delay(), Duration::from_millis(50));
    }

    #[test]
    fn test_debouncer_first_call_allowed() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        assert!(debouncer.should_process());
    }

    #[test]
    fn test_debouncer_immediate_second_call_blocked() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        assert!(debouncer.should_process());
        assert!(!debouncer.should_process());
    }

    #[test]
    fn test_debouncer_after_delay_allowed() {
        let mut debouncer = Debouncer::new(Duration::from_millis(10));
        assert!(debouncer.should_process());
        thread::sleep(Duration::from_millis(15));
        assert!(debouncer.should_process());
    }

    #[test]
    fn test_debouncer_reset() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        assert!(debouncer.should_process());
        assert!(!debouncer.should_process());
        debouncer.reset();
        assert!(debouncer.should_process());
    }

    #[test]
    fn test_debouncer_set_delay() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        debouncer.set_delay(Duration::from_millis(200));
        assert_eq!(debouncer.delay(), Duration::from_millis(200));
    }

    #[test]
    fn test_debouncer_time_since_last() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        assert!(debouncer.time_since_last().is_none());
        debouncer.should_process();
        thread::sleep(Duration::from_millis(5));
        let elapsed = debouncer.time_since_last().unwrap();
        assert!(elapsed >= Duration::from_millis(5));
    }

    #[test]
    fn test_debouncer_remaining() {
        let mut debouncer = Debouncer::new(Duration::from_millis(100));
        assert_eq!(debouncer.remaining(), Duration::ZERO);
        debouncer.should_process();
        let remaining = debouncer.remaining();
        assert!(remaining > Duration::ZERO);
        assert!(remaining <= Duration::from_millis(100));
    }

    #[test]
    fn test_throttle_creation() {
        let throttle = Throttle::new(Duration::from_millis(100));
        assert_eq!(throttle.interval(), Duration::from_millis(100));
    }

    #[test]
    fn test_throttle_from_rate() {
        let throttle = Throttle::from_rate(10);
        assert_eq!(throttle.interval(), Duration::from_millis(100));
    }

    #[test]
    #[should_panic(expected = "events_per_second must be > 0")]
    fn test_throttle_from_rate_zero_panics() {
        Throttle::from_rate(0);
    }

    #[test]
    fn test_throttle_default() {
        let throttle = Throttle::default();
        // 60 Hz = ~16.67ms interval
        assert!(throttle.interval() > Duration::from_millis(16));
        assert!(throttle.interval() < Duration::from_millis(17));
    }

    #[test]
    fn test_throttle_first_call_allowed() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        assert!(throttle.should_process());
    }

    #[test]
    fn test_throttle_immediate_second_call_blocked() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        assert!(throttle.should_process());
        assert!(!throttle.should_process());
    }

    #[test]
    fn test_throttle_after_interval_allowed() {
        let mut throttle = Throttle::new(Duration::from_millis(10));
        assert!(throttle.should_process());
        thread::sleep(Duration::from_millis(15));
        assert!(throttle.should_process());
    }

    #[test]
    fn test_throttle_reset() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        assert!(throttle.should_process());
        assert!(!throttle.should_process());
        throttle.reset();
        assert!(throttle.should_process());
    }

    #[test]
    fn test_throttle_set_interval() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        throttle.set_interval(Duration::from_millis(200));
        assert_eq!(throttle.interval(), Duration::from_millis(200));
    }

    #[test]
    fn test_throttle_time_since_last() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        assert!(throttle.time_since_last().is_none());
        throttle.should_process();
        thread::sleep(Duration::from_millis(5));
        let elapsed = throttle.time_since_last().unwrap();
        assert!(elapsed >= Duration::from_millis(5));
    }

    #[test]
    fn test_throttle_remaining() {
        let mut throttle = Throttle::new(Duration::from_millis(100));
        assert_eq!(throttle.remaining(), Duration::ZERO);
        throttle.should_process();
        let remaining = throttle.remaining();
        assert!(remaining > Duration::ZERO);
        assert!(remaining <= Duration::from_millis(100));
    }

    #[test]
    fn test_throttle_rate() {
        let throttle = Throttle::from_rate(60);
        assert!((throttle.rate() - 60.0).abs() < 0.1);
    }

    #[test]
    fn test_debouncer_debug() {
        let debouncer = Debouncer::new(Duration::from_millis(100));
        let debug_str = format!("{:?}", debouncer);
        assert!(debug_str.contains("Debouncer"));
        assert!(debug_str.contains("delay"));
    }

    #[test]
    fn test_throttle_debug() {
        let throttle = Throttle::new(Duration::from_millis(100));
        let debug_str = format!("{:?}", throttle);
        assert!(debug_str.contains("Throttle"));
        assert!(debug_str.contains("interval"));
    }

    #[test]
    fn test_debouncer_clone() {
        let debouncer = Debouncer::new(Duration::from_millis(100));
        let cloned = debouncer.clone();
        assert_eq!(debouncer.delay(), cloned.delay());
    }

    #[test]
    fn test_throttle_clone() {
        let throttle = Throttle::new(Duration::from_millis(100));
        let cloned = throttle.clone();
        assert_eq!(throttle.interval(), cloned.interval());
    }
}
