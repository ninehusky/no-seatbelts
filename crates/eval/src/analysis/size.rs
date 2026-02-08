use std::path::Path;

use anyhow::Context;
use chrono::format::parse;
use serde::{Deserialize, Serialize};

use crate::{
    docker::{TargetArch, run_in_docker, to_container_path},
    workspace::find_eval_root,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSizeAnalysis {
    pub baseline: SectionSizes,
    pub edited: SectionSizes,
    pub delta: SectionSizeDeltas,
}

pub type SectionSizes = std::collections::BTreeMap<String, u64>;
pub type SectionSizeDeltas = std::collections::BTreeMap<String, i64>;

/// Compute the section size deltas between the baseline and edited versions.
pub fn get_delta(baseline: &SectionSizes, edited: &SectionSizes) -> SectionSizeDeltas {
    let mut delta: SectionSizeDeltas = Default::default();
    for (section, baseline_size) in baseline.iter() {
        let edited_size = edited.get(section).copied().unwrap_or(0);
        delta.insert(
            section.clone(),
            (edited_size as i64 - (*baseline_size as i64)) as i64,
        );
    }
    delta
}

pub fn get_section_size_summary(
    _target: &TargetArch,
    baseline_elf_path: &Path,
    edited_elf_path: &Path,
) -> anyhow::Result<SectionSizeAnalysis> {
    let baseline_sizes = get_section_sizes(baseline_elf_path)?;
    let edited_sizes = get_section_sizes(edited_elf_path)?;

    let delta = get_delta(&baseline_sizes, &edited_sizes);

    Ok(SectionSizeAnalysis {
        baseline: baseline_sizes,
        edited: edited_sizes,
        delta,
    })
}
fn get_section_sizes(elf: &Path) -> anyhow::Result<SectionSizes> {
    let elf_path = to_container_path(&find_eval_root()?, elf);
    let output = run_in_docker(&find_eval_root()?, &["llvm-readelf", "-S", &elf_path])?;

    parse_readelf_sections(&String::from_utf8_lossy(&output.stdout))
}

fn parse_readelf_sections(output: &str) -> anyhow::Result<SectionSizes> {
    let mut sections = std::collections::BTreeMap::new();

    let mut name_idx = None;
    let mut size_idx = None;
    for line in output.lines() {
        let line = line.trim();
        // Skip anything that doesn't look like a section line.
        if !line.starts_with('[') {
            continue;
        }

        if line.starts_with("[Nr]") {
            // Initialize column indices based on header line.
            // We do `i - 1` to account for the fact we strip away the leading [ Nr] token.
            let col_names = line.split_whitespace().collect::<Vec<_>>();
            for (i, col) in col_names.iter().enumerate() {
                if *col == "Name" {
                    name_idx = Some(i - 1);
                } else if *col == "Size" {
                    size_idx = Some(i - 1);
                }
            }
            continue;
        }

        if name_idx.is_none() || size_idx.is_none() {
            anyhow::bail!("failed to find Name or Size column in llvm-readelf output");
        }

        let name_idx = name_idx.unwrap();
        let size_idx = size_idx.unwrap();

        // Example lines:
        // [ 0]                   NULL            00000000 000000 000000 00      0   0  0
        // [ 1] .interp           PROGBITS        00000194 000194 000013 00   A  0   0  1
        // first, normalize the [ 0] or [10] prefix into a single token, so the rest of the fields shift into consistent positions.
        let parts: Vec<&str> = line
            .split_whitespace()
            .filter(|s| !s.contains('[') && !s.contains(']'))
            .collect();

        let name = parts[name_idx];
        let size = u64::from_str_radix(parts[size_idx], 16)
            .with_context(|| format!("failed to parse section size (hex) in line: {}", line))?;

        sections.insert(name.to_string(), size);
    }

    Ok(sections)
}
