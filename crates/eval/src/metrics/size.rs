use std::{fs, path::Path};

use crate::project::find_elf;

pub fn get_size(crate_root: &Path) -> u64 {
    let elf_path = find_elf(crate_root);
    let metadata = fs::metadata(elf_path).expect("failed to get ELF metadata");
    metadata.len()
}
