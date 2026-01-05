use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::sym;

use crate::diagnostics::Suggestion;

pub fn get_replacement(tcx: TyCtxt<'_>, def_id: DefId) -> Option<Suggestion> {
    if tcx.is_diagnostic_item(sym::option_unwrap, def_id)
        || tcx.is_diagnostic_item(sym::option_expect, def_id)
        || tcx.is_diagnostic_item(sym::option_unwrap, def_id)
    {
        return Some(Suggestion::ReplaceCall {
            replacement: format!(
                "unsafe {{ {}::unwrap_unchecked() }}",
                tcx.def_path_str(def_id).split("::").next().unwrap()
            ),
        });
    }

    None
}
