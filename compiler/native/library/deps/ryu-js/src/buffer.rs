// `ryu_js::Buffer`: formats one number at a time into text it owns.

/// A reusable text buffer for formatting numbers the way JavaScript
/// prints them (`NaN`, `Infinity`, `-Infinity`, and `0` for -0.0).
pub struct Buffer {
    text: String,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer { text: String::new() }
    }

    /// The JavaScript text of `f`, valid until the next call.
    pub fn format<F: Float>(&mut self, f: F) -> &str {
        self.text.clear();
        f.format_js(&mut self.text);
        self.text.as_str()
    }
}

/// Upstream's Clone gives a fresh buffer; so does this one.
impl Clone for Buffer {
    fn clone(&self) -> Self {
        Buffer::new()
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Buffer::new()
    }
}

/// The float types `Buffer::format` takes: f64 only in the adapter.
pub trait Float: Sealed {}

impl Float for f64 {}

/// Writes the JavaScript text of one float; public in a private module,
/// as upstream seals `Float`.
pub trait Sealed: Copy {
    fn format_js(self, out: &mut String);
}

impl Sealed for f64 {
    fn format_js(self, out: &mut String) {
        core::intrinsics::ryu_js_fmt_f64(out, self)
    }
}
