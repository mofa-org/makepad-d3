//! Data structures and sources for chart data
//!
//! This module provides:
//! - Core data structures ([`DataPoint`], [`Dataset`], [`ChartData`])
//! - Dynamic data sources ([`DataSource`], [`BufferedDataSource`], [`StreamingDataSource`])
//! - Observable datasets with change tracking ([`ObservableDataset`])
//! - Data transformation pipelines ([`DataPipeline`])
//!
//! # Static Data Example
//!
//! ```
//! use makepad_d3::data::{ChartData, Dataset};
//!
//! let data = ChartData::new()
//!     .with_labels(vec!["Jan", "Feb", "Mar"])
//!     .add_dataset(Dataset::new("Revenue").with_data(vec![100.0, 200.0, 150.0]));
//! ```
//!
//! # Dynamic Data Example
//!
//! ```
//! use makepad_d3::data::{ObservableDataset, DataPoint};
//!
//! let mut dataset = ObservableDataset::new("Live Data");
//! dataset.push(DataPoint::from_y(100.0));
//!
//! // Check for changes
//! while let Some(change) = dataset.poll_change() {
//!     println!("Data changed: {:?}", change);
//! }
//! ```
//!
//! # Streaming Data Example
//!
//! ```
//! use makepad_d3::data::{StreamingDataSource, StreamMessage, DataPoint};
//!
//! let (mut source, tx) = StreamingDataSource::new();
//!
//! // Send data from another thread
//! tx.send(StreamMessage::Point(DataPoint::from_y(42.0))).unwrap();
//!
//! // Poll in render loop
//! let event = source.poll();
//! ```

mod chart_data;
mod dataset;
mod observable;
mod pipeline;
mod point;
mod polling;
mod source;
mod streaming;

// Core data structures
pub use chart_data::ChartData;
pub use dataset::{Color, Dataset, PointStyle};
pub use point::DataPoint;

// Data source traits and types
pub use source::{
    BufferedDataSource, DataSource, DataSourceConfig, DataSourceEvent, DataSourceState,
    MultiSeriesDataSource,
};

// Observable dataset
pub use observable::{DataChange, ObservableDataset};

// Streaming data source
pub use streaming::{
    SharedStreamingSource, StreamMessage, StreamingDataSource, StreamingSourceBuilder,
};

// Polling data source
pub use polling::{
    PollingConfig, PollingDataSource, PollingSourceBuilder, PollingState, PollingStrategy,
};

// Data pipeline
pub use pipeline::{Aggregation, DataPipeline, Transform};
