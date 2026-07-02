//! src/io/provider.rs

use std::fs::{ File, OpenOptions };
use std::io::{ Read, Write };
use std::path::{ Path, PathBuf };

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

// Package File & Modules
use crate::common::constants::{ MAGIC, IO_VERSION };
use crate::io::{
    engine::ProjectFile,
    error::IoError,
    migration::MigrationProvider
};

/// Contratto astratto per le operazioni di persistenza fisica del progetto.
pub trait IoProvider {
    /// Serializza, comprime e scrive il progetto sulla destinazione specificata
    fn save(&self, destination: &str, project: &ProjectFile) -> Result<(), IoError>;
    /// Legge, convalida e idrata il progetto dalla sorgente specificata
    /// Restituisce la stringa JSON pulita per l'engine
    fn load(&self, source: &str) -> Result<String, IoError>;
}

/// Implementazione standard di produzione che lavora sul File System con compressione Deflate.
pub struct DiskIoProvider {
    schemas_dir: PathBuf,
}

impl DiskIoProvider {
    /// Il costruttore ora richiede il path della cartella degli schemi (es. AppData di Tauri)
    pub fn new<P: AsRef<Path>>(schemas_dir: P) -> Self {
        Self {
            schemas_dir: schemas_dir.as_ref().to_path_buf(),
        }
    }
}
impl IoProvider for DiskIoProvider {

    fn save(&self, destination: &str, project: &ProjectFile) -> Result<(), IoError> {
        let json_str = serde_json::to_string(project).map_err(|_| IoError::InvalidJsonStructure)?;

        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_str.as_bytes()).map_err(|_| IoError::CompressionFailed)?;
        let compressed_payload = encoder.finish().map_err(|_| IoError::CompressionFailed)?;

        let mut file = File::create(Path::new(destination)).map_err(|_| IoError::PermissionDenied)?;

        file.write_all(MAGIC).map_err(|_| IoError::PermissionDenied)?;
        file.write_all(&IO_VERSION.to_be_bytes()).map_err(|_| IoError::PermissionDenied)?;
        file.write_all(&compressed_payload).map_err(|_| IoError::PermissionDenied)?;

        file.flush().map_err(|_| IoError::PermissionDenied)?;
        Ok(())
    }
    fn load(&self, source: &str) -> Result<String, IoError> {
        // 1. Lettura del file grezzo da disco
        let mut file = File::open(Path::new(source)).map_err(|_| IoError::FileNotFound)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|_| IoError::PermissionDenied)?;

        if buffer.len() < 8 {
            return Err(IoError::InvalidJsonStructure);
        }

        // 2. Validazione accoppiata dell'Header Binario
        let file_magic = &buffer[0..4];
        if file_magic != MAGIC {
            return Err(IoError::InvalidJsonStructure);
        }

        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&buffer[4..8]);
        let file_version = u32::from_be_bytes(version_bytes);

        // 3. Estrazione e Decompressione del payload
        let compressed_data = &buffer[8..];
        let mut decoder = DeflateDecoder::new(compressed_data);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).map_err(|_| IoError::DecompressionFailed)?;

        // 4. INTERCETTAZIONE LOGICA DELLA MIGRAZIONE
        if file_version < IO_VERSION {
            // Inizializziamo al volo il MigrationProvider con la cartella di fallback/Tauri
            let mut migration_engine = MigrationProvider::new();
            migration_engine.initialize_with_directory(&self.schemas_dir)?;

            // Convertiamo la stringa JSON decompressa in un albero dinamico
            let raw_json_value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|_| IoError::InvalidJsonStructure)?;

            // Eseguiamo la migrazione automatica a cascata guidata dai dati
            let migrated_value = migration_engine.migrate(raw_json_value, file_version, IO_VERSION)?;

            // Ritrasformiamo l'albero sanato in stringa JSON pronta per l'IoEngine
            json_str = serde_json::to_string(&migrated_value).map_err(|_| IoError::InvalidJsonStructure)?;
        } else if file_version > IO_VERSION {
            // Il file è più recente del software attuale (l'utente deve aggiornare l'app)
            return Err(IoError::VersionMismatch {
                expected: IO_VERSION,
                found: file_version
            });
        }

        Ok(json_str)
    }
}

// =========================================================================
// UNIT TESTS DI PERSISTENZA E IN-MEMORY MOCK
// =========================================================================
#[cfg(test)]
mod integrated_persistence_tests {
    use super::*;
    use serde_json::json;
    use std::env;
    use std::fs;

    #[test]
    fn test_disk_provider_automatic_migration_on_load() {
        let mut base_test_dir = env::temp_dir();
        base_test_dir.push("tauri_integrated_provider_test");

        if base_test_dir.exists() {
            let _ = fs::remove_dir_all(&base_test_dir);
        }
        fs::create_dir_all(&base_test_dir).unwrap();

        let schemas_directory = base_test_dir.join("schemas");
        let project_file_path = base_test_dir.join("legacy_project.script");

        // 1. Prepariamo il file legacy v0 compressato (Usiamo 0 così è sicuramente inferiore a IO_VERSION che è 1)
        let legacy_json = json!({
            "doc": {
                "script": { "title": "Re Lear", "author": "Shakespeare" },
                "scenes_pool": {
                    "sc-10": { "number": 10, "title": "La Tempesta" }
                },
                "acts_pool": {},
                "blocks_pool": {},
                "text_arena": {}
            },
            "payload": []
        });

        let json_str = serde_json::to_string(&legacy_json).unwrap();
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_str.as_bytes()).unwrap();
        let compressed_payload = encoder.finish().unwrap();

        {
            let mut file = File::create(&project_file_path).unwrap();
            file.write_all(MAGIC).unwrap();
            file.write_all(&0u32.to_be_bytes()).unwrap(); // <--- SCRIVIAMO VERSIONE 0
            file.write_all(&compressed_payload).unwrap();
            file.flush().unwrap();
        }

        // 2. Prepariamo lo schema di migrazione da 0 a 1 (l'attuale IO_VERSION) sul disco dei test
        fs::create_dir_all(&schemas_directory).unwrap();
        let migration_schema_json = json!({
            "from_version": 0,
            "to_version": 1,
            "target_rules": {
                "scenes": {
                    "inject_defaults": {
                        "duration_minutes": 15.5 // Valore riconoscibile per il test
                    }
                }
            }
        });

        let schema_file_path = schemas_directory.join("v0_to_v1.json");
        let mut schema_file = File::create(schema_file_path).unwrap();
        schema_file.write_all(serde_json::to_string_pretty(&migration_schema_json).unwrap().as_bytes()).unwrap();
        schema_file.flush().unwrap();
        drop(schema_file);

        // 3. Inizializziamo il provider puntando alla cartella degli schemi creata
        let provider = DiskIoProvider::new(&schemas_directory);

        // 4. Caricamento: intercetta la versione 0, legge lo schema v0_to_v1 e migra alla versione attuale (1)
        let load_result = provider.load(project_file_path.to_str().unwrap());
        assert!(load_result.is_ok(), "Il caricamento ha fallito l'I/O base");

        let processed_json_str = load_result.unwrap();
        let final_value: serde_json::Value = serde_json::from_str(&processed_json_str).unwrap();

        // 5. Validazione finale dell'iniezione
        let duration = final_value["doc"]["scenes_pool"]["sc-10"]["duration_minutes"].as_f64();

        assert!(duration.is_some(), "Il campo 'duration_minutes' non è stato iniettato!");
        assert_eq!(duration.unwrap(), 15.5);

        // Cleanup
        let _ = fs::remove_dir_all(&base_test_dir);
    }
}