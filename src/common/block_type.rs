// src/common/block_type.rs

use serde::{Deserialize, Serialize};

/// Identifica la funzione drammaturgica ed espressiva di un blocco di testo
/// all'interno del copione.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
    /// Note e appunti del regista
    #[serde(rename = "director_note")]
    DirectorNote,
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
    /// Nome dell'autore
    #[serde(rename = "author_name")]
    AuthorName,
    /// Versione o revisione dello script
    #[serde(rename = "script_version")]
    ScriptVersion,
    /// Note di produzione, contatti o riferimenti legali/SIAE
    #[serde(rename = "production_note")]
    ProductionNote,
}

impl BlockType {
    /// Restituisce la rappresentazione testuale statico-statica standard (snake_case) 
    /// della variante. Garantisce Zero-Allocations.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Heading1 => "heading_1",
            Self::Heading2 => "heading_2",
            Self::Heading3 => "heading_3",
            Self::DirectorNote => "director_note",
            Self::StageDirection => "stage_direction",
            Self::CharacterName => "character_name",
            Self::Parenthetical => "parenthetical",
            Self::Dialog => "dialog",
            Self::Transition => "transition",
            Self::AuthorName => "author_name",
            Self::ScriptVersion => "script_version",
            Self::ProductionNote => "production_note",
        }
    }

    /// Tenta di convertire una stringa nella corrispondente variante `BlockType`.
    /// Restituisce `Option::None` se la stringa non corrisponde a nessun tipo valido.
    pub fn from_str(text: &str) -> Option<Self> {
        match text.trim() {
            "heading_1" => Some(Self::Heading1),
            "heading_2" => Some(Self::Heading2),
            "heading_3" => Some(Self::Heading3),
            "director_note" => Some(Self::DirectorNote),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_str_and_from_str_coherence() {
        let all_types = [
            BlockType::Heading1,
            BlockType::Heading2,
            BlockType::Heading3,
            BlockType::DirectorNote,
            BlockType::StageDirection,
            BlockType::CharacterName,
            BlockType::Parenthetical,
            BlockType::Dialog,
            BlockType::Transition,
            BlockType::AuthorName,
            BlockType::ScriptVersion,
            BlockType::ProductionNote,
        ];

        for block_type in all_types {
            let str_repr = block_type.as_str();
            let parsed = BlockType::from_str(str_repr);
            assert_eq!(parsed, Some(block_type), "Fallita la conversione da e verso stringa per {:?}", block_type);
        }
    }

    #[test]
    fn test_from_str_invalid_and_whitespace() {
        assert_eq!(BlockType::from_str("dialog"), Some(BlockType::Dialog));
        assert_eq!(BlockType::from_str("  dialog  \n"), Some(BlockType::Dialog));
        assert_eq!(BlockType::from_str("invalid_type"), None);
        assert_eq!(BlockType::from_str(""), None);
    }

    #[test]
    fn test_serde_serialization() {
        let block = BlockType::Dialog;
        let json = serde_json::to_string(&block).expect("Serializzazione fallita");
        assert_eq!(json, "\"dialog\"");

        let deserialized: BlockType = serde_json::from_str(&json).expect("Deserializzazione fallita");
        assert_eq!(deserialized, BlockType::Dialog);
    }
}