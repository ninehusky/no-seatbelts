use crate::{
    docker::CompileConfig,
    workspace::{find_build_dir, find_eval_root},
};

pub fn run() -> anyhow::Result<()> {
    let cfg = CompileConfig {
        target: crate::docker::TargetArch::Thumbv7emNoneEabi,
        bin: None,
        release: true,
        exclude_std: true,
    };

    let elf_path = find_eval_root()?.join("benchmarks").join("nrf52840dk");

    // 4. Count the number of panics in the binary.
    let section_size_summary =
        crate::analysis::size::get_section_size_summary(&cfg.target, &elf_path, &elf_path)?;

    let function_summary =
        crate::analysis::functions::get_function_summary(&cfg.target, &elf_path, &elf_path)?;

    let panics = crate::analysis::functions::find_panics(function_summary.edited.functions.clone());

    let final_folder = find_build_dir()?.join("benchmarks").join("tock_board");
    std::fs::create_dir_all(&final_folder)?;
    std::fs::write(
        final_folder.join("section-size-summary.json"),
        serde_json::to_string_pretty(&section_size_summary)?,
    )?;

    // Add the JSON for the function summary.
    std::fs::write(
        final_folder.join("function-summary.json"),
        serde_json::to_string_pretty(&function_summary)?,
    )?;

    std::fs::write(
        final_folder.join("remaining_panics.json"),
        serde_json::to_string_pretty(&panics)?,
    )?;

    Ok(())
}
