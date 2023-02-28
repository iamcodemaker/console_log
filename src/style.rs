/// Log message styling.
///
/// Adapted from <https://gitlab.com/limira-rs/wasm-logger/-/blob/0c16227/src/lib.rs#L72-85>
pub(crate) struct Style<'s> {
    pub trace: &'s str,
    pub debug: &'s str,
    pub info: &'s str,
    pub warn: &'s str,
    pub error: &'s str,
    pub file_line: &'s str,
    pub text: &'s str,
}

impl Style<'static> {
    /// Returns default style values.
    pub const fn default() -> Self {
        macro_rules! bg_color {
            ($color:expr) => {
                concat!("color: white; padding: 0 3px; background: ", $color, ";")
            };
        };

        Style {
            trace: bg_color!("gray"),
            debug: bg_color!("blue"),
            info: bg_color!("green"),
            warn: bg_color!("orange"),
            error: bg_color!("darkred"),
            file_line: "font-weight: bold; color: inherit",
            text: "background: inherit; color: inherit",
        }
    }
}
