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
extern crate rustc_target;

use std::{path::PathBuf, process::Command};

use clap::Parser;
use rustc_errors::registry;
use rustc_hash::FxHashMap;
use rustc_session::config;

use no_seatbelts::PanicPass;

#[derive(Parser)]
pub struct NoSeatbeltsArgs {
    pub input: PathBuf,

    #[arg(long)]
    pub error_format: Option<String>,

    #[arg(long)]
    pub no_std: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub rustc_args: Vec<String>,
}

fn run_as_rustc_wrapper() {
    let mut args = std::env::args().collect::<Vec<String>>();
    if args.len() >= 3 && args[2] == "-vV" {
        // Forward directly to real rustc
        let status = std::process::Command::new(&args[1])
            .args(&args[2..])
            .status()
            .expect("failed to run rustc -vV");
        std::process::exit(status.code().unwrap_or(1));
    }

    let mut args_iter = std::env::args();
    args_iter.next();
    args_iter.next().expect("expected rustc path");
    let mut rustc_args: Vec<String> = vec![];
    rustc_args.push("rustc".to_string()); // argv[0] — REQUIRED
    rustc_args.extend(args_iter);

    let mut lint_pass = PanicPass::new(vec![
        Box::new(no_seatbelts::CheckedFunctionDetector),
        Box::new(no_seatbelts::DivByZeroDetector),
        Box::new(no_seatbelts::ExplicitPanicDetector),
        Box::new(no_seatbelts::ArrayAccessDetector::new()),
        Box::new(no_seatbelts::SliceDetector::new()),
    ]);

    rustc_driver::run_compiler(&rustc_args, &mut lint_pass);
}

fn main() {
    eprintln!("no-seatbelts main called");
    if std::env::var("RUSTC_WRAPPER").is_ok() {
        run_as_rustc_wrapper();
        return;
    }
}

fn stub() {
    let mut args = NoSeatbeltsArgs::parse();
    let mut opts = config::Options::default();

    if let Some(format) = args.error_format {
        let error_format = match format.as_str() {
            "json" => config::ErrorOutputType::Json {
                pretty: false,
                json_rendered: rustc_errors::emitter::HumanReadableErrorType::Default,
                color_config: rustc_errors::emitter::ColorConfig::Auto,
            },
            _ => panic!("unsupported error format: {}", format),
        };
        opts.error_format = error_format;
    }

    if args.no_std {
        opts.cg.panic = Some(rustc_target::spec::PanicStrategy::Abort);
        args.rustc_args.push("-Zbuild-std=core".to_string());
    }

    let config = rustc_interface::Config {
        extra_symbols: vec![],
        opts,
        crate_cfg: Vec::new(),
        crate_check_cfg: Vec::new(),
        input: config::Input::File(args.input),
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
        expanded_args: args.rustc_args,
        ice_file: None,
        hash_untracked_state: None,
        using_internal_features: &rustc_driver::USING_INTERNAL_FEATURES,
    };
    rustc_interface::run_compiler(config, |compiler| {
        let krate = rustc_interface::passes::parse(&compiler.sess);
        rustc_interface::create_and_enter_global_ctxt(compiler, krate, |tcx| {
            let lint_pass = PanicPass::new(vec![
                Box::new(no_seatbelts::CheckedFunctionDetector),
                Box::new(no_seatbelts::DivByZeroDetector),
                Box::new(no_seatbelts::ExplicitPanicDetector),
                Box::new(no_seatbelts::ArrayAccessDetector::new()),
                Box::new(no_seatbelts::SliceDetector::new()),
            ]);

            for free_id in tcx.hir_crate_items(()).free_items() {
                let item = tcx.hir_item(free_id);
                if let rustc_hir::ItemKind::Fn { .. } = item.kind {
                    let def_id = item.hir_id().owner.def_id;
                    if tcx.hir_maybe_body_owned_by(def_id).is_some() {
                        let body = tcx.optimized_mir(def_id.to_def_id());
                        lint_pass.check_body(&tcx, body);
                    }
                }
            }

            for impl_id in tcx.hir_crate_items(()).impl_items() {
                let impl_item = tcx.hir_impl_item(impl_id);
                if let rustc_hir::ImplItemKind::Fn { .. } = impl_item.kind {
                    let def_id = impl_item.hir_id().owner.def_id;
                    if tcx.hir_maybe_body_owned_by(def_id).is_some() {
                        let body = tcx.optimized_mir(def_id.to_def_id());
                        lint_pass.check_body(&tcx, body);
                    }
                }
            }
        });
    });
}
