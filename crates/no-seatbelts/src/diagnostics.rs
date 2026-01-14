use rustc_errors::Applicability;
use rustc_errors::{Diag, LintDiagnostic};
use rustc_span::Span;

/// The *kind* of panic site we detected.
/// This is the semantic core of no-seatbelts.
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum PanicKind {
    CheckedFunction,
    BoundsCheck,
    DivByZero,
    RemByZero,
}

/// A structured suggestion, à la Clippy.
/// This is where the "intelligence" lives.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Suggestion {
    /// Replace the panicking call with an unchecked variant.
    ReplaceCall { replacement: String },

    /// Insert `core::hint::assert_unchecked(cond)` before the expression.
    WrapWithAssertUnchecked {
        condition: String,
        original_expression: String,
    },

    /// Guard with a normal runtime check.
    GuardWithIf { condition: &'static str },
}

impl PanicKind {
    /// High-level explanation of the panic site.
    pub fn message(&self) -> &'static str {
        match self {
            PanicKind::CheckedFunction => "A panic check is emitted by this function call.",
            PanicKind::BoundsCheck => {
                "This operation emits a panic check for out-of-bounds access."
            }
            PanicKind::DivByZero => "This operation emits a panic check for division by zero.",
            PanicKind::RemByZero => "This operation emits a panic check for modulo by zero.",
        }
    }
}

/// The *single* diagnostic used by no-seatbelts.
#[derive(Clone, Debug)]
pub struct NoSeatbeltsDiag {
    pub span: Span,
    pub kind: PanicKind,
    pub suggestion: Option<Suggestion>,
}

impl<'a> LintDiagnostic<'a, ()> for NoSeatbeltsDiag {
    fn decorate_lint<'b>(self, diag: &'b mut Diag<'a, ()>) {
        diag.primary_message(self.kind.message());

        if let Some(suggestion) = self.suggestion {
            match suggestion {
                Suggestion::ReplaceCall { replacement } => {
                    diag.span_suggestion(
                        self.span,
                        "if you're sure you want to remove the check, replace this call with its unchecked variant",
                        replacement,
                        Applicability::MaybeIncorrect,
                    );
                }

                Suggestion::WrapWithAssertUnchecked {
                    condition,
                    original_expression,
                } => {
                    let replacement = format!(
                        "unsafe {{ core::hint::assert_unchecked({}); {} }}",
                        condition, original_expression
                    );

                    diag.span_suggestion(
                        self.span,
                        "wrap the expression with an unchecked assertion to remove the panic check",
                        replacement,
                        Applicability::MaybeIncorrect,
                    );
                }

                Suggestion::GuardWithIf { condition } => {
                    diag.note(format!(
                        "You may guard this operation with `if {}`.",
                        condition
                    ));
                }
            }
        }
    }
}
