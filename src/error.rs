//! Error types for makepad-d3
//!
//! This module provides error types used throughout the library.

use thiserror::Error;

/// Errors that can occur in makepad-d3
#[derive(Error, Debug, Clone, PartialEq)]
pub enum D3Error {
    /// Domain configuration is invalid
    #[error("Invalid domain: {message}")]
    InvalidDomain { message: String },

    /// Range configuration is invalid
    #[error("Invalid range: {message}")]
    InvalidRange { message: String },

    /// Value is outside expected bounds
    #[error("Value out of bounds: {value} not in [{min}, {max}]")]
    OutOfBounds { value: f64, min: f64, max: f64 },

    /// Data validation failed
    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    /// Parsing failed
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

/// Result type alias for makepad-d3
pub type D3Result<T> = Result<T, D3Error>;

impl D3Error {
    /// Create an invalid domain error
    pub fn invalid_domain(msg: impl Into<String>) -> Self {
        Self::InvalidDomain {
            message: msg.into(),
        }
    }

    /// Create an invalid range error
    pub fn invalid_range(msg: impl Into<String>) -> Self {
        Self::InvalidRange {
            message: msg.into(),
        }
    }

    /// Create an out of bounds error
    pub fn out_of_bounds(value: f64, min: f64, max: f64) -> Self {
        Self::OutOfBounds { value, min, max }
    }

    /// Create an invalid data error
    pub fn invalid_data(msg: impl Into<String>) -> Self {
        Self::InvalidData {
            message: msg.into(),
        }
    }

    /// Create a parse error
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::ParseError {
            message: msg.into(),
        }
    }

    /// Create a configuration error
    pub fn config_error(msg: impl Into<String>) -> Self {
        Self::ConfigError {
            message: msg.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = D3Error::invalid_domain("min > max");
        assert_eq!(err.to_string(), "Invalid domain: min > max");
    }

    #[test]
    fn test_out_of_bounds() {
        let err = D3Error::out_of_bounds(150.0, 0.0, 100.0);
        assert!(err.to_string().contains("150"));
        assert!(err.to_string().contains("0"));
        assert!(err.to_string().contains("100"));
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<D3Error>();
    }

    #[test]
    fn test_error_equality() {
        let err1 = D3Error::invalid_domain("test");
        let err2 = D3Error::invalid_domain("test");
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_error_clone() {
        let err1 = D3Error::invalid_data("clone test");
        let err2 = err1.clone();
        assert_eq!(err1, err2);
    }
}
