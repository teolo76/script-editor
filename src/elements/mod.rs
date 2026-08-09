// src/elements/mod.rs

mod act;
mod blocks;
mod characters;
mod cover;
mod payloads;
mod scene;
mod script;

// Re-export piatto di tutti i tipi del namespace elements
pub use act::*;
pub use blocks::*;
pub use characters::*;
pub use cover::*;
pub use payloads::*;
pub use scene::*;
pub use script::*;