#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/console_log/0.2.0")]

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
//! # Getting Fancy
//!
//! The feature set provided by this crate is intentionally very basic. If you need more flexible
//! formatting of log messages (timestamps, file and line info, etc.) this crate can be used with
//! the [`fern`] logger via the [`console_log::log`] function.
//!
//! ## Colors
//!
//! The `"color"` feature adds styling to the log messages.
//!
//! `Cargo.toml`
//! ```toml
//! console_log = { version = "0.2", features = ["color"] }
//! ```
//!
//! The styled log messages will be rendered as follows:
//!
//! ![Styled log messages](img/log_messages_styled.png)
//!
//! # Code Size
//!
//! [Twiggy] reports this library adding about 180Kb to the size of a minimal wasm binary in a
//! debug build. If you want to avoid this, mark the library as optional and conditionally
//! initialize it in your code for non-release builds.
//!
//! `Cargo.toml`
//! ```toml
//! [dependencies]
//! cfg-if = "0.1"
//! log = "0.4"
//! console_log = { version = "0.2", optional = true }
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
//! pub fn main() {
//!     init_log();
//!     // ...
//! }
//! ```
//!
//! # Limitations
//!
//! The file and line number information associated with the log messages reports locations from
//! the shims generated by `wasm-bindgen`, not the location of the logger call.
//!
//! [Twiggy]: https://github.com/rustwasm/twiggy
//! [`console_log::log`]: fn.log.html
//! [`fern`]: https://docs.rs/fern

use log::{Level, Log, Metadata, Record, SetLoggerError};
use web_sys::console;

#[cfg(feature = "color")]
use wasm_bindgen::JsValue;

#[cfg(feature = "color")]
const STYLE: style::Style<'static> = style::Style::default();

#[cfg(feature = "color")]
#[doc(hidden)]
mod style;

static LOGGER: WebConsoleLogger = WebConsoleLogger {};

struct WebConsoleLogger {}

impl Log for WebConsoleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        log(record);
    }

    fn flush(&self) {}
}

/// Print a `log::Record` to the browser's console at the appropriate level.
///
/// This function is useful for integrating with the [`fern`](https://crates.io/crates/fern) logger
/// crate.
///
/// ## Example
/// ```rust,ignore
/// fern::Dispatch::new()
///     .chain(fern::Output::call(console_log::log))
///     .apply()?;
/// ```
#[cfg_attr(not(feature = "color"), inline)]
pub fn log(record: &Record) {
    #[cfg(not(feature = "color"))]
    {
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

    #[cfg(feature = "color")]
    {
        // pick the console.log() variant for the appropriate logging level
        let console_log = match record.level() {
            Level::Error => console::error_4,
            Level::Warn => console::warn_4,
            Level::Info => console::info_4,
            Level::Debug => console::log_4,
            Level::Trace => console::debug_4,
        };

        let message = {
            let message = format!(
                "%c{level}%c {file}:{line} %c\n{text}",
                level = record.level(),
                file = record.file().unwrap_or_else(|| record.target()),
                line = record
                    .line()
                    .map_or_else(|| "[Unknown]".to_string(), |line| line.to_string()),
                text = record.args(),
            );
            JsValue::from(&message)
        };

        let level_style = {
            let style_str = match record.level() {
                Level::Trace => STYLE.trace,
                Level::Debug => STYLE.debug,
                Level::Info => STYLE.info,
                Level::Warn => STYLE.warn,
                Level::Error => STYLE.error,
            };

            JsValue::from(style_str)
        };

        let file_line_style = JsValue::from_str(STYLE.file_line);
        let text_style = JsValue::from_str(STYLE.text);
        console_log(&message, &level_style, &file_line_style, &text_style);
    }
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
#[inline]
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
#[inline]
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Info)
}
