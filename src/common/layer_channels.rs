//! src/common/layer_channels.rs
use serde::{ Serialize, Deserialize };

/// Definisce i canali visivi e logici isolabili all'interno del copione.
/// Implementa nativamente Copy ed Eq per l'uso immediato nelle matrici di filtraggio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerChannel {
    /// Il testo standard del copione (Dialoghi, Didascalie, Personaggi)
    ScriptText,
    /// Note e appunti dedicati alla regia o alla recitazione
    DirectorNotes,
    /// Note tecniche di produzione (Inquadrature, Luci, Audio)
    ProductionMeta,
    /// Layer di revisione per tracciare le modifiche storiche
    Revision,
}

impl LayerChannel {
    /// Helper per mappare l'enum LayerChannel sull'indice dell'array statico
    pub fn to_index(self) -> usize {
        match self {
            Self::ScriptText => 0,
            Self::DirectorNotes => 1,
            Self::ProductionMeta => 2,
            Self::Revision => 3,
        }
    }
}