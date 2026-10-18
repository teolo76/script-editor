// src/errors.rs

//! Errori del core. Ogni tipo di errore concreto vive qui, non nel modulo
//! che lo genera: `types.rs` resta un contenitore di value types, `core/`
//! e `lake/` restano modello e storage, gli errori hanno un namespace loro.

use std::fmt;

use crate::types::{CharacterAlias, ElementId};

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
    /// `InsertText` con un offset oltre la fine del testo attuale del
    /// blocco. `offset` e `len` sono entrambi indici di carattere, non
    /// byte — coerenti con `AnchorTarget::TextRange`.
    TextOffsetOutOfBounds { block_id: ElementId, offset: usize, len: usize },
    /// `DeleteText` con un intervallo non valido: `start > end`, oppure
    /// `end` oltre la fine del testo. Anche qui indici di carattere.
    InvalidTextRange { block_id: ElementId, start: usize, end: usize, len: usize },
    /// `InsertText`/`DeleteText` su un `Block` che non è un `TextBlock`
    /// (es. un `AssetBlock`): non ha un `text` su cui operare.
    NotATextBlock(ElementId),
    /// Placeholder generico, da sostituire con varianti specifiche
    /// man mano che implementiamo comandi/validazione.
    Invalid(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::ElementNotFound(id) => write!(f, "elemento non trovato: {id}"),
            CoreError::InvalidColor(err) => write!(f, "colore non valido: {err}"),
            CoreError::InvalidAlias(err) => write!(f, "alias non valido: {err}"),
            CoreError::DuplicateAlias(alias) => write!(f, "alias già in uso: {alias}"),
            CoreError::CannotMoveRoot => write!(f, "il nodo Root non può essere spostato"),
            CoreError::CannotRemoveRoot => write!(f, "il nodo Root non può essere rimosso"),
            CoreError::CyclicMove(id) => {
                write!(f, "spostamento non valido: creerebbe un ciclo coinvolgendo {id}")
            }
            CoreError::TextOffsetOutOfBounds { block_id, offset, len } => write!(
                f,
                "offset {offset} fuori dal testo del blocco {block_id} (lunghezza {len} caratteri)"
            ),
            CoreError::InvalidTextRange { block_id, start, end, len } => write!(
                f,
                "intervallo [{start}, {end}) non valido sul blocco {block_id} (lunghezza {len} caratteri)"
            ),
            CoreError::NotATextBlock(id) => write!(f, "il blocco {id} non è un blocco di testo"),
            CoreError::Invalid(msg) => write!(f, "operazione non valida: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

/// Permette di usare `?` su un `Result<_, ColorParseError>` dentro una
/// funzione che restituisce `CoreResult`, senza conversioni manuali.
impl From<ColorParseError> for CoreError {
    fn from(err: ColorParseError) -> Self {
        CoreError::InvalidColor(err)
    }
}

/// Come sopra, per `CharacterAliasError`.
impl From<CharacterAliasError> for CoreError {
    fn from(err: CharacterAliasError) -> Self {
        CoreError::InvalidAlias(err)
    }
}

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
                write!(f, "lunghezza esadecimale non valida: {len} cifre (attese 6 o 8)")
            }
            ColorParseError::InvalidHexDigits(s) => {
                write!(f, "cifre esadecimali non valide: \"{s}\"")
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
                "lunghezza alias non valida: {len} caratteri (attesi {}-{})",
                crate::constants::ALIAS_MIN_LEN,
                crate::constants::ALIAS_MAX_LEN
            ),
            CharacterAliasError::InvalidCharacters(s) => {
                write!(f, "caratteri non validi nell'alias: \"{s}\"")
            }
        }
    }
}

impl std::error::Error for CharacterAliasError {}