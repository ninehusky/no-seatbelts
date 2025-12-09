// Tested with nightly-2025-03-28

#![feature(rustc_private)]
extern crate rustc_driver;
extern crate rustc_error_codes;
extern crate rustc_errors;
extern crate rustc_hash;
extern crate rustc_hir;
extern crate rustc_interface;
extern crate rustc_session;
extern crate rustc_span;

use rustc_errors::registry;
use rustc_hash::FxHashMap;
use rustc_session::config;

use no_seatbelts::{UncheckedFunctionPass};

fn main() {
    let config = rustc_interface::Config {
        extra_symbols: vec![],
        opts: config::Options::default(),
        crate_cfg: Vec::new(),
        crate_check_cfg: Vec::new(),
        input: config::Input::File(std::env::args().nth(1).expect("pass a file path as argument").into()),
        output_dir: None,
        output_file: None,
        file_loader: None,
        locale_resources: rustc_driver::DEFAULT_LOCALE_RESOURCES.to_owned(),
        lint_caps: FxHashMap::default(),
        psess_created: None,
        register_lints: None,
        override_queries: None,
        registry: registry::Registry::new(rustc_errors::codes::DIAGNOSTICS),
        make_codegen_backend: None,
        expanded_args: Vec::new(),
        ice_file: None,
        hash_untracked_state: None,
        using_internal_features: &rustc_driver::USING_INTERNAL_FEATURES,
    };
    rustc_interface::run_compiler(config, |compiler| {
        let krate = rustc_interface::passes::parse(&compiler.sess);
        rustc_interface::create_and_enter_global_ctxt(&compiler, krate, |tcx| {

            let lint_pass = UncheckedFunctionPass {};

            for id in tcx.hir_free_items() {
                let item = tcx.hir_item(id);
                match item.kind {
                    rustc_hir::ItemKind::Fn { ident, .. } => {
                        let def_id = item.hir_id().owner.def_id;
                        if tcx.hir_maybe_body_owned_by(def_id).is_some() {
                            let body = tcx.optimized_mir(def_id.to_def_id());
                            lint_pass.check_body(&tcx, body);
                        }
                    }
                    _ => (),
                }
            }
        });
    });
}
