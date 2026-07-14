//! Number and time formatting utilities for axis labels
//!
//! Provides flexible formatting options for numeric and time values displayed on axes.

use chrono::{DateTime, Datelike, Timelike, Utc};
use std::sync::Arc;

/// Format specifier for numeric axis labels
#[derive(Clone)]
pub enum NumberFormat {
    /// Automatically select appropriate format based on value
    Auto,
    /// Fixed number of decimal places
    Fixed(usize),
    /// Significant digits (precision)
    Precision(usize),
    /// Percentage format (multiply by 100, add %)
    Percent,
    /// SI prefix format (k, M, G, etc.)
    SI,
    /// Currency format with prefix and decimal places
    Currency {
        /// Currency symbol (e.g., "$", "€")
        prefix: String,
        /// Number of decimal places
        decimals: usize,
    },
    /// Custom formatter function
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync>),
}

impl std::fmt::Debug for NumberFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Fixed(d) => write!(f, "Fixed({})", d),
            Self::Precision(p) => write!(f, "Precision({})", p),
            Self::Percent => write!(f, "Percent"),
            Self::SI => write!(f, "SI"),
            Self::Currency { prefix, decimals } => {
                write!(
                    f,
                    "Currency {{ prefix: {:?}, decimals: {} }}",
                    prefix, decimals
                )
            }
            Self::Custom(_) => write!(f, "Custom(<fn>)"),
        }
    }
}

impl Default for NumberFormat {
    fn default() -> Self {
        Self::Auto
    }
}

impl NumberFormat {
    /// Format a numeric value according to this format specifier
    pub fn format(&self, value: f64) -> String {
        match self {
            Self::Auto => format_auto(value),
            Self::Fixed(d) => format!("{:.1$}", value, *d),
            Self::Precision(p) => format_precision(value, *p),
            Self::Percent => format_percent(value),
            Self::SI => format_si(value),
            Self::Currency { prefix, decimals } => {
                format!("{}{:.*}", prefix, *decimals, value)
            }
            Self::Custom(f) => f(value),
        }
    }

    /// Create a fixed decimal places format
    pub fn fixed(decimals: usize) -> Self {
        Self::Fixed(decimals)
    }

    /// Create a precision (significant digits) format
    pub fn precision(sig_figs: usize) -> Self {
        Self::Precision(sig_figs)
    }

    /// Create a currency format
    pub fn currency(prefix: impl Into<String>, decimals: usize) -> Self {
        Self::Currency {
            prefix: prefix.into(),
            decimals,
        }
    }

    /// Create a custom formatter
    pub fn custom<F>(f: F) -> Self
    where
        F: Fn(f64) -> String + Send + Sync + 'static,
    {
        Self::Custom(Arc::new(f))
    }
}

/// Automatically format a number based on its magnitude
fn format_auto(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    let abs_val = value.abs();

    if abs_val == 0.0 {
        return "0".to_string();
    }

    // Very large or very small numbers use scientific notation
    if abs_val >= 1e9 || abs_val < 1e-4 {
        return format_si(value);
    }

    // Determine appropriate decimal places
    if abs_val >= 100.0 {
        format!("{:.0}", value)
    } else if abs_val >= 10.0 {
        format!("{:.1}", value)
    } else if abs_val >= 1.0 {
        format!("{:.2}", value)
    } else if abs_val >= 0.1 {
        format!("{:.3}", value)
    } else {
        format!("{:.4}", value)
    }
}

/// Format with significant digits
fn format_precision(value: f64, precision: usize) -> String {
    if !value.is_finite() || value == 0.0 {
        return value.to_string();
    }

    let precision = precision.max(1);
    let abs_val = value.abs();
    let magnitude = abs_val.log10().floor() as i32;
    let decimals = (precision as i32 - 1 - magnitude).max(0) as usize;

    format!("{:.1$}", value, decimals)
}

