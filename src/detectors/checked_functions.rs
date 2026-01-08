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

use rustc_hir::{
    Expr, ExprKind,
    intravisit::{Visitor, walk_expr},
};

struct MethodCallFinder {
    call_span: Span,
    result: Option<Span>,
}

impl<'hir> Visitor<'hir> for MethodCallFinder {
    fn visit_expr(&mut self, expr: &'hir Expr<'hir>) {
        if let ExprKind::MethodCall(..) = expr.kind {
            if expr.span.contains(self.call_span) {
                // Pick the smallest enclosing span
                if self
                    .result
                    .map_or(true, |s| expr.span.hi() - expr.span.lo() < s.hi() - s.lo())
                {
                    self.result = Some(expr.span);
                }
            }
        }

        walk_expr(self, expr);
    }
}

pub struct CheckedFunctionDetector;

fn full_method_call_span_from_hir(
    tcx: TyCtxt<'_>,
    def_id: rustc_hir::def_id::LocalDefId,
    call_span: Span,
) -> Option<Span> {
    let body = tcx.hir_body_owned_by(def_id);

    let mut finder = MethodCallFinder {
        call_span,
        result: None,
    };

    finder.visit_body(body);
    finder.result
}

fn extract_receiver_snippet<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &rustc_middle::mir::Body<'tcx>,
    terminator: &rustc_middle::mir::Terminator<'tcx>,
) -> Option<(Span, String)> {
    let rustc_middle::mir::TerminatorKind::Call { args, .. } = &terminator.kind else {
        return None;
    };

    // Method calls have the receiver as arg[0]
    let receiver = match &args.get(0)?.node {
        Operand::Copy(place) | Operand::Move(place) => place.local,
        Operand::Constant(c) => return None,
    };
    // We only handle locals for now

    let span = full_method_call_span_from_hir(
        tcx,
        body.source.def_id().as_local()?,
        body.local_decls[receiver].source_info.span,
    )?;

    // Filter out compiler-generated spans
    if span.is_dummy() {
        return None;
    }

    Some((span, tcx.sess.source_map().span_to_snippet(span).ok()?))
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

        let (recv_span, recv_str) = extract_receiver_snippet(tcx, body, terminator)?;
        let suggestion = get_replacement(tcx, def_id, Some(recv_str))?;

        Some(NoSeatbeltsDiag {
            span: recv_span,
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
