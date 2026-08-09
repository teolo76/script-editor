// src/core/mod.rs

mod arena;
mod container;
mod node;
mod record;

// Re-export piatto di tutti i tipi del namespace core
pub use arena::*;
pub use container::*;
pub use node::*;
pub use record::*;