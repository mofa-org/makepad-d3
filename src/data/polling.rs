//! Polling data source for periodic data fetching
//!
//! Provides a time-based polling mechanism that triggers data fetch
//! callbacks at specified intervals.
//!
//! # Example
//!
//! ```
//! use makepad_d3::data::{PollingDataSource, DataPoint};
//!
//! let mut source = PollingDataSource::new(1000); // Poll every 1000ms
//!
//! // In render loop, check if it's time to fetch
//! let current_time = 1.5; // seconds since start
//! if source.should_fetch(current_time) {
//!     // Fetch new data and update source
//!     let new_data = vec![DataPoint::from_y(100.0)];
//!     source.update_data(new_data);
//! }
//! ```

use super::{DataPoint, DataSource, DataSourceConfig, DataSourceEvent, DataSourceState};
use std::collections::VecDeque;

/// Polling strategy
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum PollingStrategy {
    /// Fixed interval between polls
    #[default]
    FixedInterval,
    /// Exponential backoff on errors
    ExponentialBackoff,
    /// Adaptive based on data change rate
    Adaptive,
}

/// Configuration for polling behavior
#[derive(Clone, Debug)]
pub struct PollingConfig {
    /// Base polling interval in milliseconds
    pub interval_ms: u64,
    /// Minimum interval for adaptive polling
    pub min_interval_ms: u64,
    /// Maximum interval for adaptive polling
    pub max_interval_ms: u64,
    /// Polling strategy
    pub strategy: PollingStrategy,
    /// Maximum retries on error
    pub max_retries: u32,
    /// Current backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000,
            min_interval_ms: 1000,
            max_interval_ms: 60000,
            strategy: PollingStrategy::FixedInterval,
            max_retries: 3,
            backoff_multiplier: 2.0,
        }
    }
}

impl PollingConfig {
    /// Create config for real-time updates
    pub fn realtime() -> Self {
        Self {
            interval_ms: 1000,
            min_interval_ms: 100,
            max_interval_ms: 5000,
            strategy: PollingStrategy::Adaptive,
            ..Default::default()
        }
    }

    /// Create config for background updates
    pub fn background() -> Self {
        Self {
            interval_ms: 30000,
            strategy: PollingStrategy::ExponentialBackoff,
            ..Default::default()
        }
    }

    /// Set interval
    pub fn with_interval(mut self, ms: u64) -> Self {
        self.interval_ms = ms;
        self
    }

