use std::path::Path;

use bend::{diagnostics::{self, DiagnosticsConfig}, CompileOpts, RunOpts};

pub fn run( ) {
    let path = Path::new("main.bend");
    let book = bend::load_file_to_book( &path ).unwrap();

    let run_opts = RunOpts { linear_readback: false, pretty: false };
    let compile_opts = CompileOpts::default();
    let diagnostics_cfg = DiagnosticsConfig::default();
    let args = None;

    let result = bend::run_book(book, run_opts, compile_opts, diagnostics_cfg, args, "run-cu").unwrap();

    println!("{:?}", result);

}