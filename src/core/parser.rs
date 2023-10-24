use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use std::fs;

use anyhow::{anyhow, Context, Result};

use super::vlq::vlq_decode;

pub fn parse_file_by_path(path: &str) -> Result<SourceMapping> {
    let contents = fs::read_to_string(path)?;

    let last_line = contents
        .lines()
        .rev()
        .next()
        .ok_or_else(|| anyhow!("File {path} is empty."))?;

    let raw_source_mapping = parse_raw_source_mapping(path, last_line)?;

    SourceMapping::from_raw(raw_source_mapping)
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
    if line.starts_with("//# sourceMappingURL=data:application/json;") {
        // base64 encoded source map
        let lookup = "base64,";
        let index = line
            .find(lookup)
            .ok_or_else(|| anyhow!("File {path} does not contain base64 sourcemap."))?;
        let (_, base64_value) = line.split_at(index + lookup.len());

        let base64_decoded = general_purpose::STANDARD
            .decode(base64_value)
            .with_context(|| anyhow!("File {path} contains invalid base64 sourcemap."))?;

        let base64_str = String::from_utf8_lossy(&base64_decoded);

        let raw_source_mapping: RawSourceMapping = serde_json::from_str(&base64_str)?;

        return Ok(raw_source_mapping);
    }

    Err(anyhow!(
        "Sorry, this format is not supported at the moment: {}",
        line.chars().take(100).collect::<String>(),
    ))
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
}

impl Default for Mapping {
    fn default() -> Self {
        Self::const_default()
    }
}

static EMPTY_MAPPING: Mapping = Mapping::const_default();

#[derive(Debug)]
pub struct SourceMapping {
    file: String,
    source_root: Option<String>,
    sources: Vec<String>,
    names: Vec<String>,
    mappings: Vec<Mapping>,
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

    fn from_raw(raw_mapping: RawSourceMapping) -> Result<Self> {
        let mut mappings: Vec<Mapping> = vec![];

        for (gen_line, generated_line_mapping) in raw_mapping.mappings.split(';').enumerate() {
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

        Ok(SourceMapping {
            file: raw_mapping.file,
            source_root: raw_mapping.source_root,
            sources: raw_mapping.sources,
            names: raw_mapping.names,
            mappings,
        })
    }
}
