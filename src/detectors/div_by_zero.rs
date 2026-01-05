use crate::{detectors::PanicDetector, diagnostics::Suggestion};

pub struct DivByZeroDetector;

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
        _tcx: rustc_middle::ty::TyCtxt<'tcx>,
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
                let source_map = _tcx.sess.source_map();
                let rhs_text =
                    if let Ok(snippet) = source_map.span_to_snippet(statement.source_info.span) {
                        // Extract just the denominator part - this is tricky and approximate
                        if let Some(div_pos) = snippet.find('/') {
                            snippet[div_pos + 1..].trim().to_string()
                        } else if let Some(rem_pos) = snippet.find('%') {
                            snippet[rem_pos + 1..].trim().to_string()
                        } else {
                            format!("{:?}", rhs) // fallback to MIR
                        }
                    } else {
                        format!("{:?}", rhs) // fallback to MIR
                    };

                return Some(crate::diagnostics::NoSeatbeltsDiag {
                    span: statement.source_info.span,
                    kind: match rvalue {
                        rustc_middle::mir::Rvalue::BinaryOp(rustc_middle::mir::BinOp::Div, _) => {
                            crate::diagnostics::PanicKind::DivByZero
                        }
                        rustc_middle::mir::Rvalue::BinaryOp(rustc_middle::mir::BinOp::Rem, _) => {
                            crate::diagnostics::PanicKind::RemByZero
                        }
                        _ => unreachable!(),
                    },
                    suggestion: Some(Suggestion::InsertAssertUnchecked {
                        condition: format!("{} != 0", rhs_text),
                    }),
                });
            }
        }

        None
    }
}