/// Format as percentage
fn format_percent(value: f64) -> String {
    let pct = value * 100.0;
    if pct.abs() >= 10.0 {
        format!("{:.0}%", pct)
    } else if pct.abs() >= 1.0 {
        format!("{:.1}%", pct)
    } else {
        format!("{:.2}%", pct)
    }
}

/// Format with SI prefixes
pub fn format_si(value: f64) -> String {
    if !value.is_finite() {
        return value.to_string();
    }

    if value == 0.0 {
        return "0".to_string();
    }

    let abs_val = value.abs();

    let (scaled, suffix) = if abs_val >= 1e12 {
        (value / 1e12, "T")
    } else if abs_val >= 1e9 {
        (value / 1e9, "G")
    } else if abs_val >= 1e6 {
        (value / 1e6, "M")
    } else if abs_val >= 1e3 {
        (value / 1e3, "k")
    } else if abs_val >= 1.0 {
        (value, "")
    } else if abs_val >= 1e-3 {
        (value * 1e3, "m")
    } else if abs_val >= 1e-6 {
        (value * 1e6, "μ")
    } else if abs_val >= 1e-9 {
        (value * 1e9, "n")
    } else {
        (value * 1e12, "p")
    };

    // Format the scaled value appropriately
    let formatted = if scaled.abs() >= 100.0 {
        format!("{:.0}", scaled)
    } else if scaled.abs() >= 10.0 {
        format!("{:.1}", scaled)
    } else {
        format!("{:.2}", scaled)
    };

    format!("{}{}", formatted, suffix)
}

/// Time duration formatter
#[derive(Clone, Debug, Default)]
pub struct DurationFormat {
    /// Show hours even if zero
    pub always_hours: bool,
    /// Show seconds even if zero
    pub always_seconds: bool,
    /// Show milliseconds
    pub show_millis: bool,
}

impl DurationFormat {
    /// Create a new duration formatter
    pub fn new() -> Self {
        Self::default()
    }

    /// Always show hours
    pub fn with_hours(mut self) -> Self {
        self.always_hours = true;
        self
    }

    /// Show milliseconds
    pub fn with_millis(mut self) -> Self {
        self.show_millis = true;
        self
    }

    /// Format a duration in seconds
    pub fn format(&self, seconds: f64) -> String {
        let total_seconds = seconds.abs();
        let sign = if seconds < 0.0 { "-" } else { "" };

        let hours = (total_seconds / 3600.0).floor() as u64;
        let minutes = ((total_seconds % 3600.0) / 60.0).floor() as u64;
        let secs = (total_seconds % 60.0).floor() as u64;
        let millis = ((total_seconds % 1.0) * 1000.0).round() as u64;

        if self.show_millis {
            if hours > 0 || self.always_hours {
                format!("{}{}:{:02}:{:02}.{:03}", sign, hours, minutes, secs, millis)
            } else {
                format!("{}{}:{:02}.{:03}", sign, minutes, secs, millis)
            }
        } else if hours > 0 || self.always_hours {
            format!("{}{}:{:02}:{:02}", sign, hours, minutes, secs)
        } else {
            format!("{}{}:{:02}", sign, minutes, secs)
        }
    }
}

/// Time format specifier for date/time axis labels
#[derive(Clone)]
pub enum TimeFormat {
    /// Automatically select format based on time interval
    Auto,
    /// Full date and time: "2024-01-15 14:30:00"
    Full,
    /// Date only: "2024-01-15"
    Date,
    /// Time only: "14:30:00"
    Time,
    /// Year only: "2024"
    Year,
    /// Month and year: "Jan 2024"
    MonthYear,
    /// Month and day: "Jan 15"
    MonthDay,
    /// Day of month: "15"
    Day,
    /// Hour and minute: "14:30"
    HourMinute,
    /// Custom format string (chrono format)
    Custom(String),
    /// Custom formatter function
    CustomFn(Arc<dyn Fn(DateTime<Utc>) -> String + Send + Sync>),
}

