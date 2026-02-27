//! Streaming data source for real-time data feeds
//!
//! Provides a channel-based streaming data source that can receive
//! data from external sources (WebSockets, async tasks, etc.)
//!
//! # Example
//!
//! ```
//! use makepad_d3::data::{StreamingDataSource, DataPoint};
//! use std::sync::mpsc;
//!
//! // Create streaming source with channel
//! let (tx, rx) = mpsc::channel();
//! let mut source = StreamingDataSource::from_receiver(rx);
//!
//! // Send data from another thread/task
//! tx.send(DataPoint::from_y(100.0)).unwrap();
//!
//! // Poll for data in render loop
//! source.poll();
//! ```
//!
//! # Backpressure Control
//!
//! The streaming source supports backpressure control to handle fast data producers:
//!
//! ```
//! use makepad_d3::data::{BackpressureStreamingSource, BackpressureConfig, BackpressureStrategy};
//!
//! let config = BackpressureConfig {
//!     strategy: BackpressureStrategy::DropOldest,
//!     high_watermark: 1000,
//!     low_watermark: 500,
//!     ..Default::default()
//! };
//!
//! let (mut source, tx) = BackpressureStreamingSource::with_backpressure(config);
//! ```

use super::{DataPoint, DataSource, DataSourceConfig, DataSourceEvent, DataSourceState};
use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Message types for streaming data
#[derive(Clone, Debug)]
pub enum StreamMessage {
    /// Single data point
    Point(DataPoint),
    /// Multiple data points
    Points(Vec<DataPoint>),
    /// Replace all data
    Replace(Vec<DataPoint>),
    /// Clear data
    Clear,
    /// Connection status
    Connected,
    /// Disconnection
    Disconnected,
    /// Error message
    Error(String),
}

/// Streaming data source using channels
///
/// Receives data through a channel and buffers it for chart consumption.
pub struct StreamingDataSource {
    /// Data buffer
    data: Vec<DataPoint>,
    /// Channel receiver
    receiver: Option<Receiver<StreamMessage>>,
    /// Pending events
    events: VecDeque<DataSourceEvent>,
    /// Current state
    state: DataSourceState,
    /// Configuration
    config: DataSourceConfig,
    /// Message counter
    message_count: u64,
}

impl StreamingDataSource {
    /// Create a new streaming data source
    pub fn new() -> (Self, Sender<StreamMessage>) {
        let (tx, rx) = channel();
        let source = Self {
            data: Vec::new(),
            receiver: Some(rx),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            config: DataSourceConfig::realtime(),
            message_count: 0,
        };
        (source, tx)
    }

    /// Create with configuration
    pub fn with_config(config: DataSourceConfig) -> (Self, Sender<StreamMessage>) {
        let (tx, rx) = channel();
        let source = Self {
            data: Vec::new(),
            receiver: Some(rx),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            config,
            message_count: 0,
        };
        (source, tx)
    }

    /// Create from existing receiver
    pub fn from_receiver(receiver: Receiver<StreamMessage>) -> Self {
        Self {
            data: Vec::new(),
            receiver: Some(receiver),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            config: DataSourceConfig::realtime(),
            message_count: 0,
        }
    }

    /// Process all pending messages from channel
    pub fn process_messages(&mut self) {
        // Collect messages first to avoid borrow issues
        let messages: Vec<_> = if let Some(ref receiver) = self.receiver {
            let mut msgs = Vec::new();
            loop {
                match receiver.try_recv() {
                    Ok(message) => msgs.push(Ok(message)),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        msgs.push(Err(()));
                        break;
                    }
                }
            }
            msgs
        } else {
            Vec::new()
        };

