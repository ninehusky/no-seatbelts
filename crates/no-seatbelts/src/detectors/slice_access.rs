use rustc_hir::{ExprKind, intravisit::Visitor};
use rustc_middle::{
    mir::{
        AggregateKind, AssertKind, BasicBlock, Body, Operand, Place, RawPtrKind, Rvalue, Statement,
        StatementKind, Terminator, TerminatorKind, UnOp,
    },
    ty::{TyCtxt, TyKind},
};
use rustc_span::Span;
use std::{cell::RefCell, collections::HashSet};

use crate::{
    detectors::PanicDetector,
    diagnostics::{NoSeatbeltsDiag, PanicKind, Suggestion},
};

pub struct SliceDetector {
    pub bounds_check_spans: RefCell<HashSet<Span>>,
}

impl SliceDetector {
    pub fn new() -> Self {
        Self {
            bounds_check_spans: RefCell::new(HashSet::new()),
        }
    }
}

struct IndexFinder<'tcx, 'hir> {
    tcx: TyCtxt<'tcx>,
    needle: Span,
    result: Option<(Span, String)>,
    parents: Vec<&'hir rustc_hir::Expr<'hir>>,
}

impl<'tcx, 'hir> Visitor<'hir> for IndexFinder<'tcx, 'hir> {
    fn visit_expr(&mut self, expr: &'hir rustc_hir::Expr<'hir>) {
        self.parents.push(expr);

        if expr.span.contains(self.needle) {
            if let ExprKind::Index(base, index, _) = &expr.kind {
                let sm = self.tcx.sess.source_map();

                let Ok(base_snip) = sm.span_to_snippet(base.span) else {
                    self.parents.pop();
                    return;
                };
                let Ok(index_snip) = sm.span_to_snippet(index.span) else {
                    self.parents.pop();
                    return;
                };

                let replacement =
                    format!("unsafe {{ {}.get_unchecked({}) }}", base_snip, index_snip);

                // 🔑 choose span: if parent is &expr, replace parent
                let span = match self.parents.iter().rev().nth(1) {
                    Some(parent) if matches!(parent.kind, ExprKind::AddrOf(_, _, _)) => parent.span,
                    _ => expr.span,
                };

                self.result = Some((span, replacement));
                self.parents.pop();
                return;
            }
        }

        rustc_hir::intravisit::walk_expr(self, expr);
        self.parents.pop();
    }
}

fn find_enclosing_index_expr<'tcx>(
    tcx: TyCtxt<'tcx>,
    body: &Body<'tcx>,
    needle: Span,
) -> Option<(Span, String)> {
    let Some(local_id) = body.source.def_id().as_local() else {
        return None;
    };

    let hir_body = tcx.hir_body_owned_by(local_id);
    let expr = hir_body.value;

    let mut finder = IndexFinder {
        tcx,
        needle,
        result: None,
        parents: Vec::new(),
    };

    finder.visit_expr(expr);
    finder.result
}

impl PanicDetector for SliceDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &'tcx Body<'tcx>,
        terminator: &Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        None
    }

    fn detect_statement<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &Body<'tcx>,
        statement: &Statement<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let StatementKind::Assign(inner) = &statement.kind else {
            return None;
        };

        let (place, rvalue) = &**inner;

        let Rvalue::Aggregate(agg_kind, _) = rvalue else {
            return None;
        };

        let AggregateKind::Adt(adt_def, _, _, _, _) = agg_kind.as_ref() else {
            return None;
        };

        let def_id = adt_def;
        let path = tcx.def_path_str(def_id);

        // TODO: replace this with `sym` matching using `tcx.lang_items()`
        if matches!(
            path.as_str(),
            "core::ops::Range"
                | "core::ops::RangeFrom"
                | "core::ops::RangeTo"
                | "core::ops::RangeFull"
                | "core::ops::RangeInclusive"
                | "core::ops::RangeToInclusive"
        ) {
            let (parent_span, replacement) =
                find_enclosing_index_expr(tcx, body, statement.source_info.span)?;
            return Some(NoSeatbeltsDiag {
                span: parent_span,
                kind: PanicKind::BoundsCheck,
                suggestion: Some(Suggestion::ReplaceCall { replacement }),
            });
        }

        None
    }
}
