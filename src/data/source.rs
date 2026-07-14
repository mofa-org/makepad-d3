//! Data source traits and types for dynamic data binding
//!
//! Provides abstract interfaces for connecting charts to various data sources
//! including real-time streams, polling endpoints, and reactive datasets.
//!
//! # Example
//!
//! ```ignore
//! use makepad_d3::data::{DataSource, DataSourceEvent, DataPoint};
//!
//! // Implement custom data source
//! struct MyDataSource {
//!     buffer: Vec<DataPoint>,
//! }
//!
//! impl DataSource for MyDataSource {
//!     fn poll(&mut self) -> Option<DataSourceEvent> {
//!         // Return new data events
//!         None
//!     }
//! }
//! ```

use super::{ChartData, DataPoint, Dataset};
use std::collections::VecDeque;

/// Events emitted by data sources
#[derive(Clone, Debug)]
pub enum DataSourceEvent {
    /// New data point(s) appended
    Append(Vec<DataPoint>),
    /// Data point(s) updated at index
    Update {
        index: usize,
        points: Vec<DataPoint>,
    },
    /// Data point(s) removed at index
    Remove { index: usize, count: usize },
    /// Complete data replacement
    Replace(Vec<DataPoint>),
    /// Data source connected
    Connected,
    /// Data source disconnected
    Disconnected,
    /// Error occurred
    Error(String),
    /// No new events
    None,
}

/// Data source state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataSourceState {
    /// Not connected
    #[default]
    Disconnected,
    /// Connecting to source
    Connecting,
    /// Connected and receiving data
    Connected,
    /// Connection error
    Error,
    /// Paused (not receiving updates)
    Paused,
}

/// Configuration for data source behavior
#[derive(Clone, Debug)]
pub struct DataSourceConfig {
    /// Maximum number of points to retain (0 = unlimited)
    pub max_points: usize,
    /// Whether to auto-reconnect on disconnect
    pub auto_reconnect: bool,
    /// Reconnect delay in milliseconds
    pub reconnect_delay_ms: u64,
    /// Buffer size for incoming data
    pub buffer_size: usize,
    /// Whether to batch updates
    pub batch_updates: bool,
    /// Batch interval in milliseconds
    pub batch_interval_ms: u64,
}

impl Default for DataSourceConfig {
    fn default() -> Self {
        Self {
            max_points: 1000,
            auto_reconnect: true,
            reconnect_delay_ms: 5000,
            buffer_size: 100,
            batch_updates: false,
            batch_interval_ms: 16, // ~60fps
        }
    }
}

impl DataSourceConfig {
    /// Create config for real-time streaming
    pub fn realtime() -> Self {
        Self {
            max_points: 500,
            batch_updates: true,
            batch_interval_ms: 16,
            ..Default::default()
        }
    }

    /// Create config for historical data
    pub fn historical() -> Self {
        Self {
            max_points: 0, // Unlimited
            auto_reconnect: false,
            batch_updates: false,
            ..Default::default()
        }
    }

    /// Set maximum points
    pub fn with_max_points(mut self, max: usize) -> Self {
        self.max_points = max;
        self
    }

    /// Set auto-reconnect
    pub fn with_auto_reconnect(mut self, auto: bool) -> Self {
        self.auto_reconnect = auto;
        self
    }

    /// Set batch updates
    pub fn with_batching(mut self, batch: bool, interval_ms: u64) -> Self {
        self.batch_updates = batch;
        self.batch_interval_ms = interval_ms;
        self
    }
}

/// Trait for data sources that provide chart data
///
/// Data sources can be polled for events and provide data points
/// to charts. Implementations can wrap WebSockets, HTTP endpoints,
/// file streams, or any other data provider.
pub trait DataSource: Send {
    /// Poll for new events
    ///
    /// Called periodically to check for new data. Returns `DataSourceEvent::None`
    /// if no new data is available.
    fn poll(&mut self) -> DataSourceEvent;

    /// Get current state
    fn state(&self) -> DataSourceState;

    /// Connect to the data source
    fn connect(&mut self);

    /// Disconnect from the data source
    fn disconnect(&mut self);

    /// Pause receiving updates
    fn pause(&mut self);

    /// Resume receiving updates
    fn resume(&mut self);

    /// Get current data snapshot
    fn snapshot(&self) -> Vec<DataPoint>;

    /// Get configuration
    fn config(&self) -> &DataSourceConfig;
}

/// A buffered data source that manages incoming data
#[derive(Clone, Debug, Default)]
pub struct BufferedDataSource {
    /// Internal data buffer
    data: Vec<DataPoint>,
    /// Pending events queue
    events: VecDeque<DataSourceEvent>,
    /// Current state
    state: DataSourceState,
    /// Configuration
    config: DataSourceConfig,
    /// Batch buffer for batching updates
    batch_buffer: Vec<DataPoint>,
    /// Last batch time (for batching)
    last_batch_time: f64,
}