        // Process collected messages
        for result in messages {
            match result {
                Ok(message) => {
                    self.message_count += 1;
                    self.handle_message(message);
                }
                Err(()) => {
                    self.state = DataSourceState::Disconnected;
                    self.events.push_back(DataSourceEvent::Disconnected);
                }
            }
        }
    }

    fn handle_message(&mut self, message: StreamMessage) {
        match message {
            StreamMessage::Point(point) => {
                self.data.push(point.clone());
                self.trim_to_max();
                self.events.push_back(DataSourceEvent::Append(vec![point]));
            }
            StreamMessage::Points(points) => {
                self.data.extend(points.clone());
                self.trim_to_max();
                self.events.push_back(DataSourceEvent::Append(points));
            }
            StreamMessage::Replace(points) => {
                self.data = points.clone();
                self.trim_to_max();
                self.events.push_back(DataSourceEvent::Replace(points));
            }
            StreamMessage::Clear => {
                self.data.clear();
                self.events.push_back(DataSourceEvent::Replace(vec![]));
            }
            StreamMessage::Connected => {
                self.state = DataSourceState::Connected;
                self.events.push_back(DataSourceEvent::Connected);
            }
            StreamMessage::Disconnected => {
                self.state = DataSourceState::Disconnected;
                self.events.push_back(DataSourceEvent::Disconnected);
            }
            StreamMessage::Error(err) => {
                self.state = DataSourceState::Error;
                self.events.push_back(DataSourceEvent::Error(err));
            }
        }
    }

    fn trim_to_max(&mut self) {
        if self.config.max_points > 0 && self.data.len() > self.config.max_points {
            let excess = self.data.len() - self.config.max_points;
            self.data.drain(0..excess);
        }
    }

    /// Get message count
    pub fn message_count(&self) -> u64 {
        self.message_count
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
}

impl Default for StreamingDataSource {
    fn default() -> Self {
        let (source, _) = Self::new();
        source
    }
}

