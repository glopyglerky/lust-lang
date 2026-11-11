//! Source reuse boundary: every native module below is compiled directly from the
//! pinned, untouched upstream tree. This crate adds the Cargo invocation contract.
#![allow(dead_code)]
#[path = "../../../compiler/native/src"]
mod upstream {
    pub(crate) mod ast;
    pub(crate) mod borrowck;
    pub(crate) mod capabilities;
    pub(crate) mod cc;
    pub(crate) mod cgen;
    pub(crate) mod check;
    pub(crate) mod colwidth;
    pub(crate) mod daemon;
    pub(crate) mod def;
    pub(crate) mod diag;
    pub(crate) mod driver;
    pub(crate) mod expand;
    pub(crate) mod explain;
    pub(crate) mod image;
    pub(crate) mod intern;
    pub(crate) mod ir;
    pub(crate) mod lexer;
    pub(crate) mod lower;
    pub(crate) mod mono;
    pub(crate) mod parser;
    pub(crate) mod pool;
    pub(crate) mod protocol;
    pub(crate) mod resolve;
    pub(crate) mod scan;
    pub(crate) mod stdload;
    pub(crate) mod stdloc;
    pub(crate) mod stdmods;
    pub(crate) mod tcc;
    pub(crate) mod testrun;
    pub(crate) mod traits;
    pub(crate) mod ty;
}
pub(crate) use upstream::{ast, borrowck, capabilities, cc, cgen, check, colwidth, daemon, def, diag, driver, explain, image, intern, ir, lexer, lower, mono, parser, pool, protocol, scan, stdload, stdloc, stdmods, tcc, testrun, traits, ty};
#[path = "native_expand.rs"]
mod expand;
#[path = "native_resolve.rs"]
mod resolve;
mod cargo_cli;
mod native_artifact;
mod native_codegen;
mod native_library;
mod native_import;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    std::process::exit(cargo_cli::run(std::env::args_os().skip(1).collect()));
}
