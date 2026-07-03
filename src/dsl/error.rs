//! src/dsl/error.rs

use std::fmt;

/// Errori specifici del ciclo di vita del DSL (Lexing, Parsing ed Esecuzione).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DslError {
    /// Incontrato un carattere o un token non riconosciuto dalla grammatica caricata
    LexerUnknownToken(String),
    /// Un letterale stringa (apice singolo) è stato aperto ma mai chiuso prima della fine della query
    UnterminatedStringLiteral,
    /// La sintassi della query è invalida (es. `COUNT WHERE type = 'dialog'`, manca il target)
    ParserUnexpectedToken {
        expected: String,
        found: String,
    },
    /// La clausola WHERE fa riferimento a un campo che non esiste nel file di grammatica
    UnknownField(String),
    /// Il valore fornito non è compatibile con il tipo del campo (es. `WHERE words = 'molte'`, atteso un numero)
    TypeMismatch {
        field: String,
        expected_type: String,
    },
    /// Errore generico di fine stringa precoce durante il parsing di espressioni composte
    UnexpectedEndOfQuery,
}

impl fmt::Display for DslError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LexerUnknownToken(token) => write!(f, "Token o carattere non riconosciuto: '{}'", token),
            Self::UnterminatedStringLiteral => write!(f, "Sintassi invalida: stringa letterale aperta con (') ma mai chiusa"),
            Self::ParserUnexpectedToken { expected, found } => {
                write!(f, "Errore sintattico: atteso '{}', trovato '{}'", expected, found)
            }
            Self::UnknownField(field) => write!(f, "Il campo '{}' non esiste nella grammatica corrente", field),
            Self::TypeMismatch { field, expected_type } => {
                write!(f, "Mismatch di tipi sul campo '{}': atteso un valore di tipo {}", field, expected_type)
            }
            Self::UnexpectedEndOfQuery => write!(f, "La query si è interrotta bruscamente prima di completare l'istruzione"),
        }
    }
}

impl std::error::Error for DslError {}

