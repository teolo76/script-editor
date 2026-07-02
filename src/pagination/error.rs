//! src/pagination/error.rs

use std::fmt;

/// Errori specifici del sottosistema metrico dei font.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontMetricError {
    FontNotFound(String),
    TextEvaluationFailed,
    InvalidMetrics,
}
impl fmt::Display for FontMetricError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FontNotFound(name) => write!(f, "Font '{}' non trovato nel sistema.", name),
            Self::TextEvaluationFailed => write!(f, "Impossibile calcolare le metriche del testo."),
            Self::InvalidMetrics => write!(f, "I calcoli restituiti dal provider non sono geometricamente validi."),
        }
    }
}

impl std::error::Error for FontMetricError {}