
pub const DEFAULT_ACT_TITLE: &str = "Atto Unico";
pub const MAX_SCENE_PER_ACT: usize = 64;
pub const DEFAULT_SCENE_TITLE: &str = "Scena Senza Nome";
pub const MAX_BLOCKS_PER_SCENE: usize = 128;
pub const MAX_CHARACTERS_PER_SCENE: usize = 32;
pub const MAX_SCENE_DURATION_MINUTES: f32 = 100.0;
pub const MAX_ACTS_PER_SCRIPT: usize = 64;
pub const MAX_CHARACTERS_PER_SCRIPT: usize = 256;
pub const MAX_COVER_BLOCKS: usize = 8;
/// Firma binaria fissa di 4 byte posizionata all'inizio del file per identificare il formato.script
pub const MAGIC: &[u8; 4] = b"SCRP";
/// Versione dello schema di archiviazione corrente dell'I/O Engine
pub const IO_VERSION: u32 = 1;
pub const SCRIPT_EXT: &str = ".script";
pub const QUERY_EXT: &str = ".qscript";