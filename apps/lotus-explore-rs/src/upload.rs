// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Inlined upload helpers for lotus-explore-rs.
//!
//! Provides WASM-only file blob extraction and streaming read utilities.
//! Copied from the shared `upload` crate to make lotus-explore-rs standalone.

#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Uint8Array};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
#[cfg(target_arch = "wasm32")]
use web_sys::{HtmlAnchorElement, Url};

/// The browser `Blob` type — re-exported from web_sys for WASM.
#[cfg(target_arch = "wasm32")]
pub type UploadBlob = web_sys::Blob;

/// Placeholder type on non-WASM targets where browser APIs are unavailable.
#[cfg(not(target_arch = "wasm32"))]
pub type UploadBlob = ();

/// Result of extracting a file from a form-data or drag-drop event.
#[derive(Debug)]
pub struct ExtractedFile {
    /// The file as a browser `UploadBlob` ready for streaming reads.
    pub blob: UploadBlob,
    /// The original filename from the `<input>` or drag source.
    pub name: String,
}

/// Error returned by blob read operations.
#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    /// A browser/JS API call failed.
    #[error("blob read error: {0}")]
    UploadBlob(String),

    /// The stream ended before the parser expected it to (truncated input).
    #[error("unexpected EOF while reading stream")]
    UnexpectedEof,

    /// A structural invariant of the input was violated.
    #[error("expected {expected}")]
    Expected {
        /// Human-readable description of what was expected.
        expected: &'static str,
    },

    /// The operation is only available inside the browser.
    #[error("download is only available in the browser")]
    BrowserOnly,

    /// A generic, formatted error for app-level validation.
    #[error("{0}")]
    Other(String),
}

impl UploadError {
    /// Convenience for "expected X but got end of stream".
    #[must_use]
    pub const fn expected(expected: &'static str) -> Self {
        Self::Expected { expected }
    }

    /// Convenience for wrapping a message.
    #[must_use]
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for UploadError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        use wasm_bindgen::JsCast;
        Self::UploadBlob(value.dyn_ref::<js_sys::JsString>().map_or_else(
            || format!("{value:?}"),
            |s| s.as_string().unwrap_or_default(),
        ))
    }
}

/// Default chunk size for UploadBlob reads (16 MiB).
const CHUNK_SIZE: usize = 16 * 1024 * 1024;

/// Default byte interval for progress reporting (4 MiB).
const PROGRESS_BYTE_INTERVAL: u64 = 4 * 1024 * 1024;

/// Default time interval for progress reporting (120 ms).
const PROGRESS_TIME_INTERVAL_MS: f64 = 120.0;

/// Throttles callbacks based on bytes processed and wall-clock time elapsed.
#[derive(Debug)]
struct ProgressThrottler<F, T> {
    last_reported_bytes: u64,
    last_reported_time: f64,
    callback: F,
    time_fn: T,
    byte_threshold: u64,
    time_threshold_ms: f64,
}

impl<F, T> ProgressThrottler<F, T>
where
    F: FnMut(u64, u64),
    T: Fn() -> f64,
{
    #[must_use]
    fn new(callback: F, time_fn: T, byte_threshold: u64, time_threshold_ms: f64) -> Self {
        let now = time_fn();
        Self {
            last_reported_bytes: 0,
            last_reported_time: now,
            callback,
            time_fn,
            byte_threshold,
            time_threshold_ms,
        }
    }

    fn maybe_report(&mut self, processed: u64, total: u64) -> bool {
        let now = (self.time_fn)();
        let bytes_delta = processed.saturating_sub(self.last_reported_bytes);

        if bytes_delta >= self.byte_threshold
            || now - self.last_reported_time >= self.time_threshold_ms
        {
            (self.callback)(processed, total);
            self.last_reported_bytes = processed;
            self.last_reported_time = now;
            true
        } else {
            false
        }
    }

    const fn force_next(&mut self) {
        self.last_reported_bytes = u64::MAX;
        self.last_reported_time = f64::NEG_INFINITY;
    }
}

/// A line-oriented, chunked reader over a browser [`UploadBlob`].
///
/// Yields `String` lines (without trailing `\n` or `\r`) one at a time via
/// [`next_line`](UploadBlobLines::next_line). Internally buffers one 16 MiB chunk.
#[cfg(target_arch = "wasm32")]
#[derive(Debug)]
pub struct UploadBlobLines<F> {
    blob: UploadBlob,
    total_bytes: u64,
    offset: u64,
    buffer: Vec<u8>,
    buf_start: usize,
    processed: u64,
    progress: ProgressThrottler<F, fn() -> f64>,
}