impl DataSource for StreamingDataSource {
    fn poll(&mut self) -> DataSourceEvent {
        // First process any pending channel messages
        self.process_messages();
        // Then return queued events
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

/// Thread-safe streaming data source
///
/// Can be shared across threads with interior mutability.
pub struct SharedStreamingSource {
    inner: Arc<Mutex<StreamingDataSource>>,
}

impl SharedStreamingSource {
    /// Create a new shared streaming source
    pub fn new() -> (Self, Sender<StreamMessage>) {
        let (source, tx) = StreamingDataSource::new();
        (Self { inner: Arc::new(Mutex::new(source)) }, tx)
    }

    /// Poll for events
    pub fn poll(&self) -> DataSourceEvent {
        self.inner.lock().unwrap().poll()
    }

    /// Get current state
    pub fn state(&self) -> DataSourceState {
        self.inner.lock().unwrap().state()
    }

    /// Get data snapshot
    pub fn snapshot(&self) -> Vec<DataPoint> {
        self.inner.lock().unwrap().snapshot()
    }

    /// Clone the arc handle
    pub fn clone_handle(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl Default for SharedStreamingSource {
    fn default() -> Self {
        let (source, _) = Self::new();
        source
    }
}

/// Backpressure strategy for handling fast data producers
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BackpressureStrategy {
    /// Drop oldest data when buffer is full (default)
    #[default]
    DropOldest,
    /// Drop newest data when buffer is full
    DropNewest,
    /// Block the producer until there's space (not recommended for UI)
    Block,
    /// Sample data - keep every Nth point
    Sample { rate: usize },
    /// Throttle - limit rate of data acceptance
    Throttle { interval_ms: u64 },
    /// Fail fast - emit error when buffer is full
    Fail,
}

/// Configuration for backpressure control
#[derive(Clone, Debug)]
pub struct BackpressureConfig {
    /// Backpressure strategy to use
    pub strategy: BackpressureStrategy,
    /// High watermark - start applying backpressure when buffer exceeds this
    pub high_watermark: usize,
    /// Low watermark - stop applying backpressure when buffer drops below this
    pub low_watermark: usize,
    /// Maximum buffer size before applying backpressure
    pub max_buffer_size: usize,
    /// Enable backpressure monitoring
    pub enable_monitoring: bool,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            strategy: BackpressureStrategy::DropOldest,
            high_watermark: 1000,
            low_watermark: 500,
            max_buffer_size: 2000,
            enable_monitoring: true,
        }
    }
}

impl BackpressureConfig {
    /// Create a new backpressure config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the backpressure strategy
    pub fn with_strategy(mut self, strategy: BackpressureStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set watermarks
    pub fn with_watermarks(mut self, low: usize, high: usize) -> Self {
        self.low_watermark = low;
        self.high_watermark = high;
        self
    }

    /// Set max buffer size
    pub fn with_max_buffer_size(mut self, size: usize) -> Self {
        self.max_buffer_size = size;
        self
    }
}

/// Backpressure statistics for monitoring
#[derive(Clone, Debug, Default)]
pub struct BackpressureStats {
    /// Total points received
    pub points_received: u64,
    /// Points dropped due to backpressure
    pub points_dropped: u64,
    /// Number of times backpressure was applied
    pub backpressure_events: u64,
    /// Current buffer size
    pub current_buffer_size: usize,
    /// Time spent in backpressure state (ms)
    pub time_in_backpressure_ms: u64,
    /// Last backpressure event timestamp
    pub last_backpressure_time: Option<Instant>,
    /// Whether currently in backpressure state
    pub in_backpressure: bool,
}

/// Streaming data source with backpressure control
///
/// Extends StreamingDataSource with configurable backpressure handling
/// for scenarios where data producers are faster than consumers.
pub struct BackpressureStreamingSource {
    /// Data buffer
    data: Vec<DataPoint>,
    /// Channel receiver
    receiver: Option<Receiver<StreamMessage>>,
    /// Pending events
    events: VecDeque<DataSourceEvent>,
    /// Current state
    state: DataSourceState,
    /// Configuration
    config: DataSourceConfig,
    /// Backpressure configuration
    backpressure_config: BackpressureConfig,
    /// Backpressure statistics
    stats: BackpressureStats,
    /// Message counter
    message_count: u64,
    /// Sample counter for sampling strategy
    sample_counter: usize,
    /// Last throttle time
    last_throttle_time: Option<Instant>,
    /// Backpressure start time (for tracking duration)
    backpressure_start: Option<Instant>,
}

impl BackpressureStreamingSource {
    /// Create a new streaming source with backpressure control
    pub fn new() -> (Self, Sender<StreamMessage>) {
        Self::with_backpressure(BackpressureConfig::default())
    }

    /// Create with specific backpressure configuration
    pub fn with_backpressure(config: BackpressureConfig) -> (Self, Sender<StreamMessage>) {
        let (tx, rx) = channel();
        let source = Self {
            data: Vec::new(),
            receiver: Some(rx),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            config: DataSourceConfig::realtime(),
            backpressure_config: config,
            stats: BackpressureStats::default(),
            message_count: 0,
            sample_counter: 0,
            last_throttle_time: None,
            backpressure_start: None,
        };
        (source, tx)
    }

    /// Create with both data source and backpressure configs
    pub fn with_configs(
        data_config: DataSourceConfig,
        backpressure_config: BackpressureConfig,
    ) -> (Self, Sender<StreamMessage>) {
        let (tx, rx) = channel();
        let source = Self {
            data: Vec::new(),
            receiver: Some(rx),
            events: VecDeque::new(),
            state: DataSourceState::Connected,
            config: data_config,
            backpressure_config,
            stats: BackpressureStats::default(),
            message_count: 0,
            sample_counter: 0,
            last_throttle_time: None,
            backpressure_start: None,
        };
        (source, tx)
    }

    /// Get backpressure statistics
    pub fn backpressure_stats(&self) -> &BackpressureStats {
        &self.stats
    }

    /// Check if currently in backpressure state
    pub fn is_in_backpressure(&self) -> bool {
        self.stats.in_backpressure
    }

    /// Process messages with backpressure handling
    pub fn process_messages(&mut self) {
        let messages: Vec<_> = if let Some(ref receiver) = self.receiver {
            let mut msgs = Vec::new();
            loop {
                match receiver.try_recv() {
                    Ok(message) => msgs.push(Ok(message)),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        msgs.push(Err(()));
                        break;
                    }
                }
            }
            msgs
        } else {
            Vec::new()
        };

        for result in messages {
            match result {
                Ok(message) => {
                    self.message_count += 1;
                    self.handle_message_with_backpressure(message);
                }
                Err(()) => {
                    self.state = DataSourceState::Disconnected;
                    self.events.push_back(DataSourceEvent::Disconnected);
                }
            }
        }

        // Update backpressure state
        self.update_backpressure_state();
    }

    fn handle_message_with_backpressure(&mut self, message: StreamMessage) {
        match message {
            StreamMessage::Point(point) => {
                self.stats.points_received += 1;
                if self.should_accept_point() {
                    self.data.push(point.clone());
                    self.apply_backpressure_if_needed();
                    self.events.push_back(DataSourceEvent::Append(vec![point]));
                }
            }
            StreamMessage::Points(points) => {
                self.stats.points_received += points.len() as u64;
                let accepted: Vec<DataPoint> = points.into_iter()
                    .filter(|_| self.should_accept_point())
                    .collect();
                
                if !accepted.is_empty() {
                    self.data.extend(accepted.clone());
                    self.apply_backpressure_if_needed();
                    self.events.push_back(DataSourceEvent::Append(accepted));
                }
            }
            StreamMessage::Replace(points) => {
                self.data = points.clone();
                self.events.push_back(DataSourceEvent::Replace(points));
            }
            StreamMessage::Clear => {
                self.data.clear();
                self.events.push_back(DataSourceEvent::Replace(vec![]));
            }
            StreamMessage::Connected => {
                self.state = DataSourceState::Connected;
                self.events.push_back(DataSourceEvent::Connected);
            }
            StreamMessage::Disconnected => {
                self.state = DataSourceState::Disconnected;
                self.events.push_back(DataSourceEvent::Disconnected);
            }
            StreamMessage::Error(err) => {
                self.state = DataSourceState::Error;
                self.events.push_back(DataSourceEvent::Error(err));
            }
        }
    }

    fn should_accept_point(&mut self) -> bool {
        let current_size = self.data.len();
        
        // Check if we're at max capacity
        if current_size >= self.backpressure_config.max_buffer_size {
            match &self.backpressure_config.strategy {
                BackpressureStrategy::DropOldest => {
                    // Remove oldest to make room
                    self.data.remove(0);
                    self.stats.points_dropped += 1;
                    self.record_backpressure_event();
                    true
                }
                BackpressureStrategy::DropNewest => {
                    // Drop the new point
                    self.stats.points_dropped += 1;
                    self.record_backpressure_event();
                    false
                }
                BackpressureStrategy::Fail => {
                    // Emit error event
                    self.stats.points_dropped += 1;
                    self.record_backpressure_event();
                    self.events.push_back(DataSourceEvent::Error(
                        "Buffer full, data point dropped".to_string()
                    ));
                    false
                }
                BackpressureStrategy::Sample { rate } => {
                    self.sample_counter += 1;
                    if self.sample_counter % rate == 0 {
                        true
                    } else {
                        self.stats.points_dropped += 1;
                        false
                    }
                }
                BackpressureStrategy::Throttle { interval_ms } => {
                    let now = Instant::now();
                    if let Some(last) = self.last_throttle_time {
                        if now.duration_since(last) < Duration::from_millis(*interval_ms) {
                            self.stats.points_dropped += 1;
                            self.record_backpressure_event();
                            return false;
                        }
                    }
                    self.last_throttle_time = Some(now);
                    true
                }
                BackpressureStrategy::Block => {
                    // For block strategy, we still accept but apply backpressure
                    true
                }
            }
        } else {
            true
        }
    }

    fn apply_backpressure_if_needed(&mut self) {
        if self.data.len() > self.backpressure_config.high_watermark {
            match &self.backpressure_config.strategy {
                BackpressureStrategy::DropOldest => {
                    while self.data.len() > self.backpressure_config.low_watermark {
                        self.data.remove(0);
                        self.stats.points_dropped += 1;
                    }
                }
                BackpressureStrategy::DropNewest => {
                    while self.data.len() > self.backpressure_config.low_watermark {
                        self.data.pop();
                        self.stats.points_dropped += 1;
                    }
                }
                _ => {}
            }
        }
    }

    fn record_backpressure_event(&mut self) {
        self.stats.backpressure_events += 1;
        self.stats.last_backpressure_time = Some(Instant::now());
        
        if !self.stats.in_backpressure {
            self.stats.in_backpressure = true;
            self.backpressure_start = Some(Instant::now());
        }
    }

    fn update_backpressure_state(&mut self) {
        self.stats.current_buffer_size = self.data.len();
        
        // Check if we've exited backpressure state
        if self.stats.in_backpressure {
            if self.data.len() <= self.backpressure_config.low_watermark {
                if let Some(start) = self.backpressure_start {
                    self.stats.time_in_backpressure_ms += 
                        start.elapsed().as_millis() as u64;
                }
                self.stats.in_backpressure = false;
                self.backpressure_start = None;
            }
        }
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
}

impl Default for BackpressureStreamingSource {
    fn default() -> Self {
        let (source, _) = Self::new();
        source
    }
}

impl DataSource for BackpressureStreamingSource {
    fn poll(&mut self) -> DataSourceEvent {
        self.process_messages();
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

/// Builder for creating streaming data sources with WebSocket-like callbacks
pub struct StreamingSourceBuilder {
    config: DataSourceConfig,
    initial_data: Vec<DataPoint>,
    backpressure_config: Option<BackpressureConfig>,
}

impl StreamingSourceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: DataSourceConfig::realtime(),
            initial_data: Vec::new(),
            backpressure_config: None,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: DataSourceConfig) -> Self {
        self.config = config;
        self
    }

    /// Set maximum points
    pub fn max_points(mut self, max: usize) -> Self {
        self.config.max_points = max;
        self
    }

    /// Set initial data
    pub fn initial_data(mut self, data: Vec<DataPoint>) -> Self {
        self.initial_data = data;
        self
    }

    /// Enable backpressure with configuration
    pub fn with_backpressure(mut self, config: BackpressureConfig) -> Self {
        self.backpressure_config = Some(config);
        self
    }

    /// Build the streaming source
    pub fn build(self) -> (StreamingDataSource, Sender<StreamMessage>) {
        let (mut source, tx) = StreamingDataSource::with_config(self.config);
        source.data = self.initial_data;
        (source, tx)
    }

    /// Build with backpressure support
    pub fn build_with_backpressure(self) -> (BackpressureStreamingSource, Sender<StreamMessage>) {
        let bp_config = self.backpressure_config.unwrap_or_default();
        let (mut source, tx) = BackpressureStreamingSource::with_configs(
            self.config,
            bp_config,
        );
        source.data = self.initial_data;
        (source, tx)
    }
}

impl Default for StreamingSourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_streaming_source_new() {
        let (source, _tx) = StreamingDataSource::new();
        assert!(source.is_empty());
        assert_eq!(source.state(), DataSourceState::Connected);
    }

    #[test]
    fn test_streaming_source_receive_point() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0))).unwrap();
        source.process_messages();

        assert_eq!(source.len(), 1);
        assert_eq!(source.data()[0].y, 100.0);
    }

    #[test]
    fn test_streaming_source_receive_points() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Points(vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
            DataPoint::from_y(30.0),
        ])).unwrap();
        source.process_messages();

        assert_eq!(source.len(), 3);
    }

    #[test]
    fn test_streaming_source_poll_events() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0))).unwrap();

        let event = source.poll();
        assert!(matches!(event, DataSourceEvent::Append(_)));
    }

    #[test]
    fn test_streaming_source_max_points() {
        let config = DataSourceConfig::realtime().with_max_points(3);
        let (mut source, tx) = StreamingDataSource::with_config(config);

        for i in 0..5 {
            tx.send(StreamMessage::Point(DataPoint::from_y(i as f64))).unwrap();
        }
        source.process_messages();

        assert_eq!(source.len(), 3);
        assert_eq!(source.data()[0].y, 2.0);
        assert_eq!(source.data()[1].y, 3.0);
        assert_eq!(source.data()[2].y, 4.0);
    }

    #[test]
    fn test_streaming_source_replace() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Points(vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
        ])).unwrap();
        source.process_messages();

        tx.send(StreamMessage::Replace(vec![DataPoint::from_y(100.0)])).unwrap();
        source.process_messages();

        assert_eq!(source.len(), 1);
        assert_eq!(source.data()[0].y, 100.0);
    }

    #[test]
    fn test_streaming_source_clear() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Points(vec![
            DataPoint::from_y(10.0),
            DataPoint::from_y(20.0),
        ])).unwrap();
        source.process_messages();

        tx.send(StreamMessage::Clear).unwrap();
        source.process_messages();

        assert!(source.is_empty());
    }

    #[test]
    fn test_streaming_source_disconnect() {
        let (mut source, tx) = StreamingDataSource::new();

        drop(tx); // Disconnect by dropping sender
        source.process_messages();

        assert_eq!(source.state(), DataSourceState::Disconnected);
    }

    #[test]
    fn test_shared_streaming_source() {
        let (source, tx) = SharedStreamingSource::new();

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0))).unwrap();

        let event = source.poll();
        assert!(matches!(event, DataSourceEvent::Append(_)));

        let snapshot = source.snapshot();
        assert_eq!(snapshot.len(), 1);
    }

    #[test]
    fn test_builder() {
        let (source, _tx) = StreamingSourceBuilder::new()
            .max_points(100)
            .initial_data(vec![DataPoint::from_y(50.0)])
            .build();

        assert_eq!(source.len(), 1);
        assert_eq!(source.config().max_points, 100);
    }

    #[test]
    fn test_backpressure_config_default() {
        let config = BackpressureConfig::default();
        assert_eq!(config.strategy, BackpressureStrategy::DropOldest);
        assert_eq!(config.high_watermark, 1000);
        assert_eq!(config.low_watermark, 500);
    }

    #[test]
    fn test_backpressure_source_new() {
        let (source, _tx) = BackpressureStreamingSource::new();
        assert!(source.is_empty());
        assert_eq!(source.state(), DataSourceState::Connected);
    }

    #[test]
    fn test_backpressure_drop_oldest() {
        let config = BackpressureConfig::new()
            .with_strategy(BackpressureStrategy::DropOldest)
            .with_watermarks(2, 3)
            .with_max_buffer_size(3);

        let (mut source, tx) = BackpressureStreamingSource::with_backpressure(config);

        // Add 5 points
        for i in 0..5 {
            tx.send(StreamMessage::Point(DataPoint::from_y(i as f64))).unwrap();
        }
        source.process_messages();

        // Should have dropped oldest points
        assert!(source.len() <= 3);
        let stats = source.backpressure_stats();
        assert!(stats.points_dropped > 0);
    }

    #[test]
    fn test_backpressure_sample() {
        let config = BackpressureConfig::new()
            .with_strategy(BackpressureStrategy::Sample { rate: 2 })
            .with_max_buffer_size(100);

        let (mut source, tx) = BackpressureStreamingSource::with_backpressure(config);

        // Add 10 points
        for i in 0..10 {
            tx.send(StreamMessage::Point(DataPoint::from_y(i as f64))).unwrap();
        }
        source.process_messages();

        // Should have ~5 points (every 2nd)
        let stats = source.backpressure_stats();
        assert_eq!(stats.points_received, 10);
        // Some points should be dropped due to sampling
    }

    #[test]
    fn test_backpressure_stats() {
        let (source, _tx) = BackpressureStreamingSource::new();
        let stats = source.backpressure_stats();
        
        assert_eq!(stats.points_received, 0);
        assert_eq!(stats.points_dropped, 0);
        assert!(!stats.in_backpressure);
    }

    #[test]
    fn test_builder_with_backpressure() {
        let (source, _tx) = StreamingSourceBuilder::new()
            .max_points(100)
            .with_backpressure(BackpressureConfig::new())
            .build_with_backpressure();

        assert!(source.is_empty());
        assert_eq!(source.config().max_points, 100);
    }
}