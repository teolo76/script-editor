// src/common/block_type.rs

use serde::{ Serialize, Deserialize };

/// Identifica la funzione drammaturgica ed espressiva di un blocco di testo.
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
    /// Note del Regista
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

    /// Deserializza una stringa nel corrispondente BlockType.
    /// Restituisce Option::None se la stringa non corrisponde ad alcun tipo valido.
    pub fn from_str(text: &str) -> Option<Self> {
        match text.trim() {
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

// Unit Test

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    /// Verifica che i metodi as_str e from_str siano perfettamente speculari
    /// per ogni singola variante dell'enum.
    #[test]
    fn test_string_conversion_symmetry() {
        let variants = vec![
            BlockType::Heading1,
            BlockType::Heading2,
            BlockType::Heading3,
            BlockType::StageDirection,
            BlockType::CharacterName,
            BlockType::Parenthetical,
            BlockType::Dialog,
            BlockType::Transition,
            BlockType::AuthorName,
            BlockType::ScriptVersion,
            BlockType::ProductionNote,
        ];

        for variant in variants {
            let string_representation = variant.as_str();
            let parsed_variant = BlockType::from_str(string_representation);

            assert!(
                parsed_variant.is_some(),
                "Il parsing ha fallito per la stringa: {}", string_representation
            );
            assert_eq!(
                parsed_variant.unwrap(),
                variant,
                "Asimmetria rilevata tra as_str e from_str per la variante: {:?}", variant
            );
        }
    }

    /// Verifica che il metodo from_str tolleri spazi bianchi accidentali (trim)
    /// e risponda correttamente con None in caso di stringhe sconosciute.
    #[test]
    fn test_from_str_edge_cases() {
        // Test con spazi bianchi attorno (grazie al .trim() che hai implementato)
        assert_eq!(BlockType::from_str("  dialog  "), Some(BlockType::Dialog));
        assert_eq!(BlockType::from_str("\ncharacter_name\t"), Some(BlockType::CharacterName));

        // Test stringhe non valide
        assert_eq!(BlockType::from_str("unknown_type"), None);
        assert_eq!(BlockType::from_str(""), None);
        assert_eq!(BlockType::from_str("DIALOG"), None); // Case sensitivity check
    }

    /// Verifica che Serde utilizzi correttamente i valori definiti negli attributi
    /// #[serde(rename = "...")] durante la serializzazione in JSON.
    #[test]
    fn test_serde_serialization() {
        let block = BlockType::StageDirection;
        let serialized = serde_json::to_string(&block).unwrap();

        // Ci aspettiamo che l'enum venga serializzata come stringa json "stage_direction"
        assert_eq!(serialized, "\"stage_direction\"");
    }

    /// Verifica che Serde sia in grado di ricostruire l'enum partendo da una stringa JSON
    /// in snake_case, intercettando errori di formato.
    #[test]
    fn test_serde_deserialization() {
        let json_data = "\"production_note\"";
        let deserialized: BlockType = serde_json::from_str(json_data).unwrap();

        assert_eq!(deserialized, BlockType::ProductionNote);

        // Verifica fallimento su JSON malformato o sconosciuto
        let invalid_json = "\"invalid_block_type\"";
        let result: Result<BlockType, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }
}