impl std::fmt::Debug for TimeFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "Auto"),
            Self::Full => write!(f, "Full"),
            Self::Date => write!(f, "Date"),
            Self::Time => write!(f, "Time"),
            Self::Year => write!(f, "Year"),
            Self::MonthYear => write!(f, "MonthYear"),
            Self::MonthDay => write!(f, "MonthDay"),
            Self::Day => write!(f, "Day"),
            Self::HourMinute => write!(f, "HourMinute"),
            Self::Custom(s) => write!(f, "Custom({:?})", s),
            Self::CustomFn(_) => write!(f, "CustomFn(<fn>)"),
        }
    }
}

impl Default for TimeFormat {
    fn default() -> Self {
        Self::Auto
    }
}

impl TimeFormat {
    /// Format a DateTime value
    pub fn format(&self, dt: DateTime<Utc>) -> String {
        match self {
            Self::Auto => format_time_auto(dt),
            Self::Full => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            Self::Date => dt.format("%Y-%m-%d").to_string(),
            Self::Time => dt.format("%H:%M:%S").to_string(),
            Self::Year => dt.format("%Y").to_string(),
            Self::MonthYear => dt.format("%b %Y").to_string(),
            Self::MonthDay => dt.format("%b %d").to_string(),
            Self::Day => dt.format("%d").to_string(),
            Self::HourMinute => dt.format("%H:%M").to_string(),
            Self::Custom(fmt) => dt.format(fmt).to_string(),
            Self::CustomFn(f) => f(dt),
        }
    }

    /// Format a timestamp (milliseconds since Unix epoch)
    pub fn format_timestamp(&self, timestamp_ms: f64) -> String {
        if let Some(dt) = timestamp_from_ms(timestamp_ms) {
            self.format(dt)
        } else {
            "Invalid".to_string()
        }
    }

    /// Create a custom format with a format string
    pub fn custom(format_str: impl Into<String>) -> Self {
        Self::Custom(format_str.into())
    }

    /// Create a custom formatter function
    pub fn custom_fn<F>(f: F) -> Self
    where
        F: Fn(DateTime<Utc>) -> String + Send + Sync + 'static,
    {
        Self::CustomFn(Arc::new(f))
    }
}

/// Convert milliseconds timestamp to DateTime
pub fn timestamp_from_ms(ms: f64) -> Option<DateTime<Utc>> {
    let secs = (ms / 1000.0).floor() as i64;
    let nanos = ((ms % 1000.0) * 1_000_000.0) as u32;
    DateTime::from_timestamp(secs, nanos)
}

/// Convert DateTime to milliseconds timestamp
pub fn timestamp_to_ms(dt: DateTime<Utc>) -> f64 {
    dt.timestamp() as f64 * 1000.0 + dt.timestamp_subsec_millis() as f64
}

/// Automatically format a DateTime based on what components are interesting
fn format_time_auto(dt: DateTime<Utc>) -> String {
    // Check what components are at their "boundary" values
    let at_year_start = dt.month() == 1 && dt.day() == 1;
    let at_month_start = dt.day() == 1;
    let at_midnight = dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0;
    let at_hour_start = dt.minute() == 0 && dt.second() == 0;
    let has_subsec = dt.timestamp_subsec_millis() > 0;

    if has_subsec {
        // Sub-second precision
        dt.format("%H:%M:%S.%3f").to_string()
    } else if !at_midnight {
        // Time of day matters
        if at_hour_start {
            dt.format("%H:%M").to_string()
        } else {
            dt.format("%H:%M:%S").to_string()
        }
    } else if at_year_start {
        // Start of year
        dt.format("%Y").to_string()
    } else if at_month_start {
        // Start of month
        dt.format("%b %Y").to_string()
    } else {
        // Day level
        dt.format("%b %d").to_string()
    }
}

