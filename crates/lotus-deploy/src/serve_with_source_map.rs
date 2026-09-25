// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: Contributors to the lotus-explore-rs project

//! Run a development command and attach a source map to its debug WASM output.

use std::{
    collections::{BTreeMap, HashMap, hash_map::Entry},
    env,
    error::Error,
    fs,
    io::{self, Seek, SeekFrom, Write},
    ops::Range,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    thread,
    time::{Duration, SystemTime},
};

use gimli::{self, EndianSlice, LittleEndian};
use reqwest as _;
use serde_json::to_string;
use wasmparser::{Parser, Payload};
use zip as _;

const POLL_INTERVAL: Duration = Duration::from_millis(250);
const STABLE_POLLS: usize = 2;
const SOURCE_MAP_SECTION: &str = "sourceMappingURL";
const VLQ_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

type AppResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileStamp {
    len: u64,
    modified: Option<SystemTime>,
}

#[derive(Default)]
struct WatchState {
    last: Option<FileStamp>,
    pending: Option<(FileStamp, usize)>,
}

#[derive(Debug)]
struct WasmSections {
    code_offset: u64,
    debug: HashMap<String, Vec<u8>>,
    source_map_range: Option<Range<u64>>,
}

#[derive(Debug)]
struct Mapping {
    address: i64,
    source: String,
    line: i64,
    column: i64,
}

fn main() -> AppResult<()> {
    let (wasm_path, command) = parse_args(env::args().skip(1))?;
    let status = run_command(&command, &wasm_path)?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("command exited with {status}").into())
    }
}

fn parse_args<I>(args: I) -> AppResult<(PathBuf, Vec<String>)>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let Some(wasm_path) = args.next() else {
        return Err(usage().into());
    };
    if wasm_path.is_empty() {
        return Err("WASM path cannot be empty".into());
    }

    let mut command = Vec::new();
    let mut after_separator = false;
    for arg in args {
        if !after_separator && arg == "--" {
            after_separator = true;
        } else if after_separator {
            command.push(arg);
        } else {
            return Err(usage().into());
        }
    }

    if !after_separator || command.is_empty() {
        return Err(usage().into());
    }

    Ok((PathBuf::from(wasm_path), command))
}

fn run_command(command: &[String], wasm_path: &Path) -> AppResult<ExitStatus> {
    let executable = command
        .first()
        .ok_or_else(|| "command cannot be empty".to_owned())?;
    let mut child = Command::new(executable)
        .args(command.iter().skip(1))
        .spawn()?;
    let mut state = WatchState::default();

    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status);
        }
        if let Err(error) = watch_once(wasm_path, &mut state) {
            eprintln!("source-map: {error}");
        }
        thread::sleep(POLL_INTERVAL);
    }
}

fn watch_once(wasm_path: &Path, state: &mut WatchState) -> AppResult<()> {
    let Some(stamp) = file_stamp(wasm_path) else {
        state.last = None;
        state.pending = None;
        return Ok(());
    };

    if state.last == Some(stamp) {
        return Ok(());
    }

    let polls = match state.pending {
        Some((pending, count)) if pending == stamp => count.saturating_add(1),
        _ => 1,
    };
    state.pending = Some((stamp, polls));
    if polls < STABLE_POLLS {
        return Ok(());
    }
    state.pending = None;

    if let Err(error) = generate_source_map(wasm_path) {
        if is_missing_file(error.as_ref()) {
            return Ok(());
        }
        return Err(error);
    }
    state.last = file_stamp(wasm_path);
    Ok(())
}

