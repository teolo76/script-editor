// src/elements/payloads.rs

use serde::{Deserialize, Serialize};

/// Payload informativo per le note di produzione/reparto collegate a un personaggio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepartmentNoteData {
    /// Identificativo del reparto (es: "costumes", "props", "makeup", "acting")
    pub department: String,
    /// Testo della nota di reparto
    pub text: String,
}

impl DepartmentNoteData {
    pub fn new(department: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            department: department.into().trim().to_lowercase(),
            text: text.into().trim().to_string(),
        }
    }
}

/// Payload generico di testo per i blocchi drammaturgici (Dialoghi, Didascalie, Personaggi, Titoli, ecc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPayload {
    pub text: String,
}

impl TextPayload {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
        }
    }
}

/// Payload per riferimenti ad asset grafici o multimediali (es. loghi o immagini sulla copertina).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetPayload {
    /// Percorso relativo o risorsa dell'asset
    pub source_path: String,
    /// Didascalia o testo alternativo
    pub alt_text: String,
}

impl AssetPayload {
    pub fn new(source_path: impl Into<String>, alt_text: impl Into<String>) -> Self {
        Self {
            source_path: source_path.into(),
            alt_text: alt_text.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_note_data() {
        let note = DepartmentNoteData::new("  COSTUMES  ", " Tunica scura ");
        assert_eq!(note.department, "costumes");
        assert_eq!(note.text, "Tunica scura");
    }

    #[test]
    fn test_text_payload() {
        let payload = TextPayload::new("Essere o non essere");
        assert_eq!(payload.text, "Essere o non essere");
    }

    #[test]
    fn test_serde_serialization() {
        let note = DepartmentNoteData::new("props", "Fazzoletto ricamato");
        let json = serde_json::to_string(&note).expect("Serializzazione fallita");
        let deserialized: DepartmentNoteData = serde_json::from_str(&json).expect("Deserializzazione fallita");

        assert_eq!(note, deserialized);
    }
}