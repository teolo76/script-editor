//! src/io/migration.rs

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json::Value;

// Package Files & Modules
use crate::io::error::IoError;

/// Definisce le azioni di mutazione strutturale da applicare a un percorso JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SchemaRule {
    #[serde(default)]
    pub rename_fields: HashMap<String, String>,
    #[serde(default)]
    pub remove_fields: Vec<String>,
    #[serde(default)]
    pub inject_defaults: HashMap<String, Value>,
}

impl SchemaRule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply(&self, obj: &mut serde_json::Map<String, Value>) {
        // 1. Ridenominazione dei campi
        for (old_name, new_name) in &self.rename_fields {
            if let Some(value) = obj.remove(old_name) {
                obj.insert(new_name.clone(), value);
            }
        }

        // 2. Rimozione dei campi obsoleti
        for field in &self.remove_fields {
            obj.remove(field);
        }

        // 3. Iniezione dei valori di default per i nuovi campi
        for (field, default_value) in &self.inject_defaults {
            if !obj.contains_key(field) {
                obj.insert(field.clone(), default_value.clone());
            }
        }
    }
}

/// Rappresenta lo schema di transizione per un singolo salto di versione (es: v1 -> v2).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionTransition {
    pub from_version: u32,
    pub to_version: u32,
    pub target_rules: HashMap<String, SchemaRule>,
}

impl VersionTransition {
    pub fn new(from: u32, to: u32) -> Self {
        Self {
            from_version: from,
            to_version: to,
            target_rules: HashMap::new(),
        }
    }
}

/// Il provider centrale che gestisce il ciclo di vita dei template di migrazione (Embedded e su Disco).
#[derive(Debug, Clone, Default)]
pub struct MigrationProvider {
    pub transitions: HashMap<u32, VersionTransition>,
}

// In src/engine/migration.rs

impl MigrationProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            transitions: HashMap::new(),
        };
        // CORREZIONE INDUSTRIALE: Carica SEMPRE i preset embedded in memoria all'istante
        provider.load_embedded_presets();
        provider
    }
    /// Pre-carica i template di fabbrica direttamente in memoria a costo zero
    fn load_embedded_presets(&mut self) {
        let mut t_1_to_2 = VersionTransition::new(1, 2);
        let mut scene_rule = SchemaRule::new();
        scene_rule.inject_defaults.insert("duration_minutes".to_string(), Value::from(0.0));
        t_1_to_2.target_rules.insert("scenes".to_string(), scene_rule);

        self.transitions.insert(t_1_to_2.from_version, t_1_to_2);
    }
    pub fn initialize_with_directory(&mut self, target_dir: &Path) -> Result<(), IoError> {
        if !target_dir.exists() {
            fs::create_dir_all(target_dir).map_err(|_| IoError::PermissionDenied)?;
        }

        let is_empty = fs::read_dir(target_dir)
            .map_err(|_| IoError::PermissionDenied)?
            .next()
            .is_none();

        if is_empty {
            self.deploy_embedded_schemas(target_dir)?;
        }

        // Carica i file esterni che andranno a integrare o sovrascrivere quelli embedded
        self.load_from_directory(target_dir)?;

        Ok(())
    }
    fn deploy_embedded_schemas(&self, target_dir: &Path) -> Result<(), IoError> {
        let v1_to_v2_embedded = r#"{
            "from_version": 1,
            "to_version": 2,
            "target_rules": {
                "scenes": {
                    "inject_defaults": {
                        "duration_minutes": 0.0
                    }
                }
            }
        }"#;

        let file_path = target_dir.join("v1_to_v2.json");
        let mut file = File::create(file_path).map_err(|_| IoError::PermissionDenied)?;
        file.write_all(v1_to_v2_embedded.as_bytes()).map_err(|_| IoError::PermissionDenied)?;
        file.flush().map_err(|_| IoError::PermissionDenied)?;
        drop(file);
        Ok(())
    }
    /// Scansiona la directory dei template accumulando le transizioni dinamiche (sovrascrive eventuali conflitti)
    pub fn load_from_directory(&mut self, dir_path: &Path) -> Result<(), IoError> {
        let entries = fs::read_dir(dir_path).map_err(|_| IoError::FileNotFound)?;

        for entry in entries {
            let entry = entry.map_err(|_| IoError::PermissionDenied)?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json_str = fs::read_to_string(&path).map_err(|_| IoError::PermissionDenied)?;
                let transition: VersionTransition = serde_json::from_str(&json_str)
                    .map_err(|_| IoError::InvalidJsonStructure)?;

                // Registrazione dinamica guidata dai dati nel grafo delle migrazioni
                self.transitions.insert(transition.from_version, transition);
            }
        }

        Ok(())
    }
    /// Esegue la pipeline di migrazione incrementale trasformando l'albero JSON grezzo
    pub fn migrate(&self, mut json_value: Value, from_version: u32, target_version: u32) -> Result<Value, IoError> {
        let mut current_version = from_version;

        while current_version < target_version {
            let transition = self.transitions.get(&current_version)
                .ok_or_else(|| IoError::VersionMismatch { expected: target_version, found: from_version })?;

            if let Some(root_obj) = json_value.as_object_mut() {
                // 1. Regole sulla radice globale
                if let Some(rule) = transition.target_rules.get("root") {
                    rule.apply(root_obj);
                }

                // Navigazione interna alla sezione "doc"
                if let Some(doc) = root_obj.get_mut("doc").and_then(|d| d.as_object_mut()) {

                    // 2. Regole applicate a tappeto nel pool delle scene
                    if let Some(rule) = transition.target_rules.get("scenes") {
                        if let Some(scenes_pool) = doc.get_mut("scenes_pool").and_then(|s| s.as_object_mut()) {
                            for (_, scene_val) in scenes_pool.iter_mut() {
                                if let Some(scene_obj) = scene_val.as_object_mut() {
                                    rule.apply(scene_obj);
                                }
                            }
                        }
                    }

                    // 3. Regole applicate a tappeto nel pool dei blocchi
                    if let Some(rule) = transition.target_rules.get("blocks") {
                        if let Some(blocks_pool) = doc.get_mut("blocks_pool").and_then(|b| b.as_object_mut()) {
                            for (_, block_val) in blocks_pool.iter_mut() {
                                if let Some(block_obj) = block_val.as_object_mut() {
                                    rule.apply(block_obj);
                                }
                            }
                        }
                    }
                }
            }

            current_version = transition.to_version;
        }

        Ok(json_value)
    }
}

