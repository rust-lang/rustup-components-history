//! [![pipeline status](https://gitlab.com/mexus/rustup-components-availability/badges/master/pipeline.svg)](https://gitlab.com/mexus/rustup-components-availability/commits/master)
//! [![crates.io](https://img.shields.io/crates/v/rustup-available-packages.svg)](https://crates.io/crates/rustup-available-packages)
//! [![docs.rs](https://docs.rs/rustup-available-packages/badge.svg)](https://docs.rs/rustup-available-packages)
//!
//! [[Release docs]](https://docs.rs/rustup-available-packages/)
//!
//! [[Master docs]](https://mexus.gitlab.io/rustup-components-availability/rustup_available_packages/)
//!
//! A library that helps you to find out which packages are available in your **rustup** tool for
//! specific dates and targets.
//!
//! Suggestions and critiques are welcome!

#![deny(missing_docs)]

#[macro_use]
extern crate derive_more;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate either;
extern crate reqwest;
extern crate toml;

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
#[derive(Debug, From, Fail)]
pub enum Error {
    /// TOML parsing error.
    #[fail(display = "TOML deserialization error {} on manifest {}", _0, _1)]
    TomlDe(#[cause] toml::de::Error, String),

    /// TOML serialization error.
    #[fail(display = "TOML serialization error {} on manifest {}", _0, _1)]
    TomlSer(#[cause] toml::ser::Error, String),

    /// Error in the `reqwest` library.
    #[fail(display = "reqwest error {} on url {}", _0, _1)]
    Reqwest(#[cause] reqwest::Error, String),

    /// Got a bad HTTP response.
    #[fail(display = "HTTP error {} on url {}", _0, _1)]
    BadResponse(reqwest::StatusCode, String),

    /// I/O error.
    #[fail(display = "I/O error {} at {}", _0, _1)]
    Io(#[cause] io::Error, String),
}