#[cfg(target_arch = "wasm32")]
impl<F> UploadBlobLines<F>
where
    F: FnMut(u64, u64),
{
    /// Creates a new line reader for the given blob.
    #[must_use]
    pub fn new(blob: &UploadBlob, on_progress: F) -> Self {
        Self {
            blob: blob.clone(),
            total_bytes: blob.size() as u64,
            offset: 0,
            buffer: Vec::with_capacity(CHUNK_SIZE),
            buf_start: 0,
            processed: 0,
            progress: ProgressThrottler::new(
                on_progress,
                js_sys::Date::now,
                PROGRESS_BYTE_INTERVAL,
                PROGRESS_TIME_INTERVAL_MS,
            ),
        }
    }

    /// Total blob size in bytes.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Returns the next line from the blob, or `Ok(None)` at end-of-stream.
    pub async fn next_line(&mut self) -> Result<Option<String>, UploadError> {
        loop {
            // Try to extract a complete line from the current buffer.
            if let Some(line) = self.take_line_from_buffer() {
                return Ok(Some(line));
            }

            // No complete line yet — check for EOF.
            if self.offset >= self.total_bytes {
                if self.buf_start < self.buffer.len() {
                    let remaining =
                        String::from_utf8_lossy(&self.buffer[self.buf_start..]).into_owned();
                    self.buf_start = self.buffer.len();
                    return Ok(Some(remaining));
                }
                return Ok(None);
            }

            self.load_next_chunk().await?;
        }
    }

    fn take_line_from_buffer(&mut self) -> Option<String> {
        let available = &self.buffer[self.buf_start..];
        if let Some(pos) = available.iter().position(|b| *b == b'\n') {
            let line_bytes = &available[..pos];
            let mut line = String::from_utf8_lossy(line_bytes).into_owned();
            self.buf_start += pos + 1;
            if line.ends_with('\r') {
                line.pop();
            }
            // Compact buffer when it's more than half consumed.
            if self.buf_start > self.buffer.len() / 2 {
                self.buffer.drain(..self.buf_start);
                self.buf_start = 0;
            }
            Some(line)
        } else {
            None
        }
    }

    async fn load_next_chunk(&mut self) -> Result<(), UploadError> {
        let start = self.offset;
        let end = (self.offset + CHUNK_SIZE as u64).min(self.total_bytes);
        let slice = self
            .blob
            .slice_with_f64_and_f64(start as f64, end as f64)
            .map_err(UploadError::from)?;
        let bytes = JsFuture::from(slice.array_buffer()).await?;
        let array = Uint8Array::new(&bytes);
        let chunk_len = array.byte_length() as usize;
        let mut chunk_bytes = vec![0u8; chunk_len];
        array.copy_to(&mut chunk_bytes);
        self.buffer.extend_from_slice(&chunk_bytes);
        self.offset = end;
        self.processed = self.processed.saturating_add((end - start).max(1));
        if self.progress.maybe_report(self.processed, self.total_bytes) {
            // Yield to the event loop so the UI stays responsive.
            TimeoutFuture::new(0).await;
        }
        Ok(())
    }
}

/// Non-WASM stub for UploadBlobLines.
#[cfg(not(target_arch = "wasm32"))]
pub struct UploadBlobLines<F> {
    _phantom: std::marker::PhantomData<F>,
}

#[cfg(not(target_arch = "wasm32"))]
impl<F> UploadBlobLines<F>
where
    F: FnMut(u64, u64),
{
    #[must_use]
    pub fn new(_blob: &UploadBlob, _on_progress: F) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        0
    }

    pub async fn next_line(&mut self) -> Result<Option<String>, UploadError> {
        Err(UploadError::BrowserOnly)
    }
}

/// Extracts a [`UploadBlob`] from the first `FileData` in a list.
///
/// Callers pass the result of `evt.data().files()` (which works for both
/// file-input `FormData` events and drag-drop `DragData` events).
///
/// # Errors
/// Returns a string message if the file cannot be downcast to a `UploadBlob`.
#[allow(clippy::unnecessary_wraps)]
// `Result<Option<..>, _>` is intentional: the wasm path returns `Err` for
// unsupported file types, so the `Result` is genuinely used there even though
// the native (non-wasm) cfg branch only ever returns `Ok(None)` — which is what
// trips the lint. Scoped here instead of allowed workspace-wide.
pub fn extract_blob_from_file_data(
    files: &[dioxus::html::FileData],
) -> Result<Option<ExtractedFile>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        type WebFile = web_sys::File;

        let Some(file) = files.iter().next() else {
            return Ok(None);
        };

        let file_name = file.name();
        let Some(web_file) = file.inner().downcast_ref::<WebFile>() else {
            return Err("This file type is not supported in the browser.".to_string());
        };

        let blob = web_file
            .clone()
            .dyn_into::<UploadBlob>()
            .map_err(|_| "Unable to read the selected file as a blob.".to_string())?;
        Ok(Some(ExtractedFile {
            blob,
            name: file_name,
        }))
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = files;
        Ok(None)
    }
}

