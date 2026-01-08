use rustc_middle::{
    mir::TerminatorKind,
    ty::TyCtxt,
};

use crate::{
    detectors::PanicDetector,
    diagnostics::{NoSeatbeltsDiag, PanicKind, Suggestion},
};


pub struct ExplicitPanicDetector;

impl PanicDetector for ExplicitPanicDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let TerminatorKind::Call { func, .. } = &terminator.kind else {
            return None;
        };

        let (def_id, _) = func.const_fn_def()?;

        if tcx.lang_items().panic_fn()? == def_id {
            let call_span = terminator.source_info.span.source_callsite();
            let sm = tcx.sess.source_map();

            return Some(NoSeatbeltsDiag {
                span: call_span,
                kind: PanicKind::CheckedFunction,
                suggestion: Some(Suggestion::ReplaceCall {
                    replacement: "unsafe { std::hint::unreachable_unchecked() }".to_string(),
                }),
            });
        }

        None
    }

    fn detect_statement<'tcx>(
        &self,
        _tcx: rustc_middle::ty::TyCtxt<'tcx>,
        _body: &rustc_middle::mir::Body<'tcx>,
        _statement: &rustc_middle::mir::Statement<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        None
    }
}
