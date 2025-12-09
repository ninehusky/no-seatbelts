#![feature(rustc_private)]

use rustc_errors::LintDiagnostic;
use rustc_fluent_macro::fluent_messages;
use rustc_macros::{Diagnostic, LintDiagnostic, Subdiagnostic};
use rustc_middle::mir::{Body, TerminatorKind};
use rustc_session::declare_lint;
use rustc_span::Span;

use crate::fluent_generated::no_seatbelts_boo_you_stink;

extern crate rustc_errors;
extern crate rustc_fluent_macro;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_macros;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

declare_lint! {
    /// Suggests replacing panicking functions with their unsafe counterparts.
    pub UNCHECKED_UNWRAP,
    Warn,
    "suggest replacing `unwrap` with `unwrap_unchecked` to avoid panics"
}

// **Important:** Use CARGO_MANIFEST_DIR to make the path absolute
fluent_messages! { "/Users/acheung/research/projects/no-seatbelts/locales/en-US.ftl" }

// This macro should handle everything, but it doesn't.
// #[derive(LintDiagnostic)]
// #[diag(no_seatbelts_boo_you_stink)]

pub struct UncheckedUnwrapSugg {
    // no fields needed for a simple message
    pub span: Span,
}

impl<'a> rustc_errors::LintDiagnostic<'a, ()> for UncheckedUnwrapSugg {
    fn decorate_lint<'b>(self, diag: &'b mut rustc_errors::Diag<'a, ()>) {
        // let my_debug_msg: rustc_errors::DiagMessage = rustc_errors::DiagMessage::FluentIdentifier(
        //     std::borrow::Cow::Borrowed("no_seatbelts_boo_you_stink"),
        //     None,
        // );

        diag.primary_message("Consider using `unwrap_unchecked` if you are sure the value is `Some`.");
    }
}

// Now, define a lint pass to implement the above lint.

/// Suggests rewriting code which panics to unsafe
/// variants which exhibit undefined behavior.
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
                            let msg = no_seatbelts_boo_you_stink;
                            println!("message: {:?}", msg);
                            tcx.emit_node_span_lint(
                                UNCHECKED_UNWRAP,
                                hir_id,
                                span,
                                UncheckedUnwrapSugg {
                                    span,

                                },
                            );
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
