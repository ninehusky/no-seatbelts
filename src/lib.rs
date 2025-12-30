#![feature(rustc_private)]

use rustc_errors::LintDiagnostic;
use rustc_fluent_macro::fluent_messages;
use rustc_macros::{Diagnostic, LintDiagnostic, Subdiagnostic};
use rustc_middle::{mir::{Body, TerminatorKind}, ty::TyCtxt};
use rustc_session::declare_lint;
use rustc_span::Span;

use crate::diagnostics::{NoSeatbeltsDiag, PanicKind, Suggestion};

extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod diagnostics;

declare_lint! {
    /// Suggests replacing panicking functions with their unsafe counterparts.
    pub UNCHECKED_UNWRAP,
    Warn,
    "suggest replacing `unwrap` with `unwrap_unchecked` to avoid panics"
}


use rustc_span::SpanSnippetError;

fn make_unwrap_replacement(
    tcx: TyCtxt<'_>,
    span: Span,
) -> Option<String> {
    let sm = tcx.sess.source_map();
    let snippet = sm.span_to_snippet(span).ok()?;

    // some.unwrap() → unsafe { some.unwrap_unchecked() }
    let new_call = snippet.replace(
        "unwrap()",
        "unwrap_unchecked()",
    );

    Some(format!("unsafe {{ {} }}", new_call))
}


pub struct UncheckedFunctionPass {}

impl<'tcx> UncheckedFunctionPass {
    pub fn check_body(&self, tcx: &rustc_middle::ty::TyCtxt<'tcx>, body: &Body<'tcx>) {
        if !tcx.is_mir_available(body.source.def_id()) {
            return;
        }
        for bb in tcx.optimized_mir(body.source.def_id()).basic_blocks.iter() {
            let terminator = &bb.terminator;
            if terminator.is_none() {
                continue;
            }
            let terminator = terminator.as_ref().unwrap();

            match &terminator.kind {
                TerminatorKind::Call { func, .. } => {
                    if let Some((def_id, _)) = func.const_fn_def() {
                        let called_path = tcx.def_path_str(def_id);
                        // look for `unwrap`.
                        let name = called_path.split("::").last().unwrap();
                        if name == "unwrap" {
                            // emit a lint warning here suggesting `unwrap` -> `unwrap_unchecked`.
                            let hir_id =
                                tcx.local_def_id_to_hir_id(body.source.def_id().expect_local());
                            let span = terminator.source_info.span;

                            if let Some(replacement) = make_unwrap_replacement(*tcx, span) {
                            tcx.emit_node_span_lint(
                                UNCHECKED_UNWRAP,
                                hir_id,
                                span,
                                NoSeatbeltsDiag {
                                    span,
                                    kind: PanicKind::Unwrap,
                                    suggestion: Some(Suggestion::ReplaceCall { replacement }),
                                }
                            );
                        }
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
