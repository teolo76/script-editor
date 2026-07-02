//! src/io/error.rs
use uuid::Uuid;
use serde::{ Serialize, Deserialize };
use std::fmt;

/// Errori specifici del sottosistema di I/O.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoError {
    FileNotFound,
    PermissionDenied,
    InvalidJsonStructure,
    DecompressionFailed,
    CompressionFailed,
    VersionMismatch { expected: u32, found: u32 },
    MissingScriptRoot,
    BlockNotFound(Uuid),
    PresetNotFound(String),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound => write!(f, "File di progetto non trovato."),
            Self::PermissionDenied => write!(f, "Permessi insufficienti per accedere al file."),
            Self::InvalidJsonStructure => write!(f, "Struttura JSON corrotta o non valida."),
            Self::DecompressionFailed => write!(f, "Decompressione del file fallita."),
            Self::CompressionFailed => write!(f, "Compressione del file fallita."),
            Self::VersionMismatch { expected, found } => write!(f, "Incompatibilità versione: attesa v{}, trovata v{}.", expected, found),
            Self::MissingScriptRoot => write!(f, "Radice 'Script' mancante nella sezione doc."),
            Self::BlockNotFound(id) => write!(f, "Blocco di testo {} non trovato nell'arena.", id),
            Self::PresetNotFound(name) => write!(f, "Preset di configurazione '{}' non trovato.", name),
        }
    }
}

impl std::error::Error for IoError {}

//*************************************************************************************************
//* UNIT TEST
//*************************************************************************************************

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_io_error_display_formatting() {
        // Test errore semplice
        let err_file = IoError::FileNotFound;
        assert_eq!(err_file.to_string(), "File di progetto non trovato.");

        // Test errore parametrizzato con UUID
        let test_uuid = Uuid::new_v4();
        let err_block = IoError::BlockNotFound(test_uuid);
        assert!(err_block.to_string().contains(&test_uuid.to_string()));

        // Test errore di mismatch versione
        let err_version = IoError::VersionMismatch { expected: 2, found: 1 };
        assert_eq!(err_version.to_string(), "Incompatibilità versione: attesa v2, trovata v1.");
    }
}