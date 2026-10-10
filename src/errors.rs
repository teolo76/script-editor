// src/errors.rs

//! Errori del core. Ogni tipo di errore concreto vive qui, non nel modulo
//! che lo genera: `types.rs` resta un contenitore di value types, `core/`
//! e `lake/` restano modello e storage, gli errori hanno un namespace loro.

use std::fmt;

// Package Modules
use crate::constants::{ ALIAS_MIN_LEN, ALIAS_MAX_LEN };
use crate::types::{ElementId, CharacterAlias};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Riferimento a un ElementId che non esiste nell'arena.
    ElementNotFound(ElementId),
    /// Un `ColorEncoding` non si è risolto in un `Color` valido.
    InvalidColor(ColorParseError),
    /// Una stringa non rispetta i vincoli di `CharacterAlias`.
    InvalidAlias(CharacterAliasError),
    /// L'alias è già assegnato a un altro personaggio.
    DuplicateAlias(CharacterAlias),
    /// Il nodo Root non può essere spostato: è l'unico elemento con
    /// `parent: None`, e non ha senso dargliene uno.
    CannotMoveRoot,
    /// Il nodo Root non può essere rimosso: cancellerebbe l'intero
    /// documento, non un elemento al suo interno.
    CannotRemoveRoot,
    /// Uno spostamento renderebbe un elemento discendente di se stesso
    /// (o lo renderebbe figlio diretto di se stesso).
    CyclicMove(ElementId),
    /// Placeholder generico, da sostituire con varianti specifiche
    /// man mano che implementiamo comandi/validazione.
    Invalid(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::ElementNotFound(id) => write!(f, "Elemento non trovato: {id}"),
            CoreError::InvalidColor(e) => write!(f, "Colore non valido: {}", e),
            CoreError::InvalidAlias(e) => write!(f, "Alias non valido: {}", e),
            CoreError::DuplicateAlias(e) => write!(f, "Alias già in uso: {}", e),
            CoreError::CannotMoveRoot => write!(f, "Il Nodo \'root\' non può essere spostato"),
            CoreError::CannotRemoveRoot =>write!(f, "Il Nodo \'root\' non può essere rimosso"),
            CoreError::CyclicMove(id) => write!(f, "Spostamento non valido. In conflitto con {}", id),
            CoreError::Invalid(err) => write!(f, "Operazione non valida: {err}"),
        }
    }
}

impl std::error::Error for CoreError {}

pub type CoreResult<T> = Result<T, CoreError>;

/// Errore nella risoluzione di un `types::ColorEncoding`: tipicamente un
/// valore esadecimale scritto male in un file di progetto o inserito
/// dall'utente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorParseError {
    /// Lunghezza della stringa esadecimale diversa da 6 (RGB) o 8 (RGBA),
    /// cancelletto escluso.
    InvalidHexLength(usize),
    InvalidHexDigits(String),
}

impl fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorParseError::InvalidHexLength(len) => {
                write!(f, "Lunghezza esadecimale non valida: {len} atteso 6 o 8 caratteri")
            }
            ColorParseError::InvalidHexDigits(s) => {
                write!(f, "Cifre esadecimali non valide: \"{s}\"")
            }
        }
    }
}

impl std::error::Error for ColorParseError {}

/// Errore di validazione di un `types::CharacterAlias`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterAliasError {
    /// Lunghezza fuori dall'intervallo [`crate::constants::ALIAS_MIN_LEN`,
    /// `crate::constants::ALIAS_MAX_LEN`].
    InvalidLength(usize),
    /// Contiene caratteri diversi da lettere/cifre ASCII.
    InvalidCharacters(String),
}

impl fmt::Display for CharacterAliasError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CharacterAliasError::InvalidLength(len) => write!(
                f,
                "Lunghezza Alias non valida: {len} atteso tra {ALIAS_MIN_LEN} e {ALIAS_MAX_LEN} caratteri"
            ),
            CharacterAliasError::InvalidCharacters(s) => write!(
                f,
                "Caratteri non validi in Alias: \"{s}\""
            ),
        }
    }
}

impl std::error::Error for CharacterAliasError {}


/// Permette di usare `?` su un `Result<_, ColorParseError>` dentro una
/// funzione che restituisce `CoreResult`, senza conversioni manuali — utile
/// non appena il loader del TOML dovrà propagare un colore malformato.
impl From<ColorParseError> for CoreError {
    fn from(err: ColorParseError) -> Self {
        CoreError::InvalidColor(err)
    }
}

impl From<CharacterAliasError> for CoreError {
    fn from(err: CharacterAliasError) -> Self {
        CoreError::InvalidAlias(err)
    }
}
