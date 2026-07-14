//! Enhanced tick configuration and generation
//!
//! This module provides additional tick configuration options beyond the basic
//! TickOptions in scale traits, including minor ticks and tick filtering.

use crate::scale::{Scale, Tick, TickOptions};

/// Configuration for tick appearance and behavior
#[derive(Clone, Debug)]
pub struct TickConfig {
    /// Options for generating major ticks from the scale
    pub tick_options: TickOptions,
    /// Number of minor ticks between major ticks (0 = no minor ticks)
    pub minor_count: usize,
    /// Size of minor tick marks
    pub minor_size: f64,
    /// Filter function to selectively show ticks
    filter: Option<TickFilter>,
    /// Custom tick values (overrides scale-generated ticks)
    custom_values: Option<Vec<f64>>,
    /// Custom tick labels (must match custom_values length)
    custom_labels: Option<Vec<String>>,
}

/// Filter for tick visibility
#[derive(Clone)]
pub enum TickFilter {
    /// Show every Nth tick
    EveryN(usize),
    /// Show ticks at specific indices
    AtIndices(Vec<usize>),
    /// Show first and last only
    FirstAndLast,
    /// Show first, last, and middle
    FirstMiddleLast,
    /// Custom filter function
    Custom(std::sync::Arc<dyn Fn(&Tick, usize) -> bool + Send + Sync>),
}

impl std::fmt::Debug for TickFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EveryN(n) => write!(f, "EveryN({})", n),
            Self::AtIndices(indices) => write!(f, "AtIndices({:?})", indices),
            Self::FirstAndLast => write!(f, "FirstAndLast"),
            Self::FirstMiddleLast => write!(f, "FirstMiddleLast"),
            Self::Custom(_) => write!(f, "Custom(<fn>)"),
        }
    }
}

impl TickFilter {
    /// Check if a tick should be shown
    pub fn should_show(&self, tick: &Tick, index: usize, total: usize) -> bool {
        match self {
            Self::EveryN(n) => index % n == 0,
            Self::AtIndices(indices) => indices.contains(&index),
            Self::FirstAndLast => index == 0 || index == total.saturating_sub(1),
            Self::FirstMiddleLast => {
                index == 0 || index == total.saturating_sub(1) || index == total / 2
            }
            Self::Custom(f) => f(tick, index),
        }
    }
}

impl Default for TickConfig {
    fn default() -> Self {
        Self {
            tick_options: TickOptions::default(),
            minor_count: 0,
            minor_size: 3.0,
            filter: None,
            custom_values: None,
            custom_labels: None,
        }
    }
}

impl TickConfig {
    /// Create a new tick configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of minor ticks between major ticks
    pub fn with_minor_ticks(mut self, count: usize) -> Self {
        self.minor_count = count;
        self
    }

    /// Set the size of minor tick marks
    pub fn with_minor_size(mut self, size: f64) -> Self {
        self.minor_size = size;
        self
    }

    /// Set a filter to show only certain ticks
    pub fn with_filter(mut self, filter: TickFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Show every Nth tick
    pub fn every_n(mut self, n: usize) -> Self {
        self.filter = Some(TickFilter::EveryN(n));
        self
    }

    /// Show only first and last ticks
    pub fn first_and_last(mut self) -> Self {
        self.filter = Some(TickFilter::FirstAndLast);
        self
    }

    /// Set custom tick values
    pub fn with_values(mut self, values: Vec<f64>) -> Self {
        self.custom_values = Some(values);
        self
    }

    /// Set custom tick values with labels
    pub fn with_values_and_labels(mut self, values: Vec<f64>, labels: Vec<String>) -> Self {
        self.custom_values = Some(values);
        self.custom_labels = Some(labels);
        self
    }

    /// Set tick count
    pub fn with_count(mut self, count: usize) -> Self {
        self.tick_options.count = count;
        self
    }

    /// Include domain bounds as ticks
    pub fn with_bounds(mut self, include: bool) -> Self {
        self.tick_options.include_bounds = include;
        self
    }

    /// Get the tick options
    pub fn tick_options(&self) -> &TickOptions {
        &self.tick_options
    }

    /// Check if minor ticks are enabled
    pub fn has_minor_ticks(&self) -> bool {
        self.minor_count > 0
    }

    /// Get minor tick size
    pub fn minor_size(&self) -> f64 {
        self.minor_size
    }
}

/// A minor tick mark (between major ticks)
#[derive(Clone, Debug)]
pub struct MinorTick {
    /// Position in pixels
    pub position: f64,
    /// Value in domain
    pub value: f64,
}

/// Result of tick generation
#[derive(Clone, Debug)]
pub struct TickResult {
    /// Major ticks
    pub major: Vec<Tick>,
    /// Minor ticks
    pub minor: Vec<MinorTick>,
}

impl TickResult {
    /// Create empty tick result
    pub fn empty() -> Self {
        Self {
            major: Vec::new(),
            minor: Vec::new(),
        }
    }
}

/// Generate ticks from a scale with the given configuration
pub fn generate_ticks<S: Scale>(scale: &S, config: &TickConfig) -> TickResult {
    // Generate major ticks
    let mut major = if let Some(ref values) = config.custom_values {
        // Use custom values
        let labels = config.custom_labels.as_ref();
        values
            .iter()
            .enumerate()
            .map(|(i, &value)| {
                let position = scale.scale(value);
                let label = labels
                    .and_then(|l| l.get(i))
                    .cloned()
                    .unwrap_or_else(|| format!("{}", value));
                Tick {
                    value,
                    label,
                    position,
                }
            })
            .collect()
    } else {
        // Use scale-generated ticks
        scale.ticks(&config.tick_options)
    };

    // Apply filter if set
    if let Some(ref filter) = config.filter {
        let total = major.len();
        major = major
            .into_iter()
            .enumerate()
            .filter(|(i, tick)| filter.should_show(tick, *i, total))
            .map(|(_, tick)| tick)
            .collect();
    }

    // Generate minor ticks if enabled
    let minor = if config.minor_count > 0 && major.len() >= 2 {
        generate_minor_ticks(&major, config.minor_count, scale)
    } else {
        Vec::new()
    };

    TickResult { major, minor }
}

/// Generate minor ticks between major ticks
fn generate_minor_ticks<S: Scale>(major: &[Tick], count: usize, scale: &S) -> Vec<MinorTick> {
    if major.len() < 2 || count == 0 {
        return Vec::new();
    }

    let mut minor = Vec::new();
    let divisions = count + 1;

    for i in 0..major.len() - 1 {
        let start_value = major[i].value;
        let end_value = major[i + 1].value;
        let step = (end_value - start_value) / divisions as f64;

        for j in 1..=count {
            let value = start_value + step * j as f64;
            let position = scale.scale(value);
            minor.push(MinorTick { position, value });
        }
    }

    minor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scale::{LinearScale, ScaleExt};

    #[test]
    fn test_tick_config_default() {
        let config = TickConfig::default();
        assert_eq!(config.minor_count, 0);
        assert_eq!(config.minor_size, 3.0);
    }

    #[test]
    fn test_tick_config_builder() {
        let config = TickConfig::new()
            .with_minor_ticks(4)
            .with_minor_size(2.0)
            .with_count(5);

        assert_eq!(config.minor_count, 4);
        assert_eq!(config.minor_size, 2.0);
        assert_eq!(config.tick_options.count, 5);
    }

    #[test]
    fn test_generate_ticks_basic() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::default();
        let result = generate_ticks(&scale, &config);

        assert!(!result.major.is_empty());
        assert!(result.minor.is_empty()); // No minor ticks by default
    }

