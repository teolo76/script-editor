// src/common/node_type.rs

use serde::{ Serialize, Deserialize };

/// Rappresenta le categorie testuali e drammaturgiche standard che compongono una pagina di copione.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeType {
    /// Il livello radice supremo del copione, guardiano del documento globale.
    Script,
    /// La copertina del copione (gestisce titoli, contatti e FloatingElement grafici).
    Cover,
    /// Il foglio del cast (l'anagrafica che associa i personaggi agli attori).
    CastSheet,
    /// La macro-area di un'opera (es: Atto I, Atto II).
    Act,
    /// L'unità spazio-temporale fondamentale che ospita direttamente i blocchi di testo.
    Scene,
}

impl NodeType {
    /// Helper per mappare i valori sull'indice dell'array statico
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}