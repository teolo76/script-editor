// src/common/constants.rs

// --- PERSISTENZA E I/O ---
/// Magic Number di 4 byte in formato ASCII per identificare l'intestazione binaria del file ("SCPT").
pub const MAGIC_NUMBER: &[u8; 4] = b"SCPT";
/// Versione corrente dello schema di archiviazione binario/JSON.
pub const VERSION_NUMBER: u32 = 1;
/// Estensione primaria per i file di progetto.
pub const SCRIPT_EXT: &str = ".script";
/// Estensione secondaria/binaria supportata.
pub const SCPT_EXT: &str = ".scpt";
/// Elenco di tutte le estensioni valide supportate dai validatori di I/O.
pub const FILE_EXTENSIONS: &[&str] = &[SCRIPT_EXT, SCPT_EXT];

// --- STRUTTURA E VALORI DI DEFAULT DEL COPIONE ---
/// Titolo di default assegnato a un atto se non diversamente specificato.
pub const DEFAULT_ACT_TITLE: &str = "Atto Unico";
/// Titolo di default assegnato a una scena se non diversamente specificato.
pub const DEFAULT_SCENE_TITLE: &str = "Scena Senza Nome";

// --- DIMENSIONI METRICHE E TIPOGRAFICHE STANDARD (DPI 72) ---
/// Un pollice espresso in punti tipografici (72 pt = 1 in).
pub const POINTS_PER_INCH: f32 = 72.0;
/// Un millimetro espresso in punti tipografici (1 mm ≈ 2.8346457 pt).
pub const POINTS_PER_MM: f32 = 2.834_645_7;

// --- METRICHE DEL TESTO DI FALLBACK (Courier 12pt Monospace) ---
/// Altezza teorica di default per una riga di testo a spaziatura fissa (in pt).
pub const FALLBACK_LINE_HEIGHT: f32 = 12.0;
/// Larghezza teorica di un singolo carattere a spaziatura fissa Courier 12pt (in pt).
pub const FALLBACK_CHAR_WIDTH: f32 = 7.2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_number_and_version() {
        assert_eq!(MAGIC_NUMBER, b"SCPT");
        assert_eq!(MAGIC_NUMBER.len(), 4);
        assert!(VERSION_NUMBER > 0);
    }

    #[test]
    fn test_file_extensions() {
        assert_eq!(SCRIPT_EXT, ".script");
        assert_eq!(SCPT_EXT, ".scpt");
        assert_eq!(FILE_EXTENSIONS.len(), 2);
        assert!(FILE_EXTENSIONS.contains(&".script"));
        assert!(FILE_EXTENSIONS.contains(&".scpt"));
    }

    #[test]
    fn test_metric_conversions() {
        // Verifica che le proporzioni metriche di base siano corrette
        let one_inch_in_mm = 25.4;
        let calculated_points = one_inch_in_mm * POINTS_PER_MM;

        // Tolleranza per arrotondamenti su float
        assert!((calculated_points - POINTS_PER_INCH).abs() < 0.001);
    }

    #[test]
    fn test_defaults_not_empty() {
        assert!(!DEFAULT_ACT_TITLE.is_empty());
        assert!(!DEFAULT_SCENE_TITLE.is_empty());
        assert!(FALLBACK_LINE_HEIGHT > 0.0);
        assert!(FALLBACK_CHAR_WIDTH > 0.0);
    }
}