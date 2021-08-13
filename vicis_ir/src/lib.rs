#![feature(vec_into_raw_parts)]

// pub mod codegen;
// pub mod exec;
#[macro_use]
pub mod macros;
pub mod ir;
pub mod pass;
pub mod traits;

extern crate anyhow;
extern crate nom;
