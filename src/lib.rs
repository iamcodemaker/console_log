#![deny(missing_docs)]

//! A logger that logs to the browser's console.
//!
//! # Example
//!
//! ```rust,no_run
//! use log::Level;
//! use log::info;
//! fn main() {
//!     console_log::init_with_level(Level::Debug);
//!
//!     info!("It works!");
//!
//!     // ...
//! }
//! ```
//!
//! # Log Levels
//! 
//! Rust's log levels map to the browser's console log in the following way.
//! 
//! | Rust       | Web Console       |
//! |------------|-------------------|
//! | `trace!()` | `console.debug()` |
//! | `debug!()` | `console.log()`   |
//! | `info!()`  | `console.info()`  |
//! | `warn!()`  | `console.warn()`  |
//! | `error!()` | `console.error()` |
//! 
//! # Code Size
//! 
//! [Twiggy](https://github.com/rustwasm/twiggy) reports this library adding about
//! 180Kb to the size of a minimal wasm binary in a debug build. If you want to
//! avoid this, mark the library as optional and conditionally initialize it in
//! your code for non-release builds.
//! 
//! `Cargo.toml`
//! ```toml
//! [dependencies]
//! cfg_if = "0.1"
//! log = "0.4"
//! console_log = { version = "0.1", optional = true }
//! 
//! [features]
//! default = ["console_log"]
//! ```
//! 
//! `lib.rs`
//! ```rust,ignore
//! use wasm_bindgen::prelude::*;
//! use cfg_if::cfg_if;
//!
//! cfg_if! {
//!     if #[cfg(feature = "console_log")] {
//!         fn init_log() {
//!             use log::Level;
//!             console_log::init_with_level(Level::Trace).expect("error initializing log");
//!         }
//!     } else {
//!         fn init_log() {}
//!     }
//! }
//! 
//! #[wasm_bindgen]
//! fn main() {
//!     init_log();
//!     // ...
//! }
//! ```
//! 
//! # Limitations
//! 
//! The file and line number information associated with the log messages reports locations from
//! the shims generated by `wasm-bindgen`, not the location of the logger call.

use log::{Log, Level, Record, Metadata, SetLoggerError};
use web_sys::console;

static LOGGER: WebConsoleLogger = WebConsoleLogger {};

struct WebConsoleLogger {}

impl Log for WebConsoleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // pick the console.log() variant for the appropriate logging level
        let console_log = match record.level() {
            Level::Error => console::error_1,
            Level::Warn => console::warn_1,
            Level::Info => console::info_1,
            Level::Debug => console::log_1,
            Level::Trace => console::debug_1,
        };

        console_log(&format!("{}", record.args()).into());
    }

    fn flush(&self) {}
}

/// Initializes the global logger setting `max_log_level` to the given value.
///
/// ## Example
///
/// ```
/// use log::Level;
/// fn main() {
///     console_log::init_with_level(Level::Debug).expect("error initializing logger");
/// }
/// ```
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initializes the global logger with `max_log_level` set to `Level::Info` (a sensible default).
///
/// ## Example
///
/// ```
/// fn main() {
///     console_log::init().expect("error initializing logger");
/// }
/// ```
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Info)
}
