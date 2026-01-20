use rustc_middle::{
    mir::{AssertKind, TerminatorKind},
    ty::TyCtxt,
};
use rustc_span::Span;
use std::{cell::RefCell, collections::HashSet};

use crate::{
    detectors::PanicDetector,
    diagnostics::{NoSeatbeltsDiag, PanicKind, Suggestion},
};

pub struct ArrayAccessDetector {
    // spans of BoundsCheck asserts (recorded in detect_terminator)
    bounds_check_spans: RefCell<HashSet<Span>>,

    // spans of statements we already emitted a diagnostic for
    emitted_bounds_spans: RefCell<HashSet<(u32, u32)>>,
}

impl Default for ArrayAccessDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ArrayAccessDetector {
    pub fn new() -> Self {
        Self {
            bounds_check_spans: RefCell::new(HashSet::new()),
            emitted_bounds_spans: RefCell::new(HashSet::new()),
        }
    }
}

fn span_key(s: Span) -> (u32, u32) {
    (s.lo().0, s.hi().0)
}

#[derive(Copy, Clone)]
enum IndexCtx {
    Value,  // arr[i]
    Ref,    // &arr[i]
    RefMut, // &mut arr[i]
}

fn rewrite_core_index(base: &str, idx: &str, ctx: IndexCtx) -> String {
    match ctx {
        IndexCtx::Value => format!("unsafe {{ *{}.get_unchecked({}) }}", base, idx),
        IndexCtx::Ref => format!("unsafe {{ &*{}.get_unchecked({}) }}", base, idx),
        IndexCtx::RefMut => format!("unsafe {{ &mut *{}.get_unchecked_mut({}) }}", base, idx),
    }
}

fn strip_index_ctx(s: &str) -> (IndexCtx, &str) {
    let s = s.trim_start();

    if let Some(rest) = s.strip_prefix("&mut ") {
        return (IndexCtx::RefMut, rest.trim_start());
    }

    if let Some(rest) = s.strip_prefix('&') {
        return (IndexCtx::Ref, rest.trim_start());
    }

    (IndexCtx::Value, s)
}

fn rewrite_index_in_snippet(snippet: &str) -> Option<String> {
    let (ctx, rest) = strip_index_ctx(snippet);

    let open = rest.rfind('[')?;
    let close = rest.rfind(']')?;
    if close < open {
        return None;
    }

    let base = rest[..open].trim();
    let idx = rest[open + 1..close].trim();

    let rewritten = rewrite_core_index(base, idx, ctx);

    // Replace only the index expression, not the whole statement
    let prefix_len = snippet.len() - rest.len();
    let prefix = &snippet[..prefix_len];
    let suffix = &rest[close + 1..];

    Some(format!("{}{}{}", prefix, rewritten, suffix))
}

fn parse_index(expr: &str) -> Option<(String, String)> {
    let expr = expr.trim();

    // Find the *last* '[' to be robust against things like `foo()[i]`
    let open = expr.rfind('[')?;
    let close = expr.rfind(']')?;

    if close < open {
        return None;
    }

    let base = expr[..open].trim();
    let idx = expr[open + 1..close].trim();

    if base.is_empty() || idx.is_empty() {
        return None;
    }

    Some((base.to_string(), idx.to_string()))
}

fn rewrite_assignment_atomic(stmt: &str) -> Option<String> {
    let stmt = stmt.trim().trim_end_matches(';');
    let (lhs, rhs) = stmt.split_once('=')?;

    let (_, lhs_rest) = strip_index_ctx(lhs);
    let (rhs_ctx, rhs_rest) = strip_index_ctx(rhs);

    let (lhs_base, lhs_idx) = parse_index(lhs_rest.trim())?;
    let (rhs_base, rhs_idx) = parse_index(rhs_rest.trim())?;

    // LHS must be mutable
    let lhs = rewrite_core_index(&lhs_base, &lhs_idx, IndexCtx::RefMut);
    let rhs = rewrite_core_index(&rhs_base, &rhs_idx, rhs_ctx);

    Some(format!("{} = {}", lhs, rhs))
}

