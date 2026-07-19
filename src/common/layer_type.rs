// src/common/layer_type.rs

use serde::{ Serialize, Deserialize };

/// Definisce i canali visivi e logici isolabili all'interno del copione.
/// Implementa nativamente Copy ed Eq per l'uso immediato nelle matrici di filtraggio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(usize)]
pub enum LayerType {
    /// Il testo standard del copione (Dialoghi, Didascalie, Personaggi)
    ScriptText,
    /// Note e appunti dedicati alla regia o alla recitazione
    DirectorNotes,
    /// Note tecniche di produzione (Inquadrature, Luci, Audio)
    ProductionMeta,
    /// Layer di revisione per tracciare le modifiche storiche
    Revision,
}

impl LayerType {
    /// Helper per mappare i valori di enum LayerType sull'indice dell'array statico
    pub fn to_index(&self) -> usize {
        *self as usize
    }
}

// Unit Test

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// Verifica che il mapping degli indici sia stabile, sequenziale
    /// e coerente con la rappresentazione in memoria impostata da #[repr(usize)].
    #[test]
    fn test_layer_indices() {
        assert_eq!(LayerType::ScriptText.to_index(), 0);
        assert_eq!(LayerType::DirectorNotes.to_index(), 1);
        assert_eq!(LayerType::ProductionMeta.to_index(), 2);
        assert_eq!(LayerType::Revision.to_index(), 3);

        // Verifica che il casting esplicito "as usize" combaci perfettamente con to_index()
        assert_eq!(LayerType::ScriptText as usize, LayerType::ScriptText.to_index());
        assert_eq!(LayerType::Revision as usize, LayerType::Revision.to_index());
    }

    /// Verifica che l'enum mantenga l'integrità dei dati quando viene trasferita
    /// tramite JSON (fondamentale per la sincronizzazione dello stato dei layer con la UI).
    #[test]
    fn test_serde_layer_serialization() {
        let layer = LayerType::DirectorNotes;
        let serialized = serde_json::to_string(&layer).unwrap();

        // Serde di default serializza le varianti semplici come stringhe con il nome della variante stessa
        assert_eq!(serialized, "\"DirectorNotes\"");

        // Verifica la deserializzazione speculare
        let deserialized: LayerType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, LayerType::DirectorNotes);
    }

    /// Verifica che tentativi di caricare layer inesistenti o malformati generino un errore pulito.
    #[test]
    fn test_serde_invalid_layer() {
        let invalid_json = "\"VisualEffectsLayer\"";
        let result: Result<LayerType, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}