// src/common/constants.rs

// --- I/O ---
/// Magic number di 4 byte in formato ASCII per identificare istantaneamente
/// l'intestazione del file "SCPT".
pub const MAGIC_NUMBER: &[u8; 4] = b"SCPT";

/// Versione corrente dello schema di archiviazione binario/JSON del file
/// fondamentale per la retrocompatibilità).
pub const VERSION_NUMBER: u32 = 1;

/// Elenco completo delle estensioni di file valide supportate dall'applicazione.
/// Utilizzato dai moduli di I/O e dai validatori nativi per verificare l'integrità
/// dei percorsi dei file.
pub const FILE_EXTENSIONS: &[&str] = &["script", "scpt"];

// --- STRUTTURA E VALORI DI DEFAULT DEL COPIONE ---
pub const DEFAULT_ACT_TITLE: &str = "Atto Unico";
pub const DEFAULT_SCENE_TITLE: &str = "Scena Senza Nome";

pub const MAX_SCENE_PER_ACT: usize = 64;
pub const MAX_BLOCKS_PER_SCENE: usize = 128;
pub const MAX_CHARACTERS_PER_SCENE: usize = 32;
pub const MAX_SCENE_DURATION_MINUTES: f32 = 100.0;
pub const MAX_ACTS_PER_SCRIPT: usize = 64;
pub const MAX_CHARACTERS_PER_SCRIPT: usize = 256;
pub const MAX_COVER_BLOCKS: usize = 8;

// --- DIMENSIONI METRICHE DI BASE (Standard Industriale) ---
/// Un pollice espresso in punti tipografici
/// (DPI standard per il calcolo geometrico interno: 72pt = 1in).
pub const POINTS_PER_INCH: f32 = 72.0;

/// Un millimetro espresso in punti tipografici
/// (utilizzato per la conversione dei formati ISO come l'A4).
pub const POINTS_PER_MM: f32 = 2.834_645_7;


// --- METRICHE DEL TESTO DI RIPIEGO (Fallback Engine) ---
// Queste costanti intervengono solo se il caricamento dei JSON esterni fallisce,
// garantendo che il motore di calcolo non vada mai in crash.

/// Altezza teorica di default per una riga di testo a spaziatura fissa (Monospace Courier 12pt).
pub const FALLBACK_LINE_HEIGHT: f32 = 12.0;

/// Larghezza teorica occupata da un singolo carattere a spaziatura fissa Courier 12pt.
pub const FALLBACK_CHAR_WIDTH: f32 = 7.2;