fn rewrite_statement(stmt: &str) -> Option<String> {
    let stmt = stmt.trim();

    // ASSIGNMENT: rewrite both sides, wrap whole statement
    if stmt.contains('=') {
        return rewrite_assignment_atomic(stmt);
    }

    // NON-ASSIGNMENT: rewrite single indexing expression
    rewrite_index_in_snippet(stmt)
}

impl PanicDetector for ArrayAccessDetector {
    fn detect_terminator<'tcx>(
        &self,
        _tcx: TyCtxt<'tcx>,
        _body: &'tcx rustc_middle::mir::Body<'tcx>,
        terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let TerminatorKind::Assert { msg, .. } = &terminator.kind else {
            return None;
        };

        let AssertKind::BoundsCheck { .. } = *msg.clone() else {
            return None;
        };

        let span = terminator.source_info.span.source_callsite();

        let mut spans = self.bounds_check_spans.borrow_mut();

        if spans.iter().any(|s| s.contains(span)) {
            return None; // already have a more precise span
        }

        spans.retain(|s| !span.contains(*s));
        spans.insert(span);

        // Detection only — rewriting happens at statement level
        None
    }

    fn detect_statement<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        _body: &rustc_middle::mir::Body<'tcx>,
        statement: &rustc_middle::mir::Statement<'tcx>,
    ) -> Option<NoSeatbeltsDiag> {
        let stmt_span = statement.source_info.span.source_callsite();

        // Get source snippet early — special-cases need this first
        let sm = tcx.sess.source_map();
        let snippet = sm.span_to_snippet(stmt_span).ok()?;

        let line_span = sm.span_extend_to_line(stmt_span);
        let line_snippet = sm.span_to_snippet(line_span).ok();

        let normalize = |s: &str| s.trim().trim_end_matches(';').trim().to_string();

        if let Some(line_snippet) = &line_snippet {
            if line_snippet.contains('=') && line_snippet.matches('[').count() == 2 {
                // Only the MIR statement whose snippet equals the line snippet
                // is allowed to handle the assignment rewrite.
                if normalize(&snippet) != normalize(line_snippet) {
                    return None;
                }
                // fall through to assignment special-case
            }

            let normalized_snippet = normalize(&snippet);
            let normalized_line = normalize(line_snippet);

            let is_assignment_line = normalized_line.contains('=');
            let lhs_has_index = normalized_line
                .split('=')
                .next()
                .is_some_and(|lhs| lhs.contains('['));

            if is_assignment_line && lhs_has_index && normalized_snippet != normalized_line {
                return None;
            }
            if is_assignment_line && lhs_has_index && normalized_snippet != normalized_line {
                return None;
            }
        }

        // ============================================================
        // PRESENTATION SPECIAL-CASE: a[i] = val
        // ============================================================
        if snippet.contains('=') && snippet.matches('[').count() == 1 {
            let parts: Vec<_> = snippet.split('=').collect();
            if parts.len() == 2 {
                let lhs = parts[0].trim();
                let rhs = parts[1].trim().trim_end_matches(';');

                // Expect LHS of the form `base[idx]`
                if let Some(l_bracket) = lhs.find('[') {
                    let base = lhs[..l_bracket].trim();
                    let idx = &lhs[l_bracket + 1..lhs.rfind(']').unwrap()];

                    // 🔑 Claim the bounds check for this statement
                    {
                        let spans = self.bounds_check_spans.borrow();
                        let mut emitted = self.emitted_bounds_spans.borrow_mut();
                        for s in spans.iter().filter(|s| s.overlaps(stmt_span)) {
                            emitted.insert(span_key(*s));
                        }
                    }

                    let rewritten = format!(
                        "unsafe {{ *{base}.get_unchecked_mut({idx}) = {rhs}; }}",
                        base = base,
                        idx = idx,
                        rhs = rhs,
                    );

                    return Some(NoSeatbeltsDiag {
                        span: stmt_span,
                        kind: PanicKind::BoundsCheck,
                        suggestion: Some(Suggestion::ReplaceCall {
                            replacement: rewritten,
                        }),
                    });
                }
            }
        }

        // ============================================================
        // PRESENTATION SPECIAL-CASE: a[i] = a[j]  (same base)
        // ============================================================
        if snippet.contains('=') && snippet.matches('[').count() == 2 {
            let parts: Vec<_> = snippet.split('=').collect();
            if parts.len() == 2 {
                let lhs = parts[0].trim();
                let rhs = parts[1].trim().trim_end_matches(';');

                if let (Some(l_bracket_l), Some(r_bracket_l)) = (lhs.find('['), rhs.find('[')) {
                    let base_l = lhs[..l_bracket_l].trim();
                    let base_r = rhs[..r_bracket_l].trim();

                    // Only if both sides index the SAME base
                    if base_l == base_r {
                        let idx_l = &lhs[l_bracket_l + 1..lhs.rfind(']').unwrap()];
                        let idx_r = &rhs[r_bracket_l + 1..rhs.rfind(']').unwrap()];

                        // 🔑 Claim ALL bounds checks overlapping this statement
                        {
                            let spans = self.bounds_check_spans.borrow();
                            let mut emitted = self.emitted_bounds_spans.borrow_mut();
                            for s in spans.iter().filter(|s| s.overlaps(stmt_span)) {
                                emitted.insert(span_key(*s));
                            }
                        }

                        let rewritten = format!(
                            "unsafe {{ *{base}.get_unchecked_mut({i_l}) = *{base}.get_unchecked({i_r}); }}",
                            base = base_l,
                            i_l = idx_l,
                            i_r = idx_r,
                        );

                        return Some(NoSeatbeltsDiag {
                            span: stmt_span,
                            kind: PanicKind::BoundsCheck,
                            suggestion: Some(Suggestion::ReplaceCall {
                                replacement: rewritten,
                            }),
                        });
                    }
                }
            }
        }

        // ============================================================
        // GENERIC SINGLE-INDEX CASES
        // ============================================================

        // 1. Find canonical bounds-check span overlapping this statement
        let canonical_span = {
            let spans = self.bounds_check_spans.borrow();
            spans.iter().find(|s| s.overlaps(stmt_span)).cloned()
        }?;

        // 2. Only the OUTERMOST statement may emit
        if !stmt_span.contains(canonical_span) {
            return None;
        }

        // 3. Emit AT MOST ONCE per bounds check
        let canonical_key = span_key(canonical_span);
        {
            let mut emitted = self.emitted_bounds_spans.borrow_mut();
            if !emitted.insert(canonical_key) {
                return None;
            }
        }

        // ===============================
        // PRESENTATION SAFETY FILTERS
        // ===============================

        // (a) Exactly ONE indexing operation
        if snippet.matches('[').count() != 1 {
            return None;
        }

        // (b) Skip assignments like `a[i] = ...`
        if snippet.contains('=') {
            let parts: Vec<_> = snippet.split('=').collect();
            if parts.len() > 1 && parts[0].contains('[') {
                return None;
            }
        }

        // (c) Skip reference indexing (`&a[i]`)
        if snippet.contains('&') && snippet.contains('[') {
            return None;
        }

        // 4. Rewrite
        let rewritten = rewrite_statement(&snippet)?;

        // (d) Must introduce a full unsafe block
        if !rewritten.contains("unsafe {") {
            return None;
        }

        // (e) Never rewrite something that still indexes
        if rewritten.contains('[') {
            return None;
        }

        Some(NoSeatbeltsDiag {
            span: stmt_span,
            kind: PanicKind::BoundsCheck,
            suggestion: Some(Suggestion::ReplaceCall {
                replacement: rewritten,
            }),
        })
    }
}
