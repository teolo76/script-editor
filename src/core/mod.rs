// src/core/mod.rs

//! Modello strutturale del documento: entità di base, nodi dell'albero,
//! blocchi di contenuto. `Annotation` e `Layer` arriveranno in un passo
//! successivo, insieme all'arena che conterrà tutte queste entità e alle
//! query per attraversare l'albero.
//!
//! Nota sul nome: è un semplice modulo interno — va sempre riferito
//! come `crate::core::...` (o `script_edt::core::...` dall'esterno).
//! Non c'è ambiguità con il crate di libreria standard `core` a meno di
//! scrivere un bare `use core::...`, cosa che qui non facciamo mai.
mod annotation;
mod block;
mod character;
mod doc_meta;
mod entity;
mod layer;
mod node;

// Re-export dei metodi necessari

pub use annotation::{ Annotation, AnchorTarget };
pub use block::{ Block, LayerId, AssetBlock, TextBlock, BlockPayload };
pub use character::Character;
pub use doc_meta::{ DocMeta, CoverImage, Authorship, Production };
pub use entity::{ BaseElement, Meta, Entity, TreeElement };
pub use layer::Layer;
pub use node::Node;
