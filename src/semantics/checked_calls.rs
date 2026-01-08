use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::sym;

use crate::diagnostics::Suggestion;

pub fn get_replacement(
    tcx: TyCtxt<'_>,
    def_id: DefId,
    receiver: Option<String>,
) -> Option<Suggestion> {
    if tcx.is_diagnostic_item(sym::option_unwrap, def_id)
        || tcx.is_diagnostic_item(sym::option_expect, def_id)
        || tcx.is_diagnostic_item(sym::option_unwrap, def_id)
    {
        let recv = receiver?;
        return Some(Suggestion::ReplaceCall {
            replacement: format!("unsafe {{ {}.unwrap_unchecked() }}", recv),
        });
    }

    None
}
