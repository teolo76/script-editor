//! src/engines/io/provider.rs

use std::fs::{ File, OpenOptions };
use std::io::{ Read, Write };
use std::path::{ Path };

use flate2::Compression;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;

// Package File & Modules
use crate::common::constants::{ MAGIC, IO_VERSION };
use crate::engines::io::{
    engine::ProjectFile,
    error::IoError
};

/// Contratto astratto per le operazioni di persistenza fisica del progetto.
pub trait IoProvider {
    /// Serializza, comprime e scrive il progetto sulla destinazione specificata
    fn save(&self, destination: &str, project: &ProjectFile) -> Result<(), IoError>;
    /// Legge, convalida e idrata il progetto dalla sorgente specificata
    fn load(&self, source: &str) -> Result<String, IoError>; // Restituisce la stringa JSON pulita per l'engine
}

/// Implementazione standard di produzione che lavora sul File System con compressione Deflate.
pub struct DiskIoProvider;

impl DiskIoProvider {
    pub fn new() -> Self { Self }
}

impl IoProvider for DiskIoProvider {
    fn save(&self, destination: &str, project: &ProjectFile) -> Result<(), IoError> {
        // 1. Serializzazione del ProjectFile in stringa JSON grezza
        let json_str = serde_json::to_string(project)
            .map_err(|_| IoError::InvalidJsonStructure)?;

        // 2. Compressione del payload JSON tramite Deflate
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(json_str.as_bytes()).map_err(|_| IoError::CompressionFailed)?;
        let compressed_payload = encoder.finish().map_err(|_| IoError::CompressionFailed)?;

        // 3. Creazione del file su disco
        let mut file = File::create(Path::new(destination))
            .map_err(|_| IoError::PermissionDenied)?;

        // 4. Scrittura dell'Header Binario (8 byte): Magic Bytes (4) + IO Version (4)
        file.write_all(MAGIC).map_err(|_| IoError::PermissionDenied)?;
        file.write_all(&IO_VERSION.to_be_bytes()).map_err(|_| IoError::PermissionDenied)?;

        // 5. Scrittura del payload compresso
        file.write_all(&compressed_payload).map_err(|_| IoError::PermissionDenied)?;

        file.flush().map_err(|_| IoError::PermissionDenied)?;
        Ok(())

    }
    fn load(&self, source: &str) -> Result<String, IoError> {
        // 1. Apertura del file in lettura
        let mut file = File::open(Path::new(source)).map_err(|_| IoError::FileNotFound)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|_| IoError::PermissionDenied)?;

        // 2. Validazione dei Magic Bytes (primi 4 byte)
        let file_magic = &buffer[0..4];
        if file_magic != MAGIC {
            return Err(IoError::InvalidJsonStructure);
        }

        // 3. Validazione della Versione I/O (successivi 4 byte)
        let mut version_bytes = [0u8; 4];
        version_bytes.copy_from_slice(&buffer[4..8]);
        let file_version = u32::from_be_bytes(version_bytes);

        if file_version != IO_VERSION {
            return Err(IoError::VersionMismatch {
                expected: IO_VERSION,
                found: file_version
            });
        }

        // 4. Estrazione e Decompressione del payload (dal byte 8 alla fine)
        let compressed_data = &buffer[8..];
        let mut decoder = DeflateDecoder::new(compressed_data);
        let mut json_str = String::new();
        decoder.read_to_string(&mut json_str).map_err(|_| IoError::DecompressionFailed)?;

        Ok(json_str)
    }
}

// =========================================================================
// UNIT TESTS DI PERSISTENZA E IN-MEMORY MOCK
// =========================================================================

#[cfg(test)]
mod persistence_tests {
    use super::*;
    use crate::elements::Script;
    use crate::engines::io::engine::DocSection;
    use std::collections::HashMap;

    #[test]
    fn test_disk_provider_roundtrip_and_headers() {
        let test_path = "test_output_project.script";

        // Costruiamo un ProjectFile minimo di test
        let script = Script::new("Giallo Minerale", "Autore Anonimo");
        let doc = DocSection {
            script,
            acts_pool: HashMap::new(),
            scenes_pool: HashMap::new(),
            blocks_pool: HashMap::new(),
            text_arena: HashMap::new(),
        };
        let project = ProjectFile { doc, payload: Vec::new() };

        let provider = DiskIoProvider::new();

        // 1. Salvataggio su disco
        let save_res = provider.save(test_path, &project);
        assert!(save_res.is_ok());

        // 2. Controllo manuale dell'integrità dell'header del file fisico
        let mut raw_file = File::open(test_path).unwrap();
        let mut header_buffer = [0u8; 8];
        raw_file.read_exact(&mut header_buffer).unwrap();

        assert_eq!(&header_buffer[0..4], b"SCRP"); // Magic bytes intatti
        assert_eq!(u32::from_be_bytes([header_buffer[4], header_buffer[5], header_buffer[6], header_buffer[7]]), 1); // Versione 1

        // 3. Caricamento e Decompressione simmetrica tramite Provider
        let load_res = provider.load(test_path);
        assert!(load_res.is_ok());

        let json_string = load_res.unwrap();
        assert!(json_string.contains("Giallo Minerale")); // Il dato è sopravvissuto intatto

        // Pulizia del file di test
        std::fs::remove_file(test_path).unwrap();
    }
}