impl BufferedDataSource {
    /// Create a new buffered data source
    pub fn new() -> Self {
        Self::with_config(DataSourceConfig::default())
    }

    /// Create with configuration
    pub fn with_config(config: DataSourceConfig) -> Self {
        Self {
            config,
            state: DataSourceState::Connected, // Start connected for manual sources
            ..Default::default()
        }
    }

    /// Push a single data point
    pub fn push(&mut self, point: DataPoint) {
        self.push_many(vec![point]);
    }

    /// Push multiple data points
    pub fn push_many(&mut self, points: Vec<DataPoint>) {
        if self.config.batch_updates {
            self.batch_buffer.extend(points);
        } else {
            self.append_points(points.clone());
            self.events.push_back(DataSourceEvent::Append(points));
        }
    }

    /// Flush batched updates
    pub fn flush_batch(&mut self, current_time: f64) {
        if !self.batch_buffer.is_empty() {
            let elapsed = (current_time - self.last_batch_time) * 1000.0;
            if elapsed >= self.config.batch_interval_ms as f64 {
                let points = std::mem::take(&mut self.batch_buffer);
                self.append_points(points.clone());
                self.events.push_back(DataSourceEvent::Append(points));
                self.last_batch_time = current_time;
            }
        }
    }

    /// Replace all data
    pub fn replace(&mut self, points: Vec<DataPoint>) {
        self.data = points.clone();
        self.trim_to_max();
        self.events.push_back(DataSourceEvent::Replace(points));
    }

    /// Update data at index
    pub fn update(&mut self, index: usize, points: Vec<DataPoint>) {
        for (i, point) in points.iter().enumerate() {
            if index + i < self.data.len() {
                self.data[index + i] = point.clone();
            }
        }
        self.events
            .push_back(DataSourceEvent::Update { index, points });
    }

    /// Remove data at index
    pub fn remove(&mut self, index: usize, count: usize) {
        let end = (index + count).min(self.data.len());
        self.data.drain(index..end);
        self.events
            .push_back(DataSourceEvent::Remove { index, count });
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.data.clear();
        self.events.push_back(DataSourceEvent::Replace(vec![]));
    }

    /// Get data length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get data slice
    pub fn data(&self) -> &[DataPoint] {
        &self.data
    }

    fn append_points(&mut self, points: Vec<DataPoint>) {
        self.data.extend(points);
        self.trim_to_max();
    }

    fn trim_to_max(&mut self) {
        if self.config.max_points > 0 && self.data.len() > self.config.max_points {
            let excess = self.data.len() - self.config.max_points;
            self.data.drain(0..excess);
        }
    }
}

impl DataSource for BufferedDataSource {
    fn poll(&mut self) -> DataSourceEvent {
        self.events.pop_front().unwrap_or(DataSourceEvent::None)
    }

    fn state(&self) -> DataSourceState {
        self.state
    }

    fn connect(&mut self) {
        self.state = DataSourceState::Connected;
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
        &self.config
    }
}

/// Multi-series data source for charts with multiple datasets
#[derive(Clone, Debug, Default)]
pub struct MultiSeriesDataSource {
    /// Data sources for each series
    series: Vec<BufferedDataSource>,
    /// Series labels
    labels: Vec<String>,
    /// Global state
    state: DataSourceState,
}

impl MultiSeriesDataSource {
    /// Create a new multi-series data source
    pub fn new() -> Self {
        Self {
            state: DataSourceState::Connected,
            ..Default::default()
        }
    }

    /// Add a series
    pub fn add_series(&mut self, label: impl Into<String>) -> usize {
        let index = self.series.len();
        self.series.push(BufferedDataSource::new());
        self.labels.push(label.into());
        index
    }

    /// Add a series with config
    pub fn add_series_with_config(
        &mut self,
        label: impl Into<String>,
        config: DataSourceConfig,
    ) -> usize {
        let index = self.series.len();
        self.series.push(BufferedDataSource::with_config(config));
        self.labels.push(label.into());
        index
    }

    /// Push data to a series
    pub fn push_to_series(&mut self, series_index: usize, point: DataPoint) {
        if let Some(source) = self.series.get_mut(series_index) {
            source.push(point);
        }
    }

    /// Push multiple points to a series
    pub fn push_many_to_series(&mut self, series_index: usize, points: Vec<DataPoint>) {
        if let Some(source) = self.series.get_mut(series_index) {
            source.push_many(points);
        }
    }

    /// Replace data in a series
    pub fn replace_series(&mut self, series_index: usize, points: Vec<DataPoint>) {
        if let Some(source) = self.series.get_mut(series_index) {
            source.replace(points);
        }
    }

