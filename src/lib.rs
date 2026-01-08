#![feature(rustc_private)]

use rustc_middle::mir::Body;
use rustc_session::declare_lint;

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod detectors;
mod diagnostics;
mod semantics;

pub use detectors::*;

declare_lint! {
    /// Suggests replacing panicking functions with their unsafe counterparts.
    pub PANIC_PASS,
    Warn,
    "Suggests replacing panic-checking code with its unsafe counterpart."
}

pub struct PanicPass {
    detectors: Vec<Box<dyn detectors::PanicDetector>>,
}

impl<'tcx> PanicPass {
    pub fn new(detectors: Vec<Box<dyn detectors::PanicDetector>>) -> Self {
        Self { detectors }
    }

    pub fn check_body(&self, tcx: &rustc_middle::ty::TyCtxt<'tcx>, body: &Body<'tcx>) {
        if !tcx.is_mir_available(body.source.def_id()) {
            return;
        }
        for bb in tcx.optimized_mir(body.source.def_id()).basic_blocks.iter() {
            for statement in &bb.statements {
                for detector in &self.detectors {
                    if let Some(diag) = detector.detect_statement(*tcx, body, statement) {
                        let hir_id =
                            tcx.local_def_id_to_hir_id(body.source.def_id().expect_local());
                        let span = statement.source_info.span;

                        tcx.emit_node_span_lint(PANIC_PASS, hir_id, span, diag);
                    }
                }
            }
            let terminator = &bb.terminator;
            if terminator.is_none() {
                continue;
            }
            let terminator = terminator.as_ref().unwrap();

            for detector in &self.detectors {
                if let Some(diag) = detector.detect_terminator(*tcx, body, terminator) {
                    let hir_id = tcx.local_def_id_to_hir_id(body.source.def_id().expect_local());
                    let span = terminator.source_info.span;

                    tcx.emit_node_span_lint(PANIC_PASS, hir_id, span, diag);
                }
            }
        }
    }
}
