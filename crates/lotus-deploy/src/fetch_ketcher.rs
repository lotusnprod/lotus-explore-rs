// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Fetch the standalone Ketcher editor bundle and slim it.
//!
//! Downloads the official `ketcher-standalone-<ver>.zip` release from GitHub,
//! extracts the files the editor actually uses into `public/assets/ketcher`,
//! and skips the unused ~29 MB entry bundles (`closable` / `duo` / `popup`,
//! plus their license files) that this editor never loads — only
//! `main.<hash>.js` is referenced by ketcher's own `index.html`. That trims
//! ~87 MB of dead weight from the deploy (and from local clones), with no
//! effect on the editor at runtime.
//!
//! Run from the `src` crate directory (resolves `public/assets/ketcher` relative
//! to the working directory).
//!
//! # Environment
//!
//! * `KETCHER_VERSION` — release tag (default `3.18.0`).
//! * `KETCHER_DIR` — output directory (default `public/assets/ketcher`).
//! * `KETCHER_URL` — fully override the release URL.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufWriter, Cursor, Write};
use std::path::{Path, PathBuf};

use reqwest::blocking::Client;
use zip::ZipArchive;

const DEFAULT_VERSION: &str = "3.18.0";
const OWNER_REPO: &str = "epam/ketcher";
const DEFAULT_DIR: &str = "public/assets/ketcher";