fn is_missing_file(error: &(dyn Error + Send + Sync + 'static)) -> bool {
    error
        .downcast_ref::<io::Error>()
        .is_some_and(|error| error.kind() == io::ErrorKind::NotFound)
}

fn file_stamp(path: &Path) -> Option<FileStamp> {
    let metadata = fs::metadata(path).ok()?;

    Some(FileStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

fn generate_source_map(wasm_path: &Path) -> AppResult<()> {
    let (map_path, map_url) = map_paths(wasm_path)?;
    let bytes = fs::read(wasm_path)?;
    let sections = parse_wasm_sections(&bytes)?;
    let points = parse_points(&sections)?;
    let source_map = build_source_map(&points)?;
    fs::write(&map_path, source_map)?;
    patch_wasm(wasm_path, sections.source_map_range, &map_url)
}

fn parse_wasm_sections(bytes: &[u8]) -> AppResult<WasmSections> {
    let mut code_offset = None;
    let mut debug = HashMap::new();
    let mut source_map_range = None;

    for payload in Parser::new(0).parse_all(bytes) {
        match payload? {
            Payload::CodeSectionStart { range, .. } => code_offset = Some(range.start),
            Payload::CustomSection(section) => {
                let name = section.name().to_owned();
                if name == SOURCE_MAP_SECTION {
                    source_map_range = Some(section.range());
                } else if name.starts_with(".debug_") {
                    debug.insert(name, section.data().to_vec());
                }
            }
            _ => {}
        }
    }

    Ok(WasmSections {
        code_offset: code_offset.ok_or("WASM has no code section")?,
        debug,
        source_map_range,
    })
}

fn parse_points(sections: &WasmSections) -> AppResult<BTreeMap<i64, Mapping>> {
    let dwarf_sections = gimli::DwarfSections::load(|id| {
        Ok::<_, gimli::Error>(sections.debug.get(id.name()).cloned().unwrap_or_default())
    })?;
    let dwarf = dwarf_sections.borrow(|section| EndianSlice::new(section, LittleEndian));
    let mut points = BTreeMap::new();
    let mut units = dwarf.units();

    while let Some(header) = units.next()? {
        let unit = dwarf.unit(header)?;
        let Some(program) = unit.line_program.clone() else {
            continue;
        };
        let mut rows = program.rows();
        while let Some((line_header, row)) = rows.next_row()? {
            let line = match row.line() {
                Some(value) => i64::try_from(value.get())?,
                None => 0,
            };
            if line == 0 {
                continue;
            }
            let column = match row.column() {
                gimli::ColumnType::LeftEdge => 1,
                gimli::ColumnType::Column(value) => i64::try_from(value.get())?,
            };
            let mut address = row
                .address()
                .checked_add(sections.code_offset)
                .ok_or("WASM mapping address overflow")?;
            if row.end_sequence() {
                address = address
                    .checked_sub(1)
                    .ok_or("WASM mapping address underflow")?;
            }
            let address = i64::try_from(address)?;
            let mut source = PathBuf::new();
            if let Some(file) = row.file(line_header) {
                if file.directory_index() != 0
                    && let Some(directory) = file.directory(line_header)
                {
                    source.push(
                        dwarf
                            .attr_string(&unit, directory)?
                            .to_string_lossy()
                            .as_ref(),
                    );
                }
                source.push(
                    dwarf
                        .attr_string(&unit, file.path_name())?
                        .to_string_lossy()
                        .as_ref(),
                );
            }
            points.insert(
                address,
                Mapping {
                    address,
                    source: source.to_string_lossy().into_owned(),
                    line,
                    column,
                },
            );
        }
    }

    Ok(points)
}

fn build_source_map(points: &BTreeMap<i64, Mapping>) -> AppResult<String> {
    let mut sources = Vec::new();
    let mut source_ids = HashMap::new();
    let mut mappings = Vec::new();
    let mut last_address = 0;
    let mut last_source_id = 0_i64;
    let mut last_line = 1_i64;
    let mut last_column = 1_i64;

    for point in points.values() {
        let source_id = match source_ids.entry(point.source.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let source_id = sources.len();
                sources.push(point.source.clone());
                entry.insert(source_id);
                source_id
            }
        };
        let source_id = i64::try_from(source_id)?;
        mappings.push(format!(
            "{}{}{}{}",
            vlq_encode(point.address - last_address),
            vlq_encode(source_id - last_source_id),
            vlq_encode(point.line - last_line),
            vlq_encode(point.column - last_column)
        ));
        last_address = point.address;
        last_source_id = source_id;
        last_line = point.line;
        last_column = point.column;
    }

    let sources = to_string(&sources)?;
    let mappings = mappings.join(",");
    Ok(format!(
        "{{\"version\":3,\"names\":[],\"sources\":{sources},\"sourcesContent\":null,\"mappings\":\"{mappings}\"}}"
    ))
}

fn vlq_encode(value: i64) -> String {
    let mut value = if value >= 0 {
        value << 1
    } else {
        (-value << 1) + 1
    };
    let mut result = String::new();
    loop {
        let mut digit = usize::try_from(value & 31).unwrap_or(0);
        value >>= 5;
        if value != 0 {
            digit |= 32;
        }
        result.push(char::from(*VLQ_ALPHABET.get(digit).unwrap_or(&b'?')));
        if value == 0 {
            return result;
        }
    }
}

fn patch_wasm(path: &Path, source_map_range: Option<Range<u64>>, url: &str) -> AppResult<()> {
    let mut file = fs::OpenOptions::new().write(true).open(path)?;
    let file_len = file.metadata()?.len();
    let section_start = match source_map_range {
        Some(range) if range.end == file_len => range.start,
        Some(_) => return Err("sourceMappingURL is not the final WASM section".into()),
        None => file_len,
    };
    file.set_len(section_start)?;
    file.seek(SeekFrom::Start(section_start))?;

    let name = SOURCE_MAP_SECTION.as_bytes();
    let mut content = encode_uleb(name.len());
    content.extend_from_slice(name);
    content.extend(encode_uleb(url.len()));
    content.extend_from_slice(url.as_bytes());

    let mut section = encode_uleb(0);
    section.extend(encode_uleb(content.len()));
    section.extend(content);
    file.write_all(&section)?;
    Ok(())
}

fn encode_uleb(mut value: usize) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = u8::try_from(value & 0x7f).unwrap_or(0);
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            return bytes;
        }
    }
}

fn map_paths(wasm_path: &Path) -> AppResult<(PathBuf, String)> {
    let file_name = wasm_path
        .file_name()
        .ok_or_else(|| "WASM path has no file name".to_owned())?;
    let mut map_name = file_name.to_os_string();
    map_name.push(".map");
    let map_path = wasm_path.with_file_name(&map_name);
    let map_url = map_name
        .into_string()
        .map_err(|_| "WASM file name is not valid UTF-8".to_owned())?;
    Ok((map_path, map_url))
}

fn usage() -> String {
    "usage: serve-with-source-map <path-to-wasm> -- <command> [args...]".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_relative_map_paths() {
        let result = map_paths(Path::new("target/app_bg.wasm"));
        assert!(result.is_ok());
        if let Ok((map_path, map_url)) = result {
            assert_eq!(map_path, PathBuf::from("target/app_bg.wasm.map"));
            assert_eq!(map_url, "app_bg.wasm.map");
        }
    }

    #[test]
    fn parses_wrapped_command() {
        let result = parse_args([
            "target/app_bg.wasm".to_owned(),
            "--".to_owned(),
            "dx".to_owned(),
            "serve".to_owned(),
        ]);
        assert!(result.is_ok());
        if let Ok((path, command)) = result {
            assert_eq!(path, PathBuf::from("target/app_bg.wasm"));
            assert_eq!(command, vec!["dx", "serve"]);
        }
    }

    #[test]
    fn rejects_missing_command_separator() {
        assert!(parse_args(["target/app_bg.wasm".to_owned()]).is_err());
    }
}