// =========================================================================
// UNIT TESTS DI PERSISTENZA E IN-MEMORY MOCK
// =========================================================================

#[cfg(test)]
mod data_driven_migration_tests {
    use super::*;
    use serde_json::json;
    use std::env;

    #[test]
    fn test_serialization_and_deserialization_of_schemas() {
        // Verifica che lo schema sia interamente descrivibile via JSON e manipolabile da Serde
        let raw_schema = r#"{
            "from_version": 1,
            "to_version": 2,
            "target_rules": {
                "scenes": {
                    "rename_fields": {"vecchio": "nuovo"},
                    "remove_fields": ["obsoleto"],
                    "inject_defaults": {"attivo": true}
                }
            }
        }"#;

        let deserialized: Result<VersionTransition, _> = serde_json::from_str(raw_schema);
        assert!(deserialized.is_ok());

        let transition = deserialized.unwrap();
        assert_eq!(transition.from_version, 1);
        assert_eq!(transition.to_version, 2);

        let rule = transition.target_rules.get("scenes").unwrap();
        assert_eq!(rule.rename_fields.get("vecchio").unwrap(), "nuovo");
        assert_eq!(rule.remove_fields[0], "obsoleto");
        assert_eq!(rule.inject_defaults.get("attivo").unwrap().as_bool().unwrap(), true);
    }

    #[test]
    fn test_provider_initialization_cascade_and_fallback() {
        // 1. Setup di una directory temporanea pulita per simulare l'ambiente runtime di Tauri
        let mut temp_dir = env::temp_dir();
        temp_dir.push("tauri_app_schemas_test_dir");
        if temp_dir.exists() {
            fs::remove_dir_all(&temp_dir).unwrap();
        }

        let mut provider = MigrationProvider::new();

        // 2. Prima esecuzione: la directory è vuota, il provider deve attivare il fallback embedded
        let init_res = provider.initialize_with_directory(&temp_dir);
        assert!(init_res.is_ok());

        // 3. Verifica che il file "v1_to_v2.json" sia stato effettivamente scritto su disco dal fallback
        let expected_file = temp_dir.join("v1_to_v2.json");
        assert!(expected_file.exists());

        // 4. Verifica che il grafo a runtime contenga la transizione caricata dal file system generato
        assert!(provider.transitions.contains_key(&1));

        // 5. Iniettiamo un secondo file manuale nella directory (v2_to_v3.json) per testare l'estendibilità dinamica
        let v2_to_v3_custom = json!({
            "from_version": 2,
            "to_version": 3,
            "target_rules": {
                "blocks": {
                    "inject_defaults": { "color": "#000000" }
                }
            }
        });

        let custom_file_path = temp_dir.join("v2_to_v3.json");
        let mut file = File::create(custom_file_path).unwrap();
        file.write_all(serde_json::to_string_pretty(&v2_to_v3_custom).unwrap().as_bytes()).unwrap();

        // Ricarichiamo dalla directory per simulare l'estensione dinamica via template esterni
        let reload_res = provider.load_from_directory(&temp_dir);
        assert!(reload_res.is_ok());
        assert!(provider.transitions.contains_key(&2)); // Ora abbiamo sia la v1 che la v2 dinamica!

        // 6. Test di migrazione reale a cascata sul JSON v1 grezzo
        let old_project = json!({
            "doc": {
                "script": { "title": "Otello" },
                "scenes_pool": {
                    "sc-1": { "number": 5 }
                },
                "blocks_pool": {
                    "bk-1": { "text": "Atto I" }
                }
            }
        });

        let migration_result = provider.migrate(old_project, 1, 3);
        assert!(migration_result.is_ok());

        let migrated_json = migration_result.unwrap();

        // Convalida iniezioni incrociate (v1_to_v2 embedded + v2_to_v3 caricata esternamente)
        assert_eq!(migrated_json["doc"]["scenes_pool"]["sc-1"]["duration_minutes"].as_f64().unwrap(), 0.0);
        assert_eq!(migrated_json["doc"]["blocks_pool"]["bk-1"]["color"].as_str().unwrap(), "#000000");

        // Cleanup finale
        fs::remove_dir_all(&temp_dir).unwrap();
    }
}