/// Time format based on interval for multi-scale formatting
#[derive(Clone, Debug)]
pub struct MultiScaleTimeFormat {
    /// Format for sub-second intervals
    pub millisecond: String,
    /// Format for second intervals
    pub second: String,
    /// Format for minute intervals
    pub minute: String,
    /// Format for hour intervals
    pub hour: String,
    /// Format for day intervals
    pub day: String,
    /// Format for week intervals
    pub week: String,
    /// Format for month intervals
    pub month: String,
    /// Format for year intervals
    pub year: String,
}

impl Default for MultiScaleTimeFormat {
    fn default() -> Self {
        Self {
            millisecond: "%H:%M:%S.%3f".to_string(),
            second: "%H:%M:%S".to_string(),
            minute: "%H:%M".to_string(),
            hour: "%H:%M".to_string(),
            day: "%b %d".to_string(),
            week: "%b %d".to_string(),
            month: "%b %Y".to_string(),
            year: "%Y".to_string(),
        }
    }
}

impl MultiScaleTimeFormat {
    /// Create a new multi-scale format with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Format a datetime based on the interval duration (in milliseconds)
    pub fn format(&self, dt: DateTime<Utc>, interval_ms: f64) -> String {
        let format_str = self.get_format_for_interval(interval_ms);
        dt.format(&format_str).to_string()
    }

    /// Get the format string for a given interval
    pub fn get_format_for_interval(&self, interval_ms: f64) -> &str {
        const SECOND: f64 = 1000.0;
        const MINUTE: f64 = 60.0 * SECOND;
        const HOUR: f64 = 60.0 * MINUTE;
        const DAY: f64 = 24.0 * HOUR;
        const WEEK: f64 = 7.0 * DAY;
        const MONTH: f64 = 30.0 * DAY;
        const YEAR: f64 = 365.0 * DAY;

        if interval_ms < SECOND {
            &self.millisecond
        } else if interval_ms < MINUTE {
            &self.second
        } else if interval_ms < HOUR {
            &self.minute
        } else if interval_ms < DAY {
            &self.hour
        } else if interval_ms < WEEK {
            &self.day
        } else if interval_ms < MONTH {
            &self.week
        } else if interval_ms < YEAR {
            &self.month
        } else {
            &self.year
        }
    }

    /// Set millisecond format
    pub fn with_millisecond(mut self, fmt: impl Into<String>) -> Self {
        self.millisecond = fmt.into();
        self
    }

    /// Set second format
    pub fn with_second(mut self, fmt: impl Into<String>) -> Self {
        self.second = fmt.into();
        self
    }

    /// Set minute format
    pub fn with_minute(mut self, fmt: impl Into<String>) -> Self {
        self.minute = fmt.into();
        self
    }

    /// Set hour format
    pub fn with_hour(mut self, fmt: impl Into<String>) -> Self {
        self.hour = fmt.into();
        self
    }

    /// Set day format
    pub fn with_day(mut self, fmt: impl Into<String>) -> Self {
        self.day = fmt.into();
        self
    }

    /// Set week format
    pub fn with_week(mut self, fmt: impl Into<String>) -> Self {
        self.week = fmt.into();
        self
    }

    /// Set month format
    pub fn with_month(mut self, fmt: impl Into<String>) -> Self {
        self.month = fmt.into();
        self
    }

    /// Set year format
    pub fn with_year(mut self, fmt: impl Into<String>) -> Self {
        self.year = fmt.into();
        self
    }
}