/// Read a blob as a string using streaming (handles large files without OOM).
///
/// This uses [`UploadBlobLines`] internally to stream the file in 16 MiB chunks,
/// avoiding the memory issues of [`FileReader::read_as_text()`] which loads
/// the entire file into memory at once.
///
/// # Errors
/// Returns an error if the blob cannot be read or contains invalid UTF-8.
#[cfg(target_arch = "wasm32")]
pub async fn read_blob_string(blob: &UploadBlob) -> Result<String, UploadError> {
    let mut reader = UploadBlobLines::new(blob, |_, _| {});
    let mut out = String::new();
    while let Some(line) = reader.next_line().await? {
        out.push_str(&line);
        out.push('\n');
    }
    Ok(out)
}

/// Non-WASM stub for `read_blob_string`.
///
/// # Errors
/// Always returns an error since this function is only available on WASM targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn read_blob_string(_blob: &UploadBlob) -> Result<String, UploadError> {
    Err(UploadError::other(
        "read_blob_string only available on WASM targets",
    ))
}

/// Sanitizes a filename for safe browser download.
///
/// Removes control characters and replaces path separators and quotes with
/// underscores. No external crate required.
#[must_use]
pub fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.trim().chars() {
        if c.is_control() {
            continue;
        }
        match c {
            '/' | '\\' | '"' | '\'' | '\n' | '\r' => out.push('_'),
            _ => out.push(c),
        }
    }
    out.trim_matches('.').trim().to_string()
}

// ── Download helpers ────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
fn blob_url_from_str(content: &str, mime: &str) -> Result<String, String> {
    let parts = Array::new();
    parts.push(&JsValue::from_str(content));

    let blob = {
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime);
        UploadBlob::new_with_str_sequence_and_options(&parts, &options)
            .or_else(|_| UploadBlob::new_with_str_sequence(&parts))
    };
    let blob = blob.map_err(|e| format!("failed to create blob: {e:?}"))?;
    Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("failed to create object URL: {e:?}"))
}

/// Triggers a browser download of `content` as a text file.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The full filename (including extension) for the download
///
/// # Errors
/// Returns a message if the download cannot be triggered.
#[cfg(target_arch = "wasm32")]
pub fn download_text(content: &str, filename: &str) -> Result<(), String> {
    let safe_name = sanitize_filename(filename);
    let url = blob_url_from_str(content, "text/plain;charset=utf-8")?;

    click_download_anchor(&url, &safe_name, false)
        .map(|_| ())
        .map_err(|e| format!("download failed: {e}"))
}

/// Downloads content as a blob with a specific extension and MIME type.
///
/// # Arguments
/// - `content`: The text content to download
/// - `filename`: The base filename (without extension)
/// - `extension`: File extension including dot (e.g. `".csv"`, `".json"`)
/// - `mime`: MIME type for the blob
///
/// # Errors
/// Returns a message if the download cannot be triggered.
#[cfg(target_arch = "wasm32")]
pub fn download_text_as_blob(
    content: &str,
    filename: &str,
    extension: &str,
    mime: &str,
) -> Result<(), String> {
    let safe_name = if extension.is_empty() {
        sanitize_filename(filename)
    } else {
        let name = sanitize_filename(filename);
        if name.ends_with(extension) {
            name
        } else {
            format!("{name}{extension}")
        }
    };
    let url = blob_url_from_str(content, mime)?;
    click_download_anchor(&url, &safe_name, false)
        .map(|_| ())
        .map_err(|e| format!("download failed: {e}"))
}

/// Triggers a browser download of a URL (e.g. a QLever export URL or a remote file).
///
/// Opens the URL in a new tab / triggers an anchor click.  Returns `false` if
/// the browser does not support programmatic clicks (extremely rare).
#[cfg(target_arch = "wasm32")]
pub fn download_url(url: &str, filename: &str) -> bool {
    let safe_name = sanitize_filename(filename);
    click_download_anchor(url, &safe_name, true).unwrap_or_else(|_| {
        web_sys::window()
            .and_then(|w| w.open_with_url(url).ok())
            .is_some()
    })
}

#[cfg(target_arch = "wasm32")]
fn click_download_anchor(href: &str, filename: &str, new_tab: bool) -> Result<bool, String> {
    let window = web_sys::window().ok_or("no window object")?;
    let document = window.document().ok_or("no document object")?;
    let anchor: HtmlAnchorElement = document
        .create_element("a")
        .map_err(|e| format!("failed to create anchor: {e:?}"))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| format!("failed to cast anchor: {e:?}"))?;

    anchor.set_href(href);
    anchor.set_download(filename);
    anchor.set_rel("noopener noreferrer");
    if new_tab {
        anchor.set_target("_blank");
    }

    let body = document.body().ok_or("no document body")?;
    body.append_child(&anchor)
        .map_err(|e| format!("failed to append anchor: {e:?}"))?;
    anchor.click();
    let _ = body.remove_child(&anchor);

    Ok(true)
}

