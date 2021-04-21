// #![feature(in_band_lifetimes)]
#![feature(generators, generator_trait)]


pub mod server;

pub mod module;

pub mod model;
pub mod policy;

pub const THREADS: i32 = 3;