    /// Set strategy
    pub fn with_strategy(mut self, strategy: PollingStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

/// Polling state
#[derive(Clone, Copy, Debug, Default)]
pub struct PollingState {
    /// Last poll time (seconds)
    pub last_poll_time: f64,
    /// Next scheduled poll time (seconds)
    pub next_poll_time: f64,
    /// Current interval (may differ from config due to backoff/adaptive)
    pub current_interval_ms: u64,
    /// Consecutive error count
    pub error_count: u32,
    /// Total poll count
    pub poll_count: u64,
    /// Is currently fetching
    pub is_fetching: bool,
}

/// Polling data source with time-based fetch triggers
pub struct PollingDataSource {
    /// Data buffer
    data: Vec<DataPoint>,
    /// Pending events
    events: VecDeque<DataSourceEvent>,
    /// Current state
    state: DataSourceState,
    /// Data source configuration
    source_config: DataSourceConfig,
    /// Polling configuration
    polling_config: PollingConfig,
    /// Polling state
    polling_state: PollingState,
}

impl PollingDataSource {
    /// Create a new polling data source with interval in milliseconds
    pub fn new(interval_ms: u64) -> Self {
        Self {
            data: Vec::new(),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            source_config: DataSourceConfig::default(),
            polling_config: PollingConfig::default().with_interval(interval_ms),
            polling_state: PollingState {
                current_interval_ms: interval_ms,
                ..Default::default()
            },
        }
    }

    /// Create with full configuration
    pub fn with_config(source_config: DataSourceConfig, polling_config: PollingConfig) -> Self {
        Self {
            data: Vec::new(),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            source_config,
            polling_config: polling_config.clone(),
            polling_state: PollingState {
                current_interval_ms: polling_config.interval_ms,
                ..Default::default()
            },
        }
    }

    /// Check if it's time to fetch new data
    pub fn should_fetch(&self, current_time: f64) -> bool {
        if self.state != DataSourceState::Connected {
            return false;
        }
        if self.polling_state.is_fetching {
            return false;
        }
        current_time >= self.polling_state.next_poll_time
    }

    /// Mark fetch as started
    pub fn begin_fetch(&mut self, current_time: f64) {
        self.polling_state.is_fetching = true;
        self.polling_state.last_poll_time = current_time;
    }

    /// Update data after successful fetch
    pub fn update_data(&mut self, points: Vec<DataPoint>) {
        let old_len = self.data.len();
        self.data = points.clone();
        self.trim_to_max();

        self.polling_state.is_fetching = false;
        self.polling_state.poll_count += 1;
        self.polling_state.error_count = 0;

        // Calculate next poll time
        self.calculate_next_poll_time();

        // Emit event
        if old_len == 0 || self.data.len() != old_len {
            self.events.push_back(DataSourceEvent::Replace(points));
        } else {
            self.events
                .push_back(DataSourceEvent::Update { index: 0, points });
        }
    }

    /// Append new data after fetch
    pub fn append_data(&mut self, points: Vec<DataPoint>) {
        self.data.extend(points.clone());
        self.trim_to_max();

        self.polling_state.is_fetching = false;
        self.polling_state.poll_count += 1;
        self.polling_state.error_count = 0;

        self.calculate_next_poll_time();
        self.events.push_back(DataSourceEvent::Append(points));
    }

    /// Report fetch error
    pub fn report_error(&mut self, error: String) {
        self.polling_state.is_fetching = false;
        self.polling_state.error_count += 1;

        // Apply backoff strategy
        if self.polling_config.strategy == PollingStrategy::ExponentialBackoff {
            self.apply_backoff();
        }

        self.calculate_next_poll_time();

        // Check if max retries exceeded
        if self.polling_state.error_count >= self.polling_config.max_retries {
            self.state = DataSourceState::Error;
            self.events.push_back(DataSourceEvent::Error(format!(
                "Max retries exceeded: {}",
                error
            )));
        } else {
            self.events.push_back(DataSourceEvent::Error(error));
        }
    }

    /// Reset error state and retry
    pub fn retry(&mut self) {
        self.polling_state.error_count = 0;
        self.polling_state.current_interval_ms = self.polling_config.interval_ms;
        self.state = DataSourceState::Connected;
    }

    /// Get polling state
    pub fn polling_state(&self) -> &PollingState {
        &self.polling_state
    }

    /// Get polling configuration
    pub fn polling_config(&self) -> &PollingConfig {
        &self.polling_config
    }

    /// Set polling interval (resets to base interval)
    pub fn set_interval(&mut self, interval_ms: u64) {
        self.polling_config.interval_ms = interval_ms;
        self.polling_state.current_interval_ms = interval_ms;
    }

    /// Get data reference
    pub fn data(&self) -> &[DataPoint] {
        &self.data
    }

    /// Get data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn calculate_next_poll_time(&mut self) {
        let interval_secs = self.polling_state.current_interval_ms as f64 / 1000.0;
        self.polling_state.next_poll_time = self.polling_state.last_poll_time + interval_secs;
    }

    fn apply_backoff(&mut self) {
        let new_interval = (self.polling_state.current_interval_ms as f64
            * self.polling_config.backoff_multiplier) as u64;
        self.polling_state.current_interval_ms =
            new_interval.min(self.polling_config.max_interval_ms);
    }

    fn trim_to_max(&mut self) {
        if self.source_config.max_points > 0 && self.data.len() > self.source_config.max_points {
            let excess = self.data.len() - self.source_config.max_points;
            self.data.drain(0..excess);
        }
    }
}

impl DataSource for PollingDataSource {
    fn poll(&mut self) -> DataSourceEvent {
        self.events.pop_front().unwrap_or(DataSourceEvent::None)
    }

    fn state(&self) -> DataSourceState {
        self.state
    }

    fn connect(&mut self) {
        self.state = DataSourceState::Connected;
        self.polling_state.next_poll_time = 0.0; // Trigger immediate fetch
        self.events.push_back(DataSourceEvent::Connected);
    }

    fn disconnect(&mut self) {
        self.state = DataSourceState::Disconnected;
        self.events.push_back(DataSourceEvent::Disconnected);
    }

    fn pause(&mut self) {
        self.state = DataSourceState::Paused;
    }

    fn resume(&mut self) {
        self.state = DataSourceState::Connected;
    }

    fn snapshot(&self) -> Vec<DataPoint> {
        self.data.clone()
    }

