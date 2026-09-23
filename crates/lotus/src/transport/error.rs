// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Gateway-error detection and HTTP error-text compaction.
//!
//! When an upstream SPARQL endpoint returns an error, it may return:
//! - An HTML page (e.g. from nginx or Cloudflare) instead of structured data.
//! - A JSON payload with an `exception` field (`QLever`'s format).
//! - A plain-text detail message.
//!
//! These helpers extract a concise, log-friendly summary from any of those.

/// Returns `true` if the response body looks like an HTML gateway error page
/// (e.g. "502 Bad Gateway" from `nginx` or `Cloudflare`).
///
/// Inspects the first 2 048 bytes only — enough to catch `<html>` tags and
/// gateway keywords without scanning the entire payload.
pub(super) fn looks_like_gateway_error(body: &str) -> bool {
    let cap = body.len().min(2048);
    let safe_end = (0..=cap)
        .rev()
        .find(|&i| body.is_char_boundary(i))
        .unwrap_or(0);
    let sample = &body[..safe_end];
    let html = contains_ci(sample, "<html")
        || contains_ci(sample, "<!doctype")
        || contains_ci(sample, "<head")
        || contains_ci(sample, "<title");
    let gateway = contains_ci(sample, "bad gateway")
        || contains_ci(sample, "gateway timeout")
        || contains_ci(sample, "service unavailable")
        || contains_ci(sample, "upstream")
        || contains_ci(sample, "nginx")
        || contains_ci(sample, "cloudflare");
    html && gateway
}

/// Case-insensitive substring search on raw bytes (UTF-8 is not required).
fn contains_ci(h: &str, needle: &str) -> bool {
    if needle.len() > h.len() {
        return false;
    }
    let nb = needle.as_bytes();
    let hb = h.as_bytes();
    for i in 0..=hb.len() - nb.len() {
        if hb
            .get(i..i + nb.len())
            .is_some_and(|slice| slice.iter().zip(nb).all(|(a, b)| a.eq_ignore_ascii_case(b)))
        {
            return true;
        }
    }
    false
}

// ── HTTP status classification ────────────────────────────────────────────────
//
// Pure, `const` status-code classifiers used by the retry policy in
// [`crate::transport::execute`]. Centralising the ranges (rather than inlining
// magic-number checks at each call site) keeps the retry decision logic in one
// place and makes the boundary semantics — shared by both the native and the
// wasm transport paths — explicitly unit-testable.

/// `2xx` success statuses.
pub(super) const fn is_success(code: u16) -> bool {
    code >= 200 && code <= 299
}

/// `4xx` client errors, **including** `429` (which is additionally handled
/// with backoff on the SPARQL-POST path).
pub(super) const fn is_client_error(code: u16) -> bool {
    code >= 400 && code < 500
}

/// `429 Too Many Requests`.
pub(super) const fn is_rate_limit(code: u16) -> bool {
    code == 429
}

/// Reduce a raw HTTP error body to a concise, single-line description.
///
/// Preference order:
/// 1. JSON `"exception"` field (`QLever`'s format)
/// 2. First meaningful non-brace line
///
/// Truncates to 240 characters with an ellipsis (`…`) suffix.
pub(super) fn compact_http_error_text(body: &str) -> String {
    const MAX_CHARS: usize = 240;
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "empty response body".into();
    }

    if let Some(exception) = parse_json_exception_field(trimmed) {
        return truncate_chars(exception.trim(), MAX_CHARS);
    }

    let first_meaningful_line = trimmed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && *line != "{" && *line != "}")
        .unwrap_or(trimmed);

    truncate_chars(first_meaningful_line.trim_matches(','), MAX_CHARS)
}

/// Extract the value of the `"exception"` field from a JSON string.
///
/// This is a lightweight hand-rolled parser — it doesn't need to handle
/// arbitrary JSON, only `QLever`'s error format.
fn parse_json_exception_field(input: &str) -> Option<String> {
    let key = "\"exception\"";
    let key_pos = input.find(key)?;
    let mut rest = input[key_pos + key.len()..].trim_start();
    rest = rest.strip_prefix(':')?.trim_start();
    let quoted = rest.strip_prefix('"')?;

    let mut out = String::new();
    let mut escaped = false;
    for ch in quoted.chars() {
        if escaped {
            let decoded = match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            };
            out.push(decoded);
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

/// Truncate a string to at most `max_chars` Unicode code points, appending
/// `…` when truncation occurs.
fn truncate_chars(text: &str, max_chars: usize) -> String {
    // Single forward scan: find the byte position after `max_chars` codepoints.
    let mut chars = text.char_indices();
    match chars.nth(max_chars) {
        // Fewer than max_chars codepoints — no truncation needed.
        None => text.to_string(),
        Some((byte_pos, _)) => {
            let mut out = String::with_capacity(byte_pos + 4); // 4 bytes for '…'
            out.push_str(&text[..byte_pos]);
            out.push('…');
            out
        }
    }
}