/// Unused standalone "entry" bundles (and their license files) that ketcher's
/// `index.html` never references — only `main.<hash>.js` is loaded by the
/// editor iframe. Matches the original shell helper's `rm` globs: only the
/// `closable`/`duo`/`popup` JavaScript bundles and their `.LICENSE.txt` are
/// dropped; the small mode-specific `*.html`/`*.css` entry points are kept.
#[must_use]
fn is_unused_entry(name: &str) -> bool {
    let Some(file_name) = name.rsplit('/').next() else {
        return false;
    };
    let is_entry_bundle = file_name.starts_with("closable.")
        || file_name.starts_with("duo.")
        || file_name.starts_with("popup.");
    is_entry_bundle
        && (Path::new(file_name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
            || file_name.ends_with(".js.LICENSE.txt"))
}

/// macOS zip metadata that must never be extracted: the `__MACOSX/` tree and
/// `._`-prefixed resource forks. The ketcher release zip ships these
/// (it was archived on macOS), and dioxus-cli's asset copier aborts on them
/// with "stream did not contain valid UTF-8" / esbuild `Unexpected "\x00"`.
/// The original shell helper avoided this implicitly via `cp -r standalone/*`;
/// this makes it explicit and keeps the resource forks out of `public/assets`.
#[must_use]
fn is_macos_junk(name: &str) -> bool {
    if name == "__MACOSX" || name.starts_with("__MACOSX/") {
        return true;
    }
    let Some(file_name) = name.rsplit('/').next() else {
        return false;
    };
    file_name.starts_with("._")
}

#[must_use]
fn release_url(version: &str) -> String {
    format!(
        "https://github.com/{OWNER_REPO}/releases/download/v{version}/ketcher-standalone-{version}.zip"
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let version = env::var("KETCHER_VERSION").unwrap_or_else(|_| DEFAULT_VERSION.to_string());
    let ketcher_dir =
        PathBuf::from(env::var("KETCHER_DIR").unwrap_or_else(|_| DEFAULT_DIR.to_string()));
    let index_html = ketcher_dir.join("index.html");

    if index_html.is_file() {
        println!(
            "✓ Ketcher v{version} already present in {}",
            ketcher_dir.display()
        );
        println!("  (set KETCHER_VERSION to upgrade, then re-run)");
        return Ok(());
    }

    let url = env::var("KETCHER_URL").unwrap_or_else(|_| release_url(&version));
    println!("Downloading Ketcher v{version} from {url} ...");

    let client = Client::builder().build()?;
    let resp = client.get(&url).send()?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} fetching {url}", resp.status()).into());
    }
    // The standalone zip is ~115 MB; download it fully (CI runners have
    // enough memory) and let `zip` parse it without a temp file on disk.
    let bytes = resp.bytes()?;
    let total = bytes.len();
    println!("  downloaded {total} bytes");

    fs::create_dir_all(&ketcher_dir)?;
    let mut archive = ZipArchive::new(Cursor::new(bytes.to_vec()))?;

    // The standalone build extracts to a single top-level directory
    // (`standalone/` or `ketcher-standalone-<ver>/`) — strip it so assets land
    // directly under `public/assets/ketcher`.
    let mut top_levels: BTreeSet<String> = BTreeSet::new();
    for i in 0..archive.len() {
        let name = archive.by_index(i)?.name().to_string();
        if is_macos_junk(&name) {
            continue;
        }
        let Some(first) = name.split('/').next() else {
            continue;
        };
        if !first.is_empty() {
            top_levels.insert(first.to_string());
        }
    }
    let strip_prefix = if top_levels.len() == 1 {
        top_levels.into_iter().next()
    } else {
        None
    };

    let mut entries = 0u64;
    let mut skipped_bytes = 0u64;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Skip macOS zip metadata (binary `._` files / `__MACOSX/`) and the
        // unused entry bundles before extracting (no 87 MB written).
        if is_macos_junk(file.name()) {
            continue;
        }
        if is_unused_entry(file.name()) {
            skipped_bytes += file.size();
            continue;
        }

        // Strip the single top-level directory; guard against zip-slip
        // (defensive — ketcher's archive is trusted).
        let rel = strip_prefix
            .as_deref()
            .and_then(|prefix| file.name().strip_prefix(prefix).map(str::to_string))
            .unwrap_or_else(|| file.name().to_string());
        let rel = rel.trim_start_matches('/');
        if rel.is_empty() || rel.contains("..") || rel.starts_with('/') {
            continue;
        }

        let out_path = ketcher_dir.join(rel);
        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        let Some(parent) = out_path.parent() else {
            continue;
        };
        fs::create_dir_all(parent)?;
        let mut out = BufWriter::new(fs::File::create(&out_path)?);
        io::copy(&mut file, &mut out)?;
        out.flush()?;
        entries += 1;
    }

    println!("  extracted {entries} file(s) to {}", ketcher_dir.display());
    if skipped_bytes > 0 {
        println!("  skipped {skipped_bytes} bytes of unused entry bundles (closable/duo/popup)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_unused_entries() {
        // The ~29 MB unused "entry" bundles this editor never loads, plus their
        // license files — all share a file-name prefix.
        assert!(is_unused_entry("standalone/static/js/closable.5cead650.js"));
        assert!(is_unused_entry("standalone/static/js/duo.546fbaab.js"));
        assert!(is_unused_entry("standalone/static/js/popup.ec23766a.js"));
        assert!(is_unused_entry(
            "standalone/static/js/closable.5cead650.js.LICENSE.txt"
        ));
        assert!(is_unused_entry(
            "standalone/static/js/duo.546fbaab.js.LICENSE.txt"
        ));
        // Everything else (the single loaded bundle, chunks, css, index) stays.
        assert!(!is_unused_entry("standalone/static/js/main.cb80d824.js"));
        assert!(!is_unused_entry(
            "standalone/static/js/157.7de4e426.chunk.js"
        ));
        assert!(!is_unused_entry(
            "standalone/static/js/622.ed91acd0.chunk.js.LICENSE.txt"
        ));
        assert!(!is_unused_entry("standalone/index.html"));
        assert!(!is_unused_entry("standalone/static/css/main.9cca8bc6.css"));
        // Small mode-specific entry HTML/CSS are kept (only the 29 MB JS
        // bundles + their LICENSE files are dead weight).
        assert!(!is_unused_entry("standalone/duo.html"));
        assert!(!is_unused_entry(
            "standalone/static/css/closable.9cca8bc6.css"
        ));
        // macOS resource forks look like `._<name>` — not "entry bundles", but
        // junk the dioxus asset copier cannot read (handled by is_macos_junk).
        assert!(!is_unused_entry("standalone/._duo.546fbaab.js"));
    }

    #[test]
    fn classifies_macos_junk() {
        // The real ketcher zip ships these (it was archived on macOS).
        assert!(is_macos_junk("__MACOSX"));
        assert!(is_macos_junk("__MACOSX/standalone/._index.html"));
        assert!(is_macos_junk(
            "__MACOSX/standalone/static/js/._duo.546fbaab.js"
        ));
        assert!(is_macos_junk("__MACOSX/standalone/._asset-manifest.json"));
        // Used assets and (real) entry bundles are NOT macOS junk.
        assert!(!is_macos_junk("standalone/index.html"));
        assert!(!is_macos_junk("standalone/static/js/main.cb80d824.js"));
        assert!(!is_macos_junk("standalone/static/js/duo.546fbaab.js"));
    }

    #[test]
    fn release_url_points_at_github_releases() {
        assert_eq!(
            release_url("3.18.0"),
            "https://github.com/epam/ketcher/releases/download/v3.18.0/ketcher-standalone-3.18.0.zip"
        );
        assert_eq!(
            release_url("3.10.0"),
            "https://github.com/epam/ketcher/releases/download/v3.10.0/ketcher-standalone-3.10.0.zip"
        );
    }
}
