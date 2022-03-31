#![feature(let_chains)]

extern crate vicis_core;

#[macro_use]
pub mod macros;
pub mod call_conv;
pub mod function;
pub mod isa;
pub mod lower;
pub mod module;
pub mod pass;
pub mod register;
