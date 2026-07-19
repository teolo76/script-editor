// src/errors/mod.rs

use std::fmt;

/// Rappresenta lo spettro completo delle anomalie e delle violazioni logiche
/// che possono verificarsi all'interno dell'ecosistema del copione.
#[derive(Debug)]
pub enum ScriptError {
    // --- ERRORI DI I/O E PERSISTENZA ---
    /// Errore nativo del File System (file mancante, permessi negati, ecc.).
    Io(std::io::Error),

    /// Il file aperto non inizia con il Magic Number corretto.
    InvalidMagicNumber,

    /// L'estensione del file non rientra tra quelle permesse.
    InvalidExtension,

    /// La versione del file è superiore a quella supportata dal software corrente.
    UnsupportedVersion { found: u32, current: u32 },

    // --- ERRORI DI DECOMPRESSIONE E PARSING (JSON / ZSTD / CONFIG) ---
    /// Il payload compresso o la struttura JSON del file .scpt è corrotta o malformata.
    SerializationDataCorrupted(String),

    /// Il file JSON di un Tema esterno o di un Formato Pagina contiene errori di sintassi o campi mancanti.
    ThemeParseError(String),

    // --- VIOLAZIONI DELLA GERARCHIA SEMANTICA (Albero del Documento) ---
    /// Si è tentato di inserire un elemento orfano o di creare un ciclo gerarchico invalido.
    HierarchyViolation(String),

    /// L'UUID cercato per una modifica o una rimozione non esiste nell'albero corrente.
    NodeNotFound([u8; 16]),

    // --- ERRORI DEL MOTORE GEOMETRICO ---
    /// Il PaginationProvider ha riscontrato un overflow numerico o un calcolo geometrico impossibile.
    PaginationFailure(String),
}

// Implementazione del trait Standard Display per consentire la stampa dell'errore (es. in console o log)
impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::Io(err) => write!(
                f,
                "Errore di I/O del File System: {}",
                err
            ),
            ScriptError::InvalidMagicNumber => write!(
                f,
                "File non valido: Intestazione (Magic Number) mancante o corrotta."
            ),
            ScriptError::InvalidExtension => write!(f, "Estensione del file non supportata."),
            ScriptError::UnsupportedVersion { found, current } => write!(
                f,
                "Versione del file non supportata (Trovata: {}, Corrente software: {}).",
                found, current
            ),
            ScriptError::SerializationDataCorrupted(msg) => write!(
                f,
                "Dati del copione corrotti o impossibili da deserializzare: {}",
                msg
            ),
            ScriptError::ThemeParseError(msg) => write!(
                f,
                "Impossibile caricare il tema JSON esterno: {}",
                msg
            ),
            ScriptError::HierarchyViolation(msg) => write!(
                f,
                "Violazione della struttura del copione: {}",
                msg
            ),
            ScriptError::NodeNotFound(uuid) => write!(
                f,
                "Nodo non trovato nell'albero dello script (UUID: {:?}).",
                uuid
            ),
            ScriptError::PaginationFailure(msg) => write!(
                f,
                "Fallimento critico nell'impaginazione geometrica: {}",
                msg
            ),
        }
    }
}

// Integrazione con il trait standard di gestione degli errori di Rust
impl std::error::Error for ScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ScriptError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Costruttore helper automatico (pattern From) per convertire istantaneamente gli std::io::Error in ScriptError::Io
impl From<std::io::Error> for ScriptError {
    fn from(err: std::io::Error) -> Self {
        ScriptError::Io(err)
    }
}