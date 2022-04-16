#![feature(type_alias_impl_trait)]

// pub mod codegen;
// pub mod exec;
pub mod ir;
pub mod parser;
pub mod pass;
pub mod traits;

extern crate anyhow;
extern crate nom;
