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

use super::{DataPoint, DataSource, DataSourceConfig, DataSourceEvent, DataSourceState};
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

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
        (
            Self {
                inner: Arc::new(Mutex::new(source)),
            },
            tx,
        )
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
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for SharedStreamingSource {
    fn default() -> Self {
        let (source, _) = Self::new();
        source
    }
}

/// Builder for creating streaming data sources with WebSocket-like callbacks
pub struct StreamingSourceBuilder {
    config: DataSourceConfig,
    initial_data: Vec<DataPoint>,
}

impl StreamingSourceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: DataSourceConfig::realtime(),
            initial_data: Vec::new(),
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

    /// Build the streaming source
    pub fn build(self) -> (StreamingDataSource, Sender<StreamMessage>) {
        let (mut source, tx) = StreamingDataSource::with_config(self.config);
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

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0)))
            .unwrap();
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
        ]))
        .unwrap();
        source.process_messages();

        assert_eq!(source.len(), 3);
    }

    #[test]
    fn test_streaming_source_poll_events() {
        let (mut source, tx) = StreamingDataSource::new();

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0)))
            .unwrap();

        let event = source.poll();
        assert!(matches!(event, DataSourceEvent::Append(_)));
    }

    #[test]
    fn test_streaming_source_max_points() {
        let config = DataSourceConfig::realtime().with_max_points(3);
        let (mut source, tx) = StreamingDataSource::with_config(config);

        for i in 0..5 {
            tx.send(StreamMessage::Point(DataPoint::from_y(i as f64)))
                .unwrap();
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
        ]))
        .unwrap();
        source.process_messages();

        tx.send(StreamMessage::Replace(vec![DataPoint::from_y(100.0)]))
            .unwrap();
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
        ]))
        .unwrap();
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

        tx.send(StreamMessage::Point(DataPoint::from_y(100.0)))
            .unwrap();

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
}
