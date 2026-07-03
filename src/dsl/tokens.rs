//! src/dsl/tokens.rs

use std::fmt;

// Package Modules & Files
use crate::common::BlockType;

/// Rappresenta le macro-operazioni estratte dalla query DSL
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DslCommand {
    Count,
    Sum,
    Get,
}

/// Rappresenta le entità o collezioni target dell'interrogazione
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DslTarget {
    Acts,
    Blocks,
    Scenes,
    Characters,
}

/// Rappresenta i campi strutturali su cui applicare le clausole di filtraggio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DslField {
    BlockType,
    WordCount,
    CharacterName,
    LineCount,
}

/// Rappresenta gli operatori logici e matematici di confronto
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DslOperator {
    Eq, // =
    Gt, // >
    Lt, // <
}

/// L'alberatura atomica dei Token del Lexer (Completamente Zero-Copy)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DslToken<'a> {
    // Comandi e Target
    Command(DslCommand),
    Target(DslTarget),

    // Clausole e Connettivi Logici
    Where,
    And,
    Or,
    Not,

    // Campi e Operatori di Confronto
    Field(DslField),
    Operator(DslOperator),

    // Valori Letterali Dinamici (Zero-Allocation Slices)
    LiteralString(&'a str), // Riferimento diretto racchiuso tra '...' nella query
    LiteralNumber(u32),     // Numero già convertito in-place nello stack
}

impl<'a> fmt::Display for DslToken<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Command(cmd) => write!(f, "Command({:?})", cmd),
            Self::Target(tgt) => write!(f, "Target({:?})", tgt),
            Self::Where => write!(f, "WHERE"),
            Self::And => write!(f, "AND"),
            Self::Or => write!(f, "OR"),
            Self::Not => write!(f, "NOT"),
            Self::Field(fld) => write!(f, "Field({:?})", fld),
            Self::Operator(op) => write!(f, "Operator({:?})", op),
            Self::LiteralString(val) => write!(f, "LiteralString('{}')", val),
            Self::LiteralNumber(num) => write!(f, "LiteralNumber({})", num),
        }
    }
}