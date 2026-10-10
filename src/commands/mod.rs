// src/commands/mod.rs

//! Comandi che mutano il `Document` e il bus che li applica in batch
//! atomici, con cronologia undo/redo. Separato da `lake` (che resta il
//! solo custode dello storage) e da `core` (il modello puro): qui vive la
//! logica di "come cambia il documento nel tempo", non "cosa contiene" né
//! "come è memorizzato".
//!
//! `Undo` e `ChangeKind` restano interni (`pub(crate)`): l'unica
//! superficie pubblica per mutare il documento è `Command` + `CommandBus`
//! — non è previsto costruire o applicare un `Undo` a mano.
mod bus;
mod command;

// Re-export Metodi
pub use bus::CommandBus;
pub use command::{ Command, Delta };

// Visibile solo in questo modulo
pub(crate) use command::Undo;