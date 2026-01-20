use rustc_middle::{
    mir::{Operand, TerminatorKind},
    ty::TyCtxt,
};
use rustc_span::Span;

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
) -> Option<(Span, String)> {
    let rustc_middle::mir::TerminatorKind::Call { args, .. } = &terminator.kind else {
        return None;
    };

    // Method calls have the receiver as arg[0]
    let receiver = match &args.first()?.node {
        Operand::Copy(place) | Operand::Move(place) => place.local,
        Operand::Constant(_) => return None,
    };
    // We only handle locals for now

    let span = body.local_decls[receiver].source_info.span;

    // Filter out compiler-generated spans
    // if span.is_dummy() {
    //     return None;
    // }

    Some((span, tcx.sess.source_map().span_to_snippet(span).ok()?))
}

impl PanicDetector for CheckedFunctionDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &rustc_middle::mir::Body<'tcx>,
        terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let TerminatorKind::Call { func, args, .. } = &terminator.kind else {
            return None;
        };

        let (def_id, _) = func.const_fn_def()?;

        let (_recv_span, recv_str) = extract_receiver_snippet(tcx, body, terminator)?;
        let call_span = terminator.source_info.span.source_callsite();
        let args_as_strs = args
            .iter()
            .map(|arg| tcx.sess.source_map().span_to_snippet(arg.span).ok())
            .collect();

        let suggestion = get_replacement(tcx, def_id, Some(recv_str), args_as_strs)?;

        Some(NoSeatbeltsDiag {
            span: call_span,
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