    /// Get series count
    pub fn series_count(&self) -> usize {
        self.series.len()
    }

    /// Get series data
    pub fn series_data(&self, index: usize) -> Option<&[DataPoint]> {
        self.series.get(index).map(|s| s.data())
    }

    /// Get series label
    pub fn series_label(&self, index: usize) -> Option<&str> {
        self.labels.get(index).map(|s| s.as_str())
    }

    /// Poll all series for events
    pub fn poll_all(&mut self) -> Vec<(usize, DataSourceEvent)> {
        let mut events = Vec::new();
        for (i, source) in self.series.iter_mut().enumerate() {
            let event = source.poll();
            if !matches!(event, DataSourceEvent::None) {
                events.push((i, event));
            }
        }
        events
    }

    /// Flush all batch buffers
    pub fn flush_all(&mut self, current_time: f64) {
        for source in &mut self.series {
            source.flush_batch(current_time);
        }
    }

    /// Convert to ChartData snapshot
    pub fn to_chart_data(&self) -> ChartData {
        let mut data = ChartData::new();
        for (i, source) in self.series.iter().enumerate() {
            let label = self.labels.get(i).cloned().unwrap_or_default();
            let dataset = Dataset::new(label).with_points(source.snapshot());
            data = data.add_dataset(dataset);
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffered_data_source_push() {
        let mut source = BufferedDataSource::new();
        source.push(DataPoint::from_y(10.0));
        source.push(DataPoint::from_y(20.0));

        assert_eq!(source.len(), 2);
        assert_eq!(source.data()[0].y, 10.0);
        assert_eq!(source.data()[1].y, 20.0);
    }

    #[test]
    fn test_buffered_data_source_poll() {
        let mut source = BufferedDataSource::new();
        source.push(DataPoint::from_y(10.0));

        match source.poll() {
            DataSourceEvent::Append(points) => {
                assert_eq!(points.len(), 1);
                assert_eq!(points[0].y, 10.0);
            }
            _ => panic!("Expected Append event"),
        }

        // No more events
        assert!(matches!(source.poll(), DataSourceEvent::None));
    }

    #[test]
    fn test_buffered_data_source_max_points() {
        let config = DataSourceConfig::default().with_max_points(3);
        let mut source = BufferedDataSource::with_config(config);

        source.push_many(vec![
            DataPoint::from_y(1.0),
            DataPoint::from_y(2.0),
            DataPoint::from_y(3.0),
            DataPoint::from_y(4.0),
            DataPoint::from_y(5.0),
        ]);

        assert_eq!(source.len(), 3);
        assert_eq!(source.data()[0].y, 3.0);
        assert_eq!(source.data()[1].y, 4.0);
        assert_eq!(source.data()[2].y, 5.0);
    }

    #[test]
    fn test_buffered_data_source_replace() {
        let mut source = BufferedDataSource::new();
        source.push(DataPoint::from_y(1.0));
        source.push(DataPoint::from_y(2.0));

        source.replace(vec![DataPoint::from_y(100.0)]);

        assert_eq!(source.len(), 1);
        assert_eq!(source.data()[0].y, 100.0);
    }

    #[test]
    fn test_multi_series_data_source() {
        let mut source = MultiSeriesDataSource::new();
        let s1 = source.add_series("Series A");
        let s2 = source.add_series("Series B");

        source.push_to_series(s1, DataPoint::from_y(10.0));
        source.push_to_series(s2, DataPoint::from_y(20.0));

        assert_eq!(source.series_count(), 2);
        assert_eq!(source.series_data(s1).unwrap()[0].y, 10.0);
        assert_eq!(source.series_data(s2).unwrap()[0].y, 20.0);
    }

    #[test]
    fn test_multi_series_to_chart_data() {
        let mut source = MultiSeriesDataSource::new();
        source.add_series("A");
        source.add_series("B");

        source.push_to_series(0, DataPoint::from_y(10.0));
        source.push_to_series(1, DataPoint::from_y(20.0));

        let chart_data = source.to_chart_data();
        assert_eq!(chart_data.datasets.len(), 2);
        assert_eq!(chart_data.datasets[0].label, "A");
        assert_eq!(chart_data.datasets[1].label, "B");
    }

    #[test]
    fn test_data_source_state() {
        let mut source = BufferedDataSource::new();
        assert_eq!(source.state(), DataSourceState::Connected);

        source.pause();
        assert_eq!(source.state(), DataSourceState::Paused);

        source.resume();
        assert_eq!(source.state(), DataSourceState::Connected);

        source.disconnect();
        assert_eq!(source.state(), DataSourceState::Disconnected);
    }
}