    #[test]
    fn test_generate_ticks_with_minor() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().with_minor_ticks(4);
        let result = generate_ticks(&scale, &config);

        assert!(!result.major.is_empty());
        // Should have minor ticks between each pair of major ticks
        if result.major.len() >= 2 {
            assert!(!result.minor.is_empty());
            // (major - 1) gaps * 4 minor ticks each
            assert_eq!(result.minor.len(), (result.major.len() - 1) * 4);
        }
    }

    #[test]
    fn test_tick_filter_every_n() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().every_n(2);
        let result = generate_ticks(&scale, &config);

        // Every other tick should be shown
        let all_ticks = scale.ticks(&TickOptions::default());
        let expected_count = (all_ticks.len() + 1) / 2;
        assert_eq!(result.major.len(), expected_count);
    }

    #[test]
    fn test_tick_filter_first_and_last() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().first_and_last();
        let result = generate_ticks(&scale, &config);

        assert_eq!(result.major.len(), 2);
    }

    #[test]
    fn test_custom_tick_values() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().with_values(vec![0.0, 25.0, 50.0, 75.0, 100.0]);
        let result = generate_ticks(&scale, &config);

        assert_eq!(result.major.len(), 5);
        assert_eq!(result.major[0].value, 0.0);
        assert_eq!(result.major[2].value, 50.0);
        assert_eq!(result.major[4].value, 100.0);
    }

    #[test]
    fn test_custom_tick_values_with_labels() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().with_values_and_labels(
            vec![0.0, 50.0, 100.0],
            vec!["Low".to_string(), "Medium".to_string(), "High".to_string()],
        );
        let result = generate_ticks(&scale, &config);

        assert_eq!(result.major.len(), 3);
        assert_eq!(result.major[0].label, "Low");
        assert_eq!(result.major[1].label, "Medium");
        assert_eq!(result.major[2].label, "High");
    }

    #[test]
    fn test_tick_filter_custom() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        // Only show ticks with value >= 50
        let config =
            TickConfig::new().with_filter(TickFilter::Custom(std::sync::Arc::new(|tick, _| {
                tick.value >= 50.0
            })));
        let result = generate_ticks(&scale, &config);

        for tick in &result.major {
            assert!(tick.value >= 50.0);
        }
    }

    #[test]
    fn test_tick_filter_at_indices() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 500.0);

        let config = TickConfig::new().with_filter(TickFilter::AtIndices(vec![0, 2, 4]));
        let result = generate_ticks(&scale, &config);

        assert!(result.major.len() <= 3);
    }

    #[test]
    fn test_minor_tick_positions() {
        let scale = LinearScale::new()
            .with_domain(0.0, 100.0)
            .with_range(0.0, 100.0);

        let config = TickConfig::new()
            .with_values(vec![0.0, 50.0, 100.0])
            .with_minor_ticks(1);

        let result = generate_ticks(&scale, &config);

        assert_eq!(result.minor.len(), 2);
        assert!((result.minor[0].value - 25.0).abs() < 0.001);
        assert!((result.minor[1].value - 75.0).abs() < 0.001);
    }
}
