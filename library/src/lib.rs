//! [![Travis CI badge](https://api.travis-ci.com/rust-lang/rustup-components-history.svg?branch=master)](https://travis-ci.com/github/rust-lang/rustup-components-history)
//! [![crates.io](https://img.shields.io/crates/v/rustup-available-packages.svg)](https://crates.io/crates/rustup-available-packages)
//! [![docs.rs](https://docs.rs/rustup-available-packages/badge.svg)](https://docs.rs/rustup-available-packages)
//!
//! [[Release docs]](https://docs.rs/rustup-available-packages/)
//!
//! A library that helps you to find out which packages are available in your **rustup** tool for
//! specific dates and targets.
//!
//! Suggestions and critiques are welcome!

#![deny(missing_docs)]

pub mod availability;
pub mod cache;
mod downloader;
pub mod manifest;
mod skip_errors;
mod source;
pub mod table;
mod types;

pub use availability::AvailabilityData;
pub use downloader::Downloader;
pub use source::{DefaultSource, SourceInfo};
use std::io;

/// An error that might happen inside the library.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// TOML parsing error.
    #[error("TOML deserialization error {0} on manifest {1}")]
    TomlDe(#[source] toml::de::Error, String),

    /// TOML serialization error.
    #[error("TOML serialization error {0} on manifest {1}")]
    TomlSer(#[source] toml::ser::Error, String),

    /// Error in the `reqwest` library.
    #[error("reqwest error {0} on url {1}")]
    Reqwest(#[source] reqwest::Error, String),

    /// Got a bad HTTP response.
    #[error("HTTP error {0} on url {1}")]
    BadResponse(reqwest::StatusCode, String),

    /// I/O error.
    #[error("I/O error {0} at {1}")]
    Io(#[source] io::Error, String),
}
