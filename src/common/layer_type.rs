// src/common/layer_type.rs

use serde::{ Deserialize, Serialize };

/// Definisce i canali visivi e logici isolabili all'interno del copione.
/// Implementa nativamente `Copy` ed `Eq` per l'uso immediato nelle matrici di filtraggio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(usize)]
pub enum LayerType {
    /// Il testo standard del copione (Dialoghi, Didascalie, Personaggi)
    ScriptText = 0,
    /// Note e appunti dedicati alla regia o alla recitazione
    DirectorNotes = 1,
    /// Note tecniche di produzione (Inquadrature, Luci, Audio)
    ProductionMeta = 2,
    /// Layer di revisione per tracciare le modifiche storiche
    Revision = 3,
}

impl LayerType {
    /// Restituisce l'indice posizionale (0-3) del canale all'interno delle matrici fisse di visibilità.
    pub fn to_index(&self) -> usize {
        *self as usize
    }

    /// Tenta di convertire un indice numerico (0-3) nella corrispondente variante `LayerType`.
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::ScriptText),
            1 => Some(Self::DirectorNotes),
            2 => Some(Self::ProductionMeta),
            3 => Some(Self::Revision),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_type_indices() {
        assert_eq!(LayerType::ScriptText.to_index(), 0);
        assert_eq!(LayerType::DirectorNotes.to_index(), 1);
        assert_eq!(LayerType::ProductionMeta.to_index(), 2);
        assert_eq!(LayerType::Revision.to_index(), 3);
    }

    #[test]
    fn test_from_index_coherence() {
        let layers = [
            LayerType::ScriptText,
            LayerType::DirectorNotes,
            LayerType::ProductionMeta,
            LayerType::Revision,
        ];

        for layer in layers {
            let idx = layer.to_index();
            assert_eq!(LayerType::from_index(idx), Some(layer));
        }

        assert_eq!(LayerType::from_index(4), None);
        assert_eq!(LayerType::from_index(99), None);
    }

    #[test]
    fn test_serde_serialization() {
        let layer = LayerType::DirectorNotes;
        let json = serde_json::to_string(&layer).expect("Serializzazione fallita");

        let deserialized: LayerType = serde_json::from_str(&json).expect("Deserializzazione fallita");
        assert_eq!(deserialized, LayerType::DirectorNotes);
    }
}