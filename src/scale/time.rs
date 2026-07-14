//! Time scale implementation

use super::traits::{ContinuousScale, Scale, Tick, TickOptions};
use chrono::{DateTime, Datelike, Duration, Timelike, Utc};

/// Time interval for tick generation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeInterval {
    /// Milliseconds with multiplier
    Millisecond(u32),
    /// Seconds with multiplier
    Second(u32),
    /// Minutes with multiplier
    Minute(u32),
    /// Hours with multiplier
    Hour(u32),
    /// Days with multiplier
    Day(u32),
    /// Weeks with multiplier
    Week(u32),
    /// Months with multiplier
    Month(u32),
    /// Years with multiplier
    Year(u32),
}

impl TimeInterval {
    /// Get the approximate duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        match self {
            Self::Millisecond(n) => *n as f64,
            Self::Second(n) => *n as f64 * 1000.0,
            Self::Minute(n) => *n as f64 * 60_000.0,
            Self::Hour(n) => *n as f64 * 3_600_000.0,
            Self::Day(n) => *n as f64 * 86_400_000.0,
            Self::Week(n) => *n as f64 * 604_800_000.0,
            Self::Month(n) => *n as f64 * 2_592_000_000.0, // ~30 days
            Self::Year(n) => *n as f64 * 31_536_000_000.0, // 365 days
        }
    }

    /// Get a reasonable default format string for this interval
    pub fn default_format(&self) -> &'static str {
        match self {
            Self::Millisecond(_) => "%H:%M:%S.%3f",
            Self::Second(_) => "%H:%M:%S",
            Self::Minute(_) | Self::Hour(_) => "%H:%M",
            Self::Day(_) | Self::Week(_) => "%b %d",
            Self::Month(_) => "%b %Y",
            Self::Year(_) => "%Y",
        }
    }

    /// Find the appropriate interval for a given duration and target tick count
    pub fn for_duration(duration_ms: f64, target_ticks: usize) -> Self {
        if target_ticks == 0 {
            return Self::Year(1);
        }

        let target_interval = duration_ms / target_ticks as f64;

        // Intervals in ascending order of duration
        let intervals: &[(f64, TimeInterval)] = &[
            (1.0, Self::Millisecond(1)),
            (5.0, Self::Millisecond(5)),
            (10.0, Self::Millisecond(10)),
            (50.0, Self::Millisecond(50)),
            (100.0, Self::Millisecond(100)),
            (500.0, Self::Millisecond(500)),
            (1000.0, Self::Second(1)),
            (5000.0, Self::Second(5)),
            (15000.0, Self::Second(15)),
            (30000.0, Self::Second(30)),
            (60000.0, Self::Minute(1)),
            (300000.0, Self::Minute(5)),
            (900000.0, Self::Minute(15)),
            (1800000.0, Self::Minute(30)),
            (3600000.0, Self::Hour(1)),
            (10800000.0, Self::Hour(3)),
            (21600000.0, Self::Hour(6)),
            (43200000.0, Self::Hour(12)),
            (86400000.0, Self::Day(1)),
            (172800000.0, Self::Day(2)),
            (604800000.0, Self::Week(1)),
            (1209600000.0, Self::Week(2)),
            (2592000000.0, Self::Month(1)),
            (7776000000.0, Self::Month(3)),
            (15552000000.0, Self::Month(6)),
            (31536000000.0, Self::Year(1)),
            (63072000000.0, Self::Year(2)),
            (157680000000.0, Self::Year(5)),
            (315360000000.0, Self::Year(10)),
        ];

        intervals
            .iter()
            .find(|(ms, _)| *ms >= target_interval)
            .map(|(_, interval)| *interval)
            .unwrap_or(Self::Year(10))
    }
}

/// A tick mark with time information
#[derive(Clone, Debug)]
pub struct TimeTick {
    /// The DateTime value
    pub time: DateTime<Utc>,
    /// The value as milliseconds since epoch
    pub value: f64,
    /// The formatted label
    pub label: String,
    /// Position in pixels
    pub position: f64,
}

