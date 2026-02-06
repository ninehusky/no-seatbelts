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
