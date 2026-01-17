use std::{fs, path::Path};

use crate::TARGET;

pub fn get_size(crate_root: &Path) -> u64 {
    let target_dir = crate_root
        .join("target")
        .join(TARGET)
        .join("release")
        .join("deps");
    let mut total_size = 0;
    for entry in fs::read_dir(target_dir).expect("failed to read target/release/deps") {
        let entry = entry.expect("failed to read entry");
        if !entry
            .file_name()
            .to_str()
            .unwrap()
            .contains("ring_buffer_smoketest")
        {
            continue;
        }
        // make sure no file extension
        if entry.path().extension().is_some() {
            continue;
        }
        let metadata = entry.metadata().expect("failed to get metadata");
        total_size += metadata.len();
    }
    if total_size == 0 {
        panic!(
            "failed to find binary in target/release/deps. Are you calling get_size on ring_buffer_smoketest?"
        );
    }
    total_size
}
