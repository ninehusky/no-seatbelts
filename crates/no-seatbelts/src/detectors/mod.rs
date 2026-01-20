use rustc_middle::{
    mir::{Body, Statement, Terminator},
    ty::TyCtxt,
};

use crate::diagnostics::NoSeatbeltsDiag;

mod array_access;
mod checked_functions;
mod div_by_zero;
mod explicit_panics;

pub use array_access::ArrayAccessDetector;
pub use checked_functions::CheckedFunctionDetector;
pub use div_by_zero::DivByZeroDetector;
pub use explicit_panics::ExplicitPanicDetector;

pub trait PanicDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &'tcx Body<'tcx>,
        terminator: &Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag>;

    fn detect_statement<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &Body<'tcx>,
        statement: &Statement<'tcx>,
    ) -> Option<NoSeatbeltsDiag>;
}