/// Scale for date/time data
///
/// Maps dates to pixel positions with calendar-aware tick generation.
///
/// # Example
/// ```
/// use makepad_d3::scale::{Scale, TimeScale};
/// use chrono::{Utc, TimeZone};
///
/// let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
/// let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
///
/// let scale = TimeScale::new()
///     .with_time_domain(start, end)
///     .with_range(0.0, 1000.0);
///
/// // Mid-year maps to approximately mid-range
/// let mid = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();
/// let pos = scale.scale_time(mid);
/// assert!(pos > 400.0 && pos < 600.0);
/// ```
#[derive(Clone, Debug)]
pub struct TimeScale {
    domain_start: DateTime<Utc>,
    domain_end: DateTime<Utc>,
    range_start: f64,
    range_end: f64,
    clamp: bool,
    format: Option<String>,
}

impl TimeScale {
    /// Create a new time scale with current time as domain
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            domain_start: now,
            domain_end: now + Duration::hours(24),
            range_start: 0.0,
            range_end: 100.0,
            clamp: false,
            format: None,
        }
    }

    /// Set the time domain
    pub fn with_time_domain(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.domain_start = start;
        self.domain_end = end;
        self
    }

    /// Set the time domain from timestamps (milliseconds since epoch)
    pub fn with_domain_ms(mut self, start_ms: i64, end_ms: i64) -> Self {
        self.domain_start = DateTime::from_timestamp_millis(start_ms).unwrap_or_else(Utc::now);
        self.domain_end = DateTime::from_timestamp_millis(end_ms).unwrap_or_else(Utc::now);
        self
    }

    /// Set the range
    pub fn with_range(mut self, start: f64, end: f64) -> Self {
        self.range_start = start;
        self.range_end = end;
        self
    }

    /// Enable clamping
    pub fn with_clamp(mut self, clamp: bool) -> Self {
        self.clamp = clamp;
        self
    }

    /// Set custom tick format
    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Get domain start
    pub fn domain_start(&self) -> DateTime<Utc> {
        self.domain_start
    }

    /// Get domain end
    pub fn domain_end(&self) -> DateTime<Utc> {
        self.domain_end
    }

    /// Get duration in milliseconds
    pub fn duration_ms(&self) -> f64 {
        (self.domain_end - self.domain_start).num_milliseconds() as f64
    }

    /// Scale a DateTime to pixel position
    pub fn scale_time(&self, time: DateTime<Utc>) -> f64 {
        let time = if self.clamp {
            if time < self.domain_start.min(self.domain_end) {
                self.domain_start.min(self.domain_end)
            } else if time > self.domain_start.max(self.domain_end) {
                self.domain_start.max(self.domain_end)
            } else {
                time
            }
        } else {
            time
        };

        let t = self.normalize_time(time);
        self.range_start + t * (self.range_end - self.range_start)
    }

    /// Invert pixel position to DateTime
    pub fn invert_time(&self, pixel: f64) -> DateTime<Utc> {
        let range_span = self.range_end - self.range_start;
        if range_span.abs() < f64::EPSILON {
            return self.domain_start;
        }

        let t = (pixel - self.range_start) / range_span;
        let duration = self.domain_end - self.domain_start;
        let offset_ms = (duration.num_milliseconds() as f64 * t) as i64;

        self.domain_start + Duration::milliseconds(offset_ms)
    }

    /// Normalize time to [0, 1]
    fn normalize_time(&self, time: DateTime<Utc>) -> f64 {
        let duration = self.domain_end - self.domain_start;
        if duration.num_milliseconds() == 0 {
            return 0.5;
        }

        let offset = time - self.domain_start;
        offset.num_milliseconds() as f64 / duration.num_milliseconds() as f64
    }

    /// Generate time ticks
    pub fn time_ticks(&self, options: &TickOptions) -> Vec<TimeTick> {
        let duration_ms = self.duration_ms().abs();
        if duration_ms < 1.0 {
            return vec![];
        }

        let interval = TimeInterval::for_duration(duration_ms, options.count);
        let format = self
            .format
            .as_deref()
            .unwrap_or_else(|| interval.default_format());

        let mut ticks = Vec::new();
        let mut current = self.floor_to_interval(self.domain_start, interval);

        let (domain_min, domain_max) = if self.domain_start <= self.domain_end {
            (self.domain_start, self.domain_end)
        } else {
            (self.domain_end, self.domain_start)
        };

        while current <= domain_max && ticks.len() < options.max_count {
            if current >= domain_min {
                let pos = self.scale_time(current);
                let label = current.format(format).to_string();
                ticks.push(TimeTick {
                    time: current,
                    value: current.timestamp_millis() as f64,
                    label,
                    position: pos,
                });
            }
            current = self.add_interval(current, interval);
        }

        ticks
    }

    /// Floor datetime to interval boundary
    fn floor_to_interval(&self, time: DateTime<Utc>, interval: TimeInterval) -> DateTime<Utc> {
        match interval {
            TimeInterval::Millisecond(n) => {
                let ms = time.timestamp_subsec_millis();
                let floored = (ms / n) * n;
                time - Duration::milliseconds((ms - floored) as i64)
            }
            TimeInterval::Second(n) => {
                let s = time.second();
                let floored = (s / n) * n;
                time.with_second(floored)
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Minute(n) => {
                let m = time.minute();
                let floored = (m / n) * n;
                time.with_minute(floored)
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Hour(n) => {
                let h = time.hour();
                let floored = (h / n) * n;
                time.with_hour(floored)
                    .and_then(|t| t.with_minute(0))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Day(n) => {
                let d = time.day();
                let floored = ((d - 1) / n) * n + 1;
                time.with_day(floored)
                    .and_then(|t| t.with_hour(0))
                    .and_then(|t| t.with_minute(0))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Week(_) => {
                // Floor to start of week (Monday)
                let weekday = time.weekday().num_days_from_monday();
                (time - Duration::days(weekday as i64))
                    .with_hour(0)
                    .and_then(|t| t.with_minute(0))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Month(n) => {
                let m = time.month();
                let floored = ((m - 1) / n) * n + 1;
                time.with_month(floored)
                    .and_then(|t| t.with_day(1))
                    .and_then(|t| t.with_hour(0))
                    .and_then(|t| t.with_minute(0))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
            TimeInterval::Year(n) => {
                let y = time.year();
                let floored = (y / n as i32) * n as i32;
                time.with_year(floored)
                    .and_then(|t| t.with_month(1))
                    .and_then(|t| t.with_day(1))
                    .and_then(|t| t.with_hour(0))
                    .and_then(|t| t.with_minute(0))
                    .and_then(|t| t.with_second(0))
                    .and_then(|t| t.with_nanosecond(0))
                    .unwrap_or(time)
            }
        }
    }

    /// Add interval to datetime
    fn add_interval(&self, time: DateTime<Utc>, interval: TimeInterval) -> DateTime<Utc> {
        match interval {
            TimeInterval::Millisecond(n) => time + Duration::milliseconds(n as i64),
            TimeInterval::Second(n) => time + Duration::seconds(n as i64),
            TimeInterval::Minute(n) => time + Duration::minutes(n as i64),
            TimeInterval::Hour(n) => time + Duration::hours(n as i64),
            TimeInterval::Day(n) => time + Duration::days(n as i64),
            TimeInterval::Week(n) => time + Duration::weeks(n as i64),
            TimeInterval::Month(n) => {
                // Handle month addition carefully
                let mut new_month = time.month() + n;
                let mut new_year = time.year();
                while new_month > 12 {
                    new_month -= 12;
                    new_year += 1;
                }
                time.with_year(new_year)
                    .and_then(|t| t.with_month(new_month))
                    .unwrap_or(time + Duration::days(30 * n as i64))
            }
            TimeInterval::Year(n) => time
                .with_year(time.year() + n as i32)
                .unwrap_or(time + Duration::days(365 * n as i64)),
        }
    }
}

impl Default for TimeScale {
    fn default() -> Self {
        Self::new()
    }
}

impl Scale for TimeScale {
    fn scale_type(&self) -> &'static str {
        "time"
    }

    fn set_domain(&mut self, min: f64, max: f64) {
        // Interpret as milliseconds since epoch
        self.domain_start = DateTime::from_timestamp_millis(min as i64).unwrap_or_else(Utc::now);
        self.domain_end = DateTime::from_timestamp_millis(max as i64).unwrap_or_else(Utc::now);
    }

    fn set_range(&mut self, start: f64, end: f64) {
        self.range_start = start;
        self.range_end = end;
    }

    fn domain(&self) -> (f64, f64) {
        (
            self.domain_start.timestamp_millis() as f64,
            self.domain_end.timestamp_millis() as f64,
        )
    }

    fn range(&self) -> (f64, f64) {
        (self.range_start, self.range_end)
    }

    fn scale(&self, value: f64) -> f64 {
        // Value is milliseconds since epoch
        let time = DateTime::from_timestamp_millis(value as i64).unwrap_or(self.domain_start);
        self.scale_time(time)
    }

    fn invert(&self, pixel: f64) -> f64 {
        self.invert_time(pixel).timestamp_millis() as f64
    }

    fn ticks(&self, options: &TickOptions) -> Vec<Tick> {
        self.time_ticks(options)
            .into_iter()
            .map(|tt| Tick {
                value: tt.value,
                label: tt.label,
                position: tt.position,
            })
            .collect()
    }

    fn copy_from(&mut self, other: &Self) {
        self.domain_start = other.domain_start;
        self.domain_end = other.domain_end;
        self.range_start = other.range_start;
        self.range_end = other.range_end;
        self.clamp = other.clamp;
        self.format = other.format.clone();
    }

    fn clone_box(&self) -> Box<dyn Scale> {
        Box::new(self.clone())
    }
}

impl ContinuousScale for TimeScale {
    fn nice(&mut self) {
        let interval = TimeInterval::for_duration(self.duration_ms().abs(), 10);
        self.domain_start = self.floor_to_interval(self.domain_start, interval);
        self.domain_end =
            self.add_interval(self.floor_to_interval(self.domain_end, interval), interval);
    }

    fn is_clamped(&self) -> bool {
        self.clamp
    }

    fn set_clamp(&mut self, clamp: bool) {
        self.clamp = clamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_time_scale_new() {
        let scale = TimeScale::new();
        assert!(scale.duration_ms() > 0.0);
    }

    #[test]
    fn test_basic_time_scale() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        assert!((scale.scale_time(start) - 0.0).abs() < 1.0);
        assert!((scale.scale_time(end) - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_mid_year() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        let mid = Utc.with_ymd_and_hms(2024, 7, 1, 0, 0, 0).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let pos = scale.scale_time(mid);
        assert!(pos > 400.0 && pos < 600.0);
    }

    #[test]
    fn test_invert() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let inverted = scale.invert_time(500.0);
        assert!(inverted > start);
        assert!(inverted < end);
    }

    #[test]
    fn test_interval_selection() {
        // 1 hour duration -> should use minute intervals
        let hour_ms = 3_600_000.0;
        let interval = TimeInterval::for_duration(hour_ms, 10);
        assert!(matches!(interval, TimeInterval::Minute(_)));

        // 1 year duration -> should use month intervals
        let year_ms = 31_536_000_000.0;
        let interval = TimeInterval::for_duration(year_ms, 12);
        assert!(matches!(interval, TimeInterval::Month(_)));
    }

    #[test]
    fn test_tick_generation() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let ticks = scale.time_ticks(&TickOptions::new().with_count(12));

        assert!(!ticks.is_empty());
        assert!(ticks.len() <= 15);

        // Ticks should be in order
        for i in 1..ticks.len() {
            assert!(ticks[i].time > ticks[i - 1].time);
        }
    }

    #[test]
    fn test_hourly_ticks() {
        let start = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 6, 15, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let ticks = scale.time_ticks(&TickOptions::new().with_count(12));

        assert!(ticks.len() >= 6);
        assert!(ticks.len() <= 24);
    }

    #[test]
    fn test_scale_trait_methods() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let start_ms = start.timestamp_millis() as f64;
        let end_ms = end.timestamp_millis() as f64;

        assert!((scale.scale(start_ms) - 0.0).abs() < 1.0);
        assert!((scale.scale(end_ms) - 1000.0).abs() < 1.0);
    }

    #[test]
    fn test_clone_box() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0);

        let boxed: Box<dyn Scale> = scale.clone_box();
        assert_eq!(boxed.scale_type(), "time");
    }

    #[test]
    fn test_clamp() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        let before = Utc.with_ymd_and_hms(2023, 6, 1, 0, 0, 0).unwrap();

        let scale = TimeScale::new()
            .with_time_domain(start, end)
            .with_range(0.0, 1000.0)
            .with_clamp(true);

        assert!((scale.scale_time(before) - 0.0).abs() < 1.0);
    }

    #[test]
    fn test_interval_duration() {
        assert!((TimeInterval::Second(1).duration_ms() - 1000.0).abs() < 0.1);
        assert!((TimeInterval::Minute(1).duration_ms() - 60000.0).abs() < 0.1);
        assert!((TimeInterval::Hour(1).duration_ms() - 3600000.0).abs() < 0.1);
    }
}
