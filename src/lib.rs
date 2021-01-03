#![feature(vec_into_raw_parts)]

#[macro_use]
pub mod macros;
pub mod codegen;
pub mod exec;
pub mod ir;

extern crate nom;
