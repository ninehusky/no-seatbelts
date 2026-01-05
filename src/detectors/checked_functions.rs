use rustc_middle::mir::TerminatorKind;
use rustc_span::sym;

use crate::{
    detectors::PanicDetector,
    diagnostics::{NoSeatbeltsDiag, PanicKind, Suggestion},
    semantics::checked_calls::get_replacement,
};

pub struct CheckedFunctionDetector;

impl PanicDetector for CheckedFunctionDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let TerminatorKind::Call { func, fn_span, .. } = &terminator.kind else {
            return None;
        };

        let (def_id, _) = func.const_fn_def()?;

        let suggestion = get_replacement(tcx, def_id)?;

        Some(NoSeatbeltsDiag {
            span: *fn_span,
            kind: PanicKind::Unwrap,
            suggestion: Some(suggestion),
        })
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
