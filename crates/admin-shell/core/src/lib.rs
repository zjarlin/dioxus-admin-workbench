#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod appearance;
mod command;
mod compiler;
mod definition;
mod extension;
mod naming;
mod resource;

pub use command::*;
pub use compiler::*;
pub use definition::*;
pub use extension::*;
pub use naming::*;
pub use resource::*;

rudi::enable! {}