/// Relative time format (e.g., "2 hours ago", "in 3 days")
pub fn format_relative(dt: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let diff = now.signed_duration_since(dt);
    let total_seconds = diff.num_seconds();

    let (value, unit, is_past) = if total_seconds.abs() < 60 {
        (total_seconds.abs(), "second", total_seconds >= 0)
    } else if total_seconds.abs() < 3600 {
        (diff.num_minutes().abs(), "minute", total_seconds >= 0)
    } else if total_seconds.abs() < 86400 {
        (diff.num_hours().abs(), "hour", total_seconds >= 0)
    } else if total_seconds.abs() < 604800 {
        (diff.num_days().abs(), "day", total_seconds >= 0)
    } else if total_seconds.abs() < 2592000 {
        (diff.num_weeks().abs(), "week", total_seconds >= 0)
    } else if total_seconds.abs() < 31536000 {
        ((diff.num_days().abs() / 30), "month", total_seconds >= 0)
    } else {
        ((diff.num_days().abs() / 365), "year", total_seconds >= 0)
    };

    let plural = if value == 1 { "" } else { "s" };

    if is_past {
        format!("{} {}{} ago", value, unit, plural)
    } else {
        format!("in {} {}{}", value, unit, plural)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_format() {
        assert_eq!(NumberFormat::Auto.format(0.0), "0");
        assert_eq!(NumberFormat::Auto.format(1234.0), "1234");
        assert_eq!(NumberFormat::Auto.format(12.34), "12.3");
        assert_eq!(NumberFormat::Auto.format(1.234), "1.23");
        assert_eq!(NumberFormat::Auto.format(0.1234), "0.123");
    }

    #[test]
    fn test_fixed_format() {
        assert_eq!(NumberFormat::Fixed(2).format(1.2345), "1.23");
        assert_eq!(NumberFormat::Fixed(0).format(1.9), "2");
        assert_eq!(NumberFormat::Fixed(4).format(1.0), "1.0000");
    }

    #[test]
    fn test_precision_format() {
        assert_eq!(NumberFormat::Precision(3).format(1234.0), "1234");
        assert_eq!(NumberFormat::Precision(3).format(12.34), "12.3");
        assert_eq!(NumberFormat::Precision(3).format(0.001234), "0.00123");
    }

    #[test]
    fn test_percent_format() {
        assert_eq!(NumberFormat::Percent.format(0.5), "50%");
        assert_eq!(NumberFormat::Percent.format(0.123), "12%"); // >= 10% shows 0 decimals
        assert_eq!(NumberFormat::Percent.format(0.056), "5.6%"); // 1-10% shows 1 decimal
        assert_eq!(NumberFormat::Percent.format(0.0056), "0.56%"); // < 1% shows 2 decimals
    }

    #[test]
    fn test_si_format() {
        assert_eq!(format_si(1000.0), "1.00k");
        assert_eq!(format_si(1500.0), "1.50k");
        assert_eq!(format_si(1_000_000.0), "1.00M");
        assert_eq!(format_si(1_500_000_000.0), "1.50G");
        assert_eq!(format_si(0.001), "1.00m");
        assert_eq!(format_si(0.000001), "1.00μ");
    }

    #[test]
    fn test_currency_format() {
        let fmt = NumberFormat::currency("$", 2);
        assert_eq!(fmt.format(1234.56), "$1234.56");
        assert_eq!(fmt.format(0.5), "$0.50");
    }

    #[test]
    fn test_custom_format() {
        let fmt = NumberFormat::custom(|v| format!("Value: {:.1}", v));
        assert_eq!(fmt.format(42.567), "Value: 42.6");
    }

    #[test]
    fn test_duration_format() {
        let fmt = DurationFormat::new();
        assert_eq!(fmt.format(65.0), "1:05");
        assert_eq!(fmt.format(3665.0), "1:01:05");

        let fmt_hours = DurationFormat::new().with_hours();
        assert_eq!(fmt_hours.format(65.0), "0:01:05");

        let fmt_millis = DurationFormat::new().with_millis();
        assert_eq!(fmt_millis.format(65.123), "1:05.123");
    }

    #[test]
    fn test_format_edge_cases() {
        assert_eq!(NumberFormat::Auto.format(f64::INFINITY), "inf");
        assert_eq!(NumberFormat::Auto.format(f64::NEG_INFINITY), "-inf");
        assert!(NumberFormat::Auto.format(f64::NAN).contains("NaN"));
    }

    // TimeFormat tests

    #[test]
    fn test_time_format_basic() {
        // 2024-01-15 14:30:45 UTC
        let dt = DateTime::from_timestamp(1705329045, 0).unwrap();

        assert_eq!(TimeFormat::Date.format(dt), "2024-01-15");
        assert_eq!(TimeFormat::Time.format(dt), "14:30:45");
        assert_eq!(TimeFormat::Year.format(dt), "2024");
        assert_eq!(TimeFormat::HourMinute.format(dt), "14:30");
    }

    #[test]
    fn test_time_format_full() {
        let dt = DateTime::from_timestamp(1705329045, 0).unwrap();
        assert_eq!(TimeFormat::Full.format(dt), "2024-01-15 14:30:45");
    }

    #[test]
    fn test_time_format_custom() {
        let dt = DateTime::from_timestamp(1705329045, 0).unwrap();
        let fmt = TimeFormat::custom("%Y/%m/%d");
        assert_eq!(fmt.format(dt), "2024/01/15");
    }

    #[test]
    fn test_timestamp_conversion() {
        let ms = 1705329045000.0; // 2024-01-15 14:30:45 UTC
        let dt = timestamp_from_ms(ms).unwrap();
        assert_eq!(dt.year(), 2024);
        assert_eq!(dt.month(), 1);
        assert_eq!(dt.day(), 15);

        // Round trip
        let back = timestamp_to_ms(dt);
        assert!((back - ms).abs() < 1.0);
    }

    #[test]
    fn test_time_format_auto() {
        // Test at year boundary (2024-01-01 00:00:00)
        let year_start = DateTime::from_timestamp(1704067200, 0).unwrap();
        assert_eq!(TimeFormat::Auto.format(year_start), "2024");

        // Test at month boundary (2024-02-01 00:00:00)
        let month_start = DateTime::from_timestamp(1706745600, 0).unwrap();
        assert_eq!(TimeFormat::Auto.format(month_start), "Feb 2024");

        // Test with time (2024-01-15 14:00:00)
        let with_time = DateTime::from_timestamp(1705327200, 0).unwrap();
        assert_eq!(TimeFormat::Auto.format(with_time), "14:00");
    }

    #[test]
    fn test_multi_scale_time_format() {
        let fmt = MultiScaleTimeFormat::new();
        let dt = DateTime::from_timestamp(1705329045, 0).unwrap();

        // Millisecond interval
        assert!(fmt.format(dt, 500.0).contains("."));

        // Hour interval
        let hour_fmt = fmt.format(dt, 3600.0 * 1000.0);
        assert!(hour_fmt.contains(":"));

        // Year interval
        let year_fmt = fmt.format(dt, 365.0 * 24.0 * 3600.0 * 1000.0);
        assert_eq!(year_fmt, "2024");
    }

    #[test]
    fn test_multi_scale_time_format_builder() {
        let fmt = MultiScaleTimeFormat::new()
            .with_year("%Y CE")
            .with_month("%B %Y");

        assert_eq!(fmt.year, "%Y CE");
        assert_eq!(fmt.month, "%B %Y");
    }

    #[test]
    fn test_relative_time_format() {
        let now = DateTime::from_timestamp(1705329045, 0).unwrap();

        // 1 hour ago
        let one_hour_ago = DateTime::from_timestamp(1705329045 - 3600, 0).unwrap();
        assert_eq!(format_relative(one_hour_ago, now), "1 hour ago");

        // 2 days ago
        let two_days_ago = DateTime::from_timestamp(1705329045 - 2 * 86400, 0).unwrap();
        assert_eq!(format_relative(two_days_ago, now), "2 days ago");

        // In the future
        let one_hour_future = DateTime::from_timestamp(1705329045 + 3600, 0).unwrap();
        assert_eq!(format_relative(one_hour_future, now), "in 1 hour");
    }

    #[test]
    fn test_time_format_timestamp() {
        let ms = 1705329045000.0;
        let fmt = TimeFormat::Year;
        assert_eq!(fmt.format_timestamp(ms), "2024");
    }
}