/// Submits a hidden form to trigger a browser download via POST.
///
/// Used when the URL + query payload is too large for a GET request.
///
/// # Errors
/// Returns a message if the form cannot be created or submitted.
#[cfg(target_arch = "wasm32")]
pub async fn submit_download_form(endpoint: &str, fields: &[(&str, &str)]) -> Result<(), String> {
    let window = web_sys::window().ok_or("no window object")?;
    let document = window.document().ok_or("no document object")?;

    let form = document
        .create_element("form")
        .map_err(|e| format!("failed to create form: {e:?}"))?
        .dyn_into::<web_sys::HtmlFormElement>()
        .map_err(|e| format!("failed to cast form: {e:?}"))?;
    form.set_method("POST");
    form.set_action(endpoint);
    form.set_target("_blank");
    let _ = form.set_attribute("accept-charset", "UTF-8");
    let _ = form.set_attribute("enctype", "application/x-www-form-urlencoded");

    for (name, value) in fields {
        let input = document
            .create_element("input")
            .map_err(|e| format!("failed to create input {name}: {e:?}"))?
            .dyn_into::<web_sys::HtmlInputElement>()
            .map_err(|e| format!("failed to cast input {name}: {e:?}"))?;
        input.set_type("hidden");
        input.set_name(name);
        input.set_value(value);
        form.append_child(&input)
            .map_err(|e| format!("failed to append input {name}: {e:?}"))?;
    }

    let body = document.body().ok_or("no document body")?;
    body.append_child(&form)
        .map_err(|e| format!("failed to append form: {e:?}"))?;
    form.submit()
        .map_err(|e| format!("failed to submit form: {e:?}"))?;
    let _ = body.remove_child(&form);

    // Yield so the form submission takes effect before the caller continues
    // (intentionally drop the future — the await advances the microtask queue).
    let _ = TimeoutFuture::new(0).await;
    Ok(())
}

// ── Non-WASM stubs ───────────────────────────────────────────────────────────

/// Triggers a browser download of text content (native stub — returns `Err`).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_text(_content: &str, _filename: &str) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

/// Downloads content as a blob (native stub — returns `Err`).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub fn download_text_as_blob(
    _content: &str,
    _filename: &str,
    _extension: &str,
    _mime: &str,
) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

/// Triggers a browser download of a URL (native stub — returns `false`).
///
/// On non-WASM targets browsers aren't available, so this always returns `false`.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub const fn download_url(_url: &str, _filename: &str) -> bool {
    false
}

/// Submits a hidden form for download (native stub — returns an error).
///
/// On non-WASM targets browsers aren't available, so this always returns `Err`.
///
/// # Errors
/// Always returns an error on native targets.
#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_download_form(_endpoint: &str, _fields: &[(&str, &str)]) -> Result<(), String> {
    Err("Download is only available in the browser".to_string())
}

#[cfg(test)]
mod tests {
    use super::sanitize_filename;

    #[test]
    fn sanitize_removes_path_separators() {
        assert_eq!(sanitize_filename("a/b\\c"), "a_b_c");
    }

    #[test]
    fn sanitize_strips_control_chars() {
        assert_eq!(sanitize_filename("file\x00name"), "filename");
    }

    #[test]
    fn sanitize_strips_leading_dots() {
        assert_eq!(sanitize_filename("...file.txt"), "file.txt");
    }

    #[test]
    fn sanitize_empty_input() {
        assert_eq!(sanitize_filename("   "), "");
        assert_eq!(sanitize_filename("."), "");
    }

    #[test]
    fn sanitize_replaces_quotes_with_underscore() {
        assert_eq!(sanitize_filename("file\"name"), "file_name");
        assert_eq!(sanitize_filename("file'name"), "file_name");
    }

    #[test]
    fn sanitize_preserves_safe_names() {
        assert_eq!(sanitize_filename("lotus_results.csv"), "lotus_results.csv");
        assert_eq!(sanitize_filename("my_file-01.json"), "my_file-01.json");
    }

    #[test]
    fn sanitize_strips_trailing_whitespace() {
        assert_eq!(sanitize_filename("file.txt "), "file.txt");
        assert_eq!(sanitize_filename(" file.txt"), "file.txt");
    }

    #[test]
    fn sanitize_unicode_passthrough() {
        assert_eq!(sanitize_filename("résultats.csv"), "résultats.csv");
        assert_eq!(sanitize_filename("α-β-γ.rdf"), "α-β-γ.rdf");
    }
}
