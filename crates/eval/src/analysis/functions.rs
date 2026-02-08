use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::docker::TargetArch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSummaryAnalysis {
    pub baseline: Vec<FunctionInfo>,
    pub edited: Vec<FunctionInfo>,
    pub delta: Vec<FunctionDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub size_bytes: u64,
    pub num_calls: u64,
    pub panic_calls: Vec<PanicCallInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDelta {
    pub name: String,
    pub size_bytes_delta: i64,
    pub num_calls_delta: i64,
    pub panic_calls_delta: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicCallInfo {
    pub caller: String,
    pub callee: String,
    pub asm: String,
}

pub fn get_function_sizes(target: &TargetArch, elf_path: &Path) -> FunctionSummaryAnalysis {
    todo!()
}
