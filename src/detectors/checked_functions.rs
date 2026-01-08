use rustc_middle::{
    mir::{Operand, TerminatorKind},
    ty::TyCtxt,
};

use crate::{
    detectors::PanicDetector,
    diagnostics::{NoSeatbeltsDiag, PanicKind},
    semantics::checked_calls::get_replacement,
};

pub struct CheckedFunctionDetector;

fn extract_receiver_snippet<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &rustc_middle::mir::Body<'tcx>,
    terminator: &rustc_middle::mir::Terminator<'tcx>,
) -> Option<String> {
    let rustc_middle::mir::TerminatorKind::Call { args, .. } = &terminator.kind else {
        return None;
    };

    // Method calls have the receiver as arg[0]
    let receiver = match &args.get(0)?.node {
        Operand::Copy(place) | Operand::Move(place) => place.local,
        Operand::Constant(c) => return Some(c.to_string()),
    };
    // We only handle locals for now

    let span = body.local_decls[receiver].source_info.span;

    // Filter out compiler-generated spans
    if span.is_dummy() {
        return None;
    }

    tcx.sess.source_map().span_to_snippet(span).ok()
}

impl PanicDetector for CheckedFunctionDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let TerminatorKind::Call { func, fn_span, .. } = &terminator.kind else {
            return None;
        };

        let (def_id, _) = func.const_fn_def()?;

        let receiver = extract_receiver_snippet(tcx, body, terminator);
        let suggestion = get_replacement(tcx, def_id, receiver)?;

        Some(NoSeatbeltsDiag {
            span: *fn_span,
            kind: PanicKind::CheckedFunction,
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
