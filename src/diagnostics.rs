use rustc_errors::Applicability;
use rustc_errors::{Diag, LintDiagnostic};
use rustc_span::Span;

/// The *kind* of panic site we detected.
/// This is the semantic core of no-seatbelts.
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum PanicKind {
    Unwrap,
    Expect,
    BoundsCheck,
    DivByZero,
}

/// A structured suggestion, à la Clippy.
/// This is where the "intelligence" lives.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Suggestion {
    /// Replace the panicking call with an unchecked variant.
    ReplaceCall { replacement: String },

    /// Insert `core::hint::assert_unchecked(cond)`
    InsertAssertUnchecked { condition: &'static str },

    /// Guard with a normal runtime check.
    GuardWithIf { condition: &'static str },
}

impl PanicKind {
    /// High-level explanation of the panic site.
    pub fn message(&self) -> &'static str {
        match self {
            PanicKind::Unwrap | PanicKind::Expect => "This call may panic if the value is invalid.",
            PanicKind::BoundsCheck => "This operation may panic due to an out-of-bounds access.",
            PanicKind::DivByZero => "This operation may panic due to division by zero.",
        }
    }
}

/// The *single* diagnostic used by no-seatbelts.
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
                        "replace this call with its unchecked variant",
                        replacement,
                        Applicability::MaybeIncorrect,
                    );
                }

                Suggestion::InsertAssertUnchecked { condition } => {
                    diag.note(format!(
                        "You may insert `unsafe {{ core::hint::assert_unchecked({}) }}` before this operation.",
                        condition
                    ));
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
