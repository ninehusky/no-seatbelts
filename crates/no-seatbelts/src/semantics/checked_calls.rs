use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::symbol::sym;

use crate::diagnostics::Suggestion;

pub fn get_replacement(
    tcx: TyCtxt<'_>,
    def_id: DefId,
    receiver: Option<String>,
    args: Option<Vec<String>>,
) -> Option<Suggestion> {
    if tcx.is_diagnostic_item(sym::option_unwrap, def_id)
        || tcx.is_diagnostic_item(sym::option_expect, def_id)
        || tcx.is_diagnostic_item(sym::option_unwrap, def_id)
    {
        let recv = receiver?;
        return Some(Suggestion::ReplaceCall {
            replacement: format!("unsafe {{ {}.unwrap_unchecked() }}", recv),
        });
    } else if tcx.is_diagnostic_item(sym::unreachable_macro, def_id)
        || tcx.is_diagnostic_item(sym::unreachable, def_id)
    {
        assert!(receiver.is_none());
        return Some(Suggestion::ReplaceCall {
            replacement: "unsafe { std::hint::unreachable_unchecked() }".to_string(),
        });
    }

    let callee = tcx.opt_item_name(def_id);

    if let Some(symbol) = callee {
        let fn_name = symbol.as_str();
        if fn_name == "split_at" {
            let recv = receiver?;
            let args = args?[1..].to_vec();
            let arg_list = args.join(", ");

            return Some(Suggestion::ReplaceCall {
                replacement: format!("unsafe {{ {}.split_at_unchecked({}) }}", recv, arg_list),
            });
        }
    }

    None
}
