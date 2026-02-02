#![feature(rustc_private)]

use rustc_driver::Compilation;
use rustc_lint::{LateLintPass, LintContext, LintPass};
use rustc_middle::mir::Body;
use rustc_session::{declare_lint, declare_lint_pass};

extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_interface;
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
    detectors: Vec<Box<dyn detectors::PanicDetector + Send>>,
}

impl PanicPass {
    pub fn new(detectors: Vec<Box<dyn detectors::PanicDetector + Send>>) -> Self {
        Self { detectors }
    }

    pub fn check_body<'tcx>(&self, tcx: &rustc_middle::ty::TyCtxt<'tcx>, body: &'tcx Body<'tcx>) {
        if !tcx.is_mir_available(body.source.def_id()) {
            return;
        }
        for bb in tcx.optimized_mir(body.source.def_id()).basic_blocks.iter() {
            let terminator = &bb.terminator;
            if terminator.is_none() {
                continue;
            }
            let terminator = terminator.as_ref().unwrap();

            for detector in &self.detectors {
                if let Some(diag) = detector.detect_terminator(*tcx, body, terminator) {
                    let hir_id = tcx.local_def_id_to_hir_id(body.source.def_id().expect_local());

                    tcx.emit_node_span_lint(PANIC_PASS, hir_id, diag.span, diag);
                }
            }

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
        }
    }
}

struct PanicLatePass;

impl LintPass for PanicLatePass {
    fn name(&self) -> &'static str {
        "PanicLatePass"
    }

    fn get_lints(&self) -> rustc_lint::LintVec {
        rustc_lint::LintVec::from_iter(vec![PANIC_PASS])
    }
}

impl<'tcx> LateLintPass<'tcx> for PanicLatePass {
    fn check_body(&mut self, _: &rustc_lint::LateContext<'tcx>, body: &rustc_hir::Body<'tcx>) {}
}

impl rustc_driver::Callbacks for PanicPass {
    fn config(&mut self, config: &mut rustc_interface::interface::Config) {
        config.register_lints = Some(Box::new(|_sess, lint_store| {
            lint_store.register_lints(&[&PANIC_PASS]);
            lint_store.register_late_pass(|_| Box::new(PanicLatePass));
        }));
    }

    fn after_expansion<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        _tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        // Side-channel proof that doesn't depend on cargo output:
        Compilation::Continue
    }

    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ) -> rustc_driver::Compilation {
        // REAL compilation — run no-seatbelts here

        for free_id in tcx.hir_crate_items(()).free_items() {
            let item = tcx.hir_item(free_id);
            if let rustc_hir::ItemKind::Fn { .. } = item.kind {
                let def_id = item.hir_id().owner.def_id;
                if tcx.hir_maybe_body_owned_by(def_id).is_some() {
                    let body = tcx.optimized_mir(def_id.to_def_id());
                    self.check_body(&tcx, body);
                }
            }
        }

        for impl_id in tcx.hir_crate_items(()).impl_items() {
            let impl_item = tcx.hir_impl_item(impl_id);
            if let rustc_hir::ImplItemKind::Fn { .. } = impl_item.kind {
                let def_id = impl_item.hir_id().owner.def_id;
                if tcx.hir_maybe_body_owned_by(def_id).is_some() {
                    let body = tcx.optimized_mir(def_id.to_def_id());
                    self.check_body(&tcx, body);
                }
            }
        }
        rustc_driver::Compilation::Continue
    }
}
