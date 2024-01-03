use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::{fs, path::Path};

use anyhow::{anyhow, Context, Result};

use super::vlq::vlq_decode;

pub fn parse_file_by_path(path: &str) -> Result<(String, SourceMapping)> {
    let file_meta = std::fs::metadata(path)?;
    let contents = fs::read_to_string(path)?;

    let last_line = contents
        .lines()
        .rev()
        .next()
        .ok_or_else(|| anyhow!("File {path} is empty."))?;

    let mut raw_source_mapping = parse_raw_source_mapping(path, last_line)?;
    raw_source_mapping.file = String::from(path); // TODO

    let mut source_mapping = SourceMapping::from_raw(raw_source_mapping)?;
    source_mapping.source_file_len = file_meta.len();
    source_mapping.source_map_len = last_line.len() as u64;

    Ok((contents, source_mapping))
}

#[allow(dead_code)]
#[derive(Default, Deserialize, Debug)]
struct RawSourceMapping {
    file: String,
    source_root: Option<String>,
    sources: Vec<String>,
    names: Vec<String>,
    mappings: String,
}

fn parse_raw_source_mapping(path: &str, line: &str) -> Result<RawSourceMapping> {
    let line_stripped = line.trim_start_matches("//# sourceMappingURL=");

    if line_stripped.len() == line.len() {
        return Err(anyhow!(
            "Unsupported format: {}",
            line.chars().take(100).collect::<String>(),
        ));
    }

    let json_str = if line_stripped.starts_with("data:application/json;") {
        // base64 encoded source map
        let lookup = "base64,";
        let index = line
            .find(lookup)
            .ok_or_else(|| anyhow!("File {path} does not contain base64 sourcemap."))?;
        let (_, base64_value) = line.split_at(index + lookup.len());

        let base64_decoded = general_purpose::STANDARD
            .decode(base64_value)
            .with_context(|| anyhow!("File {path} contains invalid base64 sourcemap."))?;

        String::from_utf8_lossy(&base64_decoded).into_owned()
    } else {
        let path = Path::new(path);
        let parent = path.parent().unwrap();
        let map_path = parent.join(line_stripped);

        fs::read_to_string(map_path)?
    };

    let raw_source_mapping: RawSourceMapping = serde_json::from_str(&json_str)?;

    return Ok(raw_source_mapping);
}

#[derive(Debug)]
pub struct Mapping {
    gen_line: u32,
    gen_column: u32,
    src_file: u32,
    src_line: u32,
    src_column: u32,
}

impl Mapping {
    const fn const_default() -> Self {
        Self {
            gen_line: 0,
            gen_column: 0,
            src_file: 0,
            src_line: 0,
            src_column: 0,
        }
    }

    pub fn gen_line(&self) -> u32 {
        self.gen_line
    }

    pub fn gen_column(&self) -> u32 {
        self.gen_column
    }

    pub fn src_file(&self) -> u32 {
        self.src_file
    }

    pub fn src_line(&self) -> u32 {
        self.src_line
    }

    pub fn src_column(&self) -> u32 {
        self.src_column
    }
}

impl Default for Mapping {
    fn default() -> Self {
        Self::const_default()
    }
}

pub static EMPTY_MAPPING: Mapping = Mapping::const_default();

#[derive(Debug)]
pub struct SourceMapping {
    file: String,
    source_root: Option<String>,
    sources: Vec<String>,
    names: Vec<String>,
    mappings: Vec<Mapping>,
    // Field not present in source JSON, but read early to split presentation logic from
    // parsing and analyzing logic
    source_file_len: u64,
    // Field not present in source JSON, but needed for presenting meaningful results
    source_map_len: u64,
    pub file_name: String,
}

impl SourceMapping {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn source_root(&self) -> Option<&str> {
        self.source_root.as_ref().map(|v| v.as_str())
    }

    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn mappings(&self) -> &[Mapping] {
        &self.mappings
    }

    pub fn source_file_len(&self) -> u64 {
        self.source_file_len
    }

    pub fn source_map_len(&self) -> u64 {
        self.source_map_len
    }

    pub fn source_file_without_source_map_len(&self) -> u64 {
        self.source_file_len - self.source_map_len
    }

    fn from_raw(raw_mapping: RawSourceMapping) -> Result<Self> {
        let mut mappings: Vec<Mapping> = vec![];

        for (gen_line, generated_line_mapping) in raw_mapping.mappings.split(';').enumerate() {
            if generated_line_mapping.is_empty() {
                continue;
            }

            let mut line_prev_column = 0i32;

            for term_mapping in generated_line_mapping.split(',') {
                let raw_mapping = vlq_decode(term_mapping)?;
                let prev_mapping = mappings.last().unwrap_or(&EMPTY_MAPPING);

                let mapping = Mapping {
                    gen_line: gen_line as u32,
                    gen_column: (raw_mapping[0] + line_prev_column) as u32,
                    src_file: (raw_mapping[1] + prev_mapping.src_file as i32) as u32,
                    src_line: (raw_mapping[2] + prev_mapping.src_line as i32) as u32,
                    src_column: (raw_mapping[3] + prev_mapping.src_column as i32) as u32,
                };

                line_prev_column = mapping.gen_column as i32;

                mappings.push(mapping);
            }
        }

        let file_name = match raw_mapping.file.rfind('/') {
            Some(pos) => raw_mapping.file.get((pos + 1)..).unwrap_or(&raw_mapping.file),
            None => &raw_mapping.file,
        }
        .to_string();

        Ok(SourceMapping {
            file: raw_mapping.file,
            source_root: raw_mapping.source_root,
            sources: raw_mapping.sources,
            names: raw_mapping.names,
            mappings,
            source_file_len: 0,
            source_map_len: 0,
            file_name,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty() && self.mappings.is_empty()
    }

    pub fn get_sources_root(&self) -> String {
        match self.source_root() {
            Some(path) if !path.is_empty() => return path.to_owned(),
            _ => {}
        }

        // This looks like crap
        let relative_jumps = self
            .sources()
            .first()
            .unwrap()
            .split('/')
            .take_while(|part| part == &"..")
            .count();

        // TODO: This looks like crap even more
        self.file()
            .split('/')
            .rev()
            .skip((relative_jumps + 1) as usize)
            .collect::<Vec<&str>>()
            .into_iter()
            .rev()
            .collect::<Vec<&str>>()
            .join("/")
    }
}
