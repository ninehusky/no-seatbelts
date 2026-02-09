use serde::{Deserialize, Serialize};

pub mod no_seatbelts;

/// The transformations to apply to the edited project, if any.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditMode {
    None,
    NoSeatbelts,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditSummary {
    pub edit_mode: EditMode,
    pub suggestions: Vec<String>,
}

impl Default for EditSummary {
    fn default() -> Self {
        EditSummary {
            edit_mode: EditMode::None,
            suggestions: vec![],
        }
    }
}
