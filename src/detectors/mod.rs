use rustc_middle::{mir::{Body, Statement, Terminator}, ty::TyCtxt};

use crate::diagnostics::NoSeatbeltsDiag;

mod checked_functions;

pub use checked_functions::CheckedFunctionDetector;

pub trait PanicDetector {
    fn detect_terminator<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &Body<'tcx>,
        terminator: &Terminator<'tcx>,
    ) -> Option<NoSeatbeltsDiag>;

    fn detect_statement<'tcx>(
        &self,
        tcx: TyCtxt<'tcx>,
        body: &Body<'tcx>,
        statement: &Statement<'tcx>,
    ) -> Option<NoSeatbeltsDiag>;
}