use crate::{detectors::PanicDetector, diagnostics::Suggestion};

pub struct DivByZeroDetector;

use rustc_middle::mir::{BinOp, Rvalue, StatementKind};
use rustc_span::{BytePos, Span};

fn trim_unbalanced_parens(s: &str) -> String {
    let mut balance = 0;
    for c in s.chars() {
        match c {
            '(' => balance += 1,
            ')' => balance -= 1,
            _ => {}
        }
    }

    // If balance >= 0, parens are balanced or under-closed: do nothing
    if balance >= 0 {
        return s.trim().to_string();
    }

    // balance < 0 => too many ')'
    let mut trimmed = s.trim_end();
    while balance < 0 {
        if let Some(last) = trimmed.chars().last() {
            if last == ')' {
                trimmed = &trimmed[..trimmed.len() - last.len_utf8()];
                balance += 1;
                trimmed = trimmed.trim_end();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    trimmed.to_string()
}

fn extract_rhs_span_of_assign(
    tcx: rustc_middle::ty::TyCtxt<'_>,
    statement: &rustc_middle::mir::Statement<'_>,
) -> Option<Span> {
    let source_map = tcx.sess.source_map();
    let stmt_span = statement.source_info.span;

    // Get full statement text
    let stmt_snippet = source_map.span_to_snippet(stmt_span).ok()?;

    // Guardrails: keep the hack contained
    if stmt_snippet.contains('\n') || stmt_snippet.contains('{') || stmt_snippet.contains('}') {
        return None;
    }

    // We are *already* inside StatementKind::Assign, so we expect exactly one '='
    let eqs: Vec<_> = stmt_snippet.match_indices('=').collect();
    if eqs.len() != 1 {
        println!("stmt_snippet: {}", stmt_snippet);
        return Some(stmt_span);
    }

    let eq_pos = eqs[0].0;

    // Extract RHS text
    let mut rhs_text = stmt_snippet[eq_pos + 1..].trim();

    // Strip trailing semicolon if present
    if rhs_text.ends_with(';') {
        rhs_text = rhs_text[..rhs_text.len() - 1].trim();
    }

    // Must still look like a div/rem expression
    if !rhs_text.contains('/') && !rhs_text.contains('%') {
        println!("done..");
        return None;
    }

    // Compute byte offsets relative to original span
    let stmt_lo = stmt_span.lo();

    // Compute how many bytes into the statement the RHS starts
    let prefix = &stmt_snippet[..eq_pos + 1];
    let rhs_offset = prefix.len() + (stmt_snippet[eq_pos + 1..].len() - rhs_text.len());

    let rhs_lo = stmt_lo + BytePos(rhs_offset as u32);
    let rhs_hi = rhs_lo + BytePos(rhs_text.len() as u32);

    Some(Span::new(
        rhs_lo,
        rhs_hi,
        stmt_span.ctxt(),
        stmt_span.parent(),
    ))
}

impl PanicDetector for DivByZeroDetector {
    fn detect_terminator<'tcx>(
        &self,
        _tcx: rustc_middle::ty::TyCtxt<'tcx>,
        _body: &rustc_middle::mir::Body<'tcx>,
        _terminator: &rustc_middle::mir::Terminator<'tcx>,
    ) -> Option<crate::diagnostics::NoSeatbeltsDiag> {
        None
    }

    fn detect_statement<'tcx>(
        &self,
        tcx: rustc_middle::ty::TyCtxt<'tcx>,
        _body: &rustc_middle::mir::Body<'tcx>,
        statement: &rustc_middle::mir::Statement<'tcx>,
    ) -> Option<crate::diagnostics::NoSeatbeltsDiag> {
        use rustc_middle::mir::StatementKind;

        if let StatementKind::Assign(info) = &statement.kind {
            let (_, rvalue) = &**info;
            if let rustc_middle::mir::Rvalue::BinaryOp(
                rustc_middle::mir::BinOp::Div | rustc_middle::mir::BinOp::Rem,
                info,
            ) = rvalue
            {
                let (_, rhs) = &**info;

                // Try to get the original source text for the span
                let source_map = tcx.sess.source_map();

                let rhs_text =
                    if let Ok(snippet) = source_map.span_to_snippet(statement.source_info.span) {
                        // Extract just the denominator part - this is tricky and approximate
                        trim_unbalanced_parens(if let Some(div_pos) = snippet.find('/') {
                            snippet[div_pos + 1..].trim()
                        } else if let Some(rem_pos) = snippet.find('%') {
                            snippet[rem_pos + 1..].trim()
                        } else {
                            panic!("expected / or % in div/rem expression")
                        })
                    } else {
                        format!("{:?}", rhs) // fallback to MIR
                    };

                let og_expression_span =
                    extract_rhs_span_of_assign(tcx, statement).expect("could not extract RHS span");
                let og_expression = source_map
                    .span_to_snippet(og_expression_span)
                    .expect("could not get original expression snippet");

                return Some(crate::diagnostics::NoSeatbeltsDiag {
                    span: og_expression_span,
                    kind: match rvalue {
                        rustc_middle::mir::Rvalue::BinaryOp(rustc_middle::mir::BinOp::Div, _) => {
                            crate::diagnostics::PanicKind::DivByZero
                        }
                        rustc_middle::mir::Rvalue::BinaryOp(rustc_middle::mir::BinOp::Rem, _) => {
                            crate::diagnostics::PanicKind::RemByZero
                        }
                        _ => unreachable!(),
                    },
                    suggestion: Some(Suggestion::WrapWithAssertUnchecked {
                        condition: format!("{} != 0", rhs_text),
                        original_expression: og_expression,
                    }),
                });
            }
        }

        None
    }
}
