//! src/reporting/error.rs

use std::fmt;

// Package Modules & Files
use crate::dsl::error::DslError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportingError {
    /// Errore di sintassi o token non valido nel micro-DSL
    DslSyntaxError(String),
    /// La query DSL fa riferimento a un BlockType o a una colonna inesistente
    InvalidQueryTarget(String),
    /// Lo schema del template JSON è corrotto o incompleto
    InvalidTemplateSchema(String),
    /// Identificatore del template richiesto non trovato nel file system o nel provider
    TemplateNotFound(String),
    /// Errore durante l'aggregazione dei dati (es. operazioni matematiche fallite)
    AggregationFailed(String),
}

impl fmt::Display for ReportingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DslSyntaxError(msg) => write!(f, "Errore di sintassi DSL: {}", msg),
            Self::InvalidQueryTarget(target) => write!(f, "Target della query non valido: '{}'", target),
            Self::InvalidTemplateSchema(msg) => write!(f, "Schema del template JSON non valido: {}", msg),
            Self::TemplateNotFound(name) => write!(f, "Template di report '{}' non trovato.", name),
            Self::AggregationFailed(msg) => write!(f, "Fallimento durante l'aggregazione dei dati: {}", msg),
        }
    }
}

impl std::error::Error for ReportingError {}

impl From<DslError> for ReportingError {
    fn from(err: DslError) -> Self {
        ReportingError::DslSyntaxError(err.to_string())
    }
}