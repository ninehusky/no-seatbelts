pub mod functions;
pub mod size;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeAnalysis {
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
