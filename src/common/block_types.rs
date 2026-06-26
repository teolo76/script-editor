//! src/common/block_types.rs
use serde::{ Serialize, Deserialize };

/// Identifica la funzione drammaturgica ed espressiva di un blocco di testo.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
pub enum BlockType {
    /// Titolo 1 - Grande
    #[serde(rename = "heading_1")]
    Heading1,
    /// Titolo 2 - Medio
    #[serde(rename = "heading_2")]
    Heading2,
    /// Titolo 3 - Piccolo
    #[serde(rename = "heading_3")]
    Heading3,
    /// Descrizione delle azioni, dei luoghi o dei movimenti (Didascalia)
    #[serde(rename = "stage_direction")]
    StageDirection,
    /// Il nome del personaggio che sta per parlare
    #[serde(rename = "character_name")]
    CharacterName,
    /// Nota di intonazione o comportamento tra parentesi (es: (sottovoce))
    #[serde(rename = "parenthetical")]
    Parenthetical,
    /// Il testo del dialogo vero e proprio
    #[serde(rename = "dialog")]
    Dialog,
    /// Nota di transizione cinematografica o teatrale (es: STACCO SU:)
    #[serde(rename = "transition")]
    Transition,

    // --- Elementi Specifici per la Cover e la Produzione ---
    /// Diventa la classe CSS `.block-author-name`
    #[serde(rename = "author_name")]
    AuthorName,
    /// Diventa la classe CSS `.block-script-version`
    #[serde(rename = "script_version")]
    ScriptVersion,
    /// Diventa la classe CSS `.block-production-note` (Contatti, SIAE)
    #[serde(rename = "production_note")]
    ProductionNote,
}

impl BlockType {
    /// Restituisce la rappresentazione testuale standard (snake_case) della variante.
    /// Utilizza stringhe statiche per garantire Zero-Allocations.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Heading1 => "heading_1",
            Self::Heading2 => "heading_2",
            Self::Heading3 => "heading_3",
            Self::StageDirection => "stage_direction",
            Self::CharacterName => "character_name",
            Self::Parenthetical => "parenthetical",
            Self::Dialog => "dialog",
            Self::Transition => "transition",
            Self::AuthorName => "author_name",
            Self::ScriptVersion => "script_version",
            Self::ProductionNote => "production_note",
            _ => "unknown",
        }
    }
    /// Tenta di deserializzare una stringa nel corrispondente BlockType.
    /// Restituisce Option::None se la stringa non corrisponde a nessun tipo valido.
    pub fn from_str(str: &str) -> Option<Self> {
        match str.trim() {
            "heading_1" => Some(Self::Heading1),
            "heading_2" => Some(Self::Heading2),
            "heading_3" => Some(Self::Heading3),
            "stage_direction" => Some(Self::StageDirection),
            "character_name" => Some(Self::CharacterName),
            "parenthetical" => Some(Self::Parenthetical),
            "dialog" => Some(Self::Dialog),
            "transition" => Some(Self::Transition),
            "author_name" => Some(Self::AuthorName),
            "script_version" => Some(Self::ScriptVersion),
            "production_note" => Some(Self::ProductionNote),
            _ => None,
        }
    }
}

// =========================================================================
// UNIT TESTS (Aggiornati con la verifica degli UUID)
// =========================================================================
#[cfg(test)]
mod block_type_tests {
    use super::*;

    #[test]
    fn test_block_type_serialization_roundtrip() {
        let original_type = BlockType::StageDirection;

        // Test serializzazione (Enum -> Stringa)
        let serialized = original_type.as_str();
        assert_eq!(serialized, "stage_direction");

        // Test deserializzazione (Stringa -> Enum)
        let deserialized = BlockType::from_str(serialized);
        assert_eq!(deserialized, Some(original_type));
    }

    #[test]
    fn test_block_type_from_str_edge_cases() {
        // Verifica tolleranza spazi vuoti accidentali
        assert_eq!(BlockType::from_str("  dialog  "), Some(BlockType::Dialog));

        // Verifica gestione valori non validi (fallback controllato)
        assert_eq!(BlockType::from_str("unknown_type"), None);
    }
}