    fn config(&self) -> &DataSourceConfig {
        &self.source_config
    }
}

/// Helper for creating polling sources with closures
pub struct PollingSourceBuilder<F>
where
    F: FnMut() -> Result<Vec<DataPoint>, String>,
{
    fetch_fn: F,
    polling_config: PollingConfig,
    source_config: DataSourceConfig,
}

impl<F> PollingSourceBuilder<F>
where
    F: FnMut() -> Result<Vec<DataPoint>, String>,
{
    /// Create a new builder with fetch function
    pub fn new(fetch_fn: F) -> Self {
        Self {
            fetch_fn,
            polling_config: PollingConfig::default(),
            source_config: DataSourceConfig::default(),
        }
    }

    /// Set polling interval
    pub fn interval(mut self, ms: u64) -> Self {
        self.polling_config.interval_ms = ms;
        self
    }

    /// Set polling strategy
    pub fn strategy(mut self, strategy: PollingStrategy) -> Self {
        self.polling_config.strategy = strategy;
        self
    }

    /// Set max points
    pub fn max_points(mut self, max: usize) -> Self {
        self.source_config.max_points = max;
        self
    }

    /// Build polling source and fetch function
    pub fn build(self) -> (PollingDataSource, F) {
        let source = PollingDataSource::with_config(self.source_config, self.polling_config);
        (source, self.fetch_fn)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polling_source_new() {
        let source = PollingDataSource::new(5000);
        assert!(source.is_empty());
        assert_eq!(source.polling_config().interval_ms, 5000);
    }

    #[test]
    fn test_polling_should_fetch() {
        let source = PollingDataSource::new(1000);

        // Should fetch immediately (next_poll_time is 0)
        assert!(source.should_fetch(0.0));
        assert!(source.should_fetch(1.0));
    }

    #[test]
    fn test_polling_fetch_cycle() {
        let mut source = PollingDataSource::new(1000);

        // Begin fetch
        source.begin_fetch(0.0);
        assert!(source.polling_state().is_fetching);
        assert!(!source.should_fetch(0.0));

        // Complete fetch
        source.update_data(vec![DataPoint::from_y(100.0)]);
        assert!(!source.polling_state().is_fetching);
        assert_eq!(source.len(), 1);

        // Next fetch should be at ~1.0 seconds
        assert!(!source.should_fetch(0.5));
        assert!(source.should_fetch(1.0));
    }

    #[test]
    fn test_polling_error_backoff() {
        let config = PollingConfig::default()
            .with_interval(1000)
            .with_strategy(PollingStrategy::ExponentialBackoff);
        let mut source = PollingDataSource::with_config(DataSourceConfig::default(), config);

        source.begin_fetch(0.0);
        source.report_error("Test error".to_string());

        // Interval should have doubled
        assert_eq!(source.polling_state().current_interval_ms, 2000);
        assert_eq!(source.polling_state().error_count, 1);
    }

    #[test]
    fn test_polling_max_retries() {
        let mut config = PollingConfig::default();
        config.max_retries = 2;
        let mut source = PollingDataSource::with_config(DataSourceConfig::default(), config);

        source.begin_fetch(0.0);
        source.report_error("Error 1".to_string());
        assert_eq!(source.state(), DataSourceState::Connected);

        source.begin_fetch(1.0);
        source.report_error("Error 2".to_string());
        assert_eq!(source.state(), DataSourceState::Error);
    }

    #[test]
    fn test_polling_retry() {
        let mut source = PollingDataSource::new(1000);

        // Simulate errors
        source.begin_fetch(0.0);
        source.report_error("Error".to_string());
        source.begin_fetch(2.0);
        source.report_error("Error".to_string());

        // Retry resets state
        source.retry();
        assert_eq!(source.polling_state().error_count, 0);
        assert_eq!(source.polling_state().current_interval_ms, 1000);
        assert_eq!(source.state(), DataSourceState::Connected);
    }

    #[test]
    fn test_polling_append_data() {
        let mut source = PollingDataSource::new(1000);

        source.begin_fetch(0.0);
        source.append_data(vec![DataPoint::from_y(10.0)]);

        source.begin_fetch(1.0);
        source.append_data(vec![DataPoint::from_y(20.0)]);

        assert_eq!(source.len(), 2);
        assert_eq!(source.data()[0].y, 10.0);
        assert_eq!(source.data()[1].y, 20.0);
    }

    #[test]
    fn test_polling_pause_resume() {
        let mut source = PollingDataSource::new(1000);

        source.pause();
        assert!(!source.should_fetch(0.0)); // Shouldn't fetch while paused

        source.resume();
        assert!(source.should_fetch(0.0)); // Should fetch after resume
    }
}
