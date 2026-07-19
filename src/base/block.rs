// src/base/block.rs

use serde::{ Serialize, Deserialize };
use uuid::Uuid;

// Package Modules
use crate::base::{
    Meta,
    VisualLayer,
};
use crate::common::BlockType;
use crate::geometry::BlockGeometry;
use crate::traits::{
    BaseElement,
    BlockBehavior,
    DocumentNode,
    LayeredElement,
};

/// Struttura dati di base (dati di pancia) che fornisce l'infrastruttura comune
/// a tutti i blocchi foglia terminali (TextBlock, FloatingBlock).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct BlockBase {
    /// Identità e tracciamento temporale dell'elemento.
    pub meta: Meta,

    /// Il blocco della grafica e dei canali di visibilità visivi.
    pub visual: VisualLayer,

    /// Geometria di formattazione (rientri e spaziature in millimetri).
    pub geometry: BlockGeometry,

    /// Tipologia espressiva e drammaturgica del blocco (es: BlockType::Dialog).
    pub block_type: BlockType,

    /// L'UUID del contenitore padre (es: l'UUID della Scene o della Cover).
    pub parent_id: Uuid,

    /// Indice di ordinamento posizionale all'interno dello stesso contenitore.
    pub sort_order: u32,

    /// Il contenuto testuale grezzo in formato UTF-8 lineare.
    pub raw_text: String,
}

impl BlockBase {
    /// Inizializza un nuovo blocco foglia vuoto assegnandogli un tipo specifico.
    pub fn new(block_type: BlockType) -> Self {
        Self {
            meta: Meta::new(),
            visual: VisualLayer::new(),
            geometry: BlockGeometry::zero(),
            parent_id: Uuid::nil(),
            block_type,
            sort_order: 0,
            raw_text: String::new(),
        }
    }

    /// Costruttore esplicito utile per i ripristini da file .scpt o database di sessione.
    pub fn with_specs(
        meta: Meta,
        visual: VisualLayer,
        geometry: BlockGeometry,
        parent_id: Uuid,
        block_type: BlockType,
        sort_order: u32,
        raw_text: String,
    ) -> Self {
        Self {
            meta,
            visual,
            geometry,
            parent_id,
            block_type,
            sort_order,
            raw_text,
        }
    }
}

// --- IMPLEMENTAZIONE DEI TRAIT TRAMITE DELEGARE ---

// 1. Risoluzione di BaseElement (Identità e Tempo)
impl BaseElement for BlockBase {
    fn id(&self) -> Uuid { self.meta.id }
    fn created_at(&self) -> u64 { self.meta.created_at }
    fn updated_at(&self) -> u64 { self.meta.updated_at }
    fn touch(&mut self) { self.meta.touch(); }
}

// 2. Risoluzione di LayeredElement (Grafica e Canali)
impl LayeredElement for BlockBase {
    fn visual_layer(&self) -> &VisualLayer { &self.visual }
    fn visual_layer_mut(&mut self) -> &mut VisualLayer {
        self.meta.touch(); // Spostare o modificare la grafica tocca il timestamp
        &mut self.visual
    }
}

// 3. Risoluzione di DocumentNode (Navigazione ad albero)
impl DocumentNode for BlockBase {
    fn uuid(&self) -> &[u8; 16] {
        self.meta.id.as_bytes()
    }
    fn parent_uuid(&self) -> &[u8; 16] {
        self.parent_id.as_bytes()
    }
    fn set_parent_uuid(&mut self, parent: [u8; 16]) {
        self.parent_id = Uuid::from_bytes(parent);
        self.meta.touch();
    }
    fn sort_order(&self) -> u32 { self.sort_order }
    fn set_sort_order(&mut self, order: u32) {
        self.sort_order = order;
        self.meta.touch();
    }
}

// 4. Risoluzione di BlockBehavior (Gestione del testo e della semantica)
impl BlockBehavior for BlockBase {
    fn is_linear(&self) -> bool {
        // I blocchi dedicati alla cover non seguono il flusso tipografico liquido continuo
        !matches!(
            self.block_type,
            BlockType::AuthorName | BlockType::ScriptVersion | BlockType::ProductionNote
        )
    }

    fn raw_text(&self) -> &str {
        &self.raw_text
    }

    fn set_raw_text(&mut self, text: String) {
        self.raw_text = text;
        self.meta.touch(); // Modificare il testo aggiorna istantaneamente il ciclo vitale temporale
    }
}

// Unit Test

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;
    use crate::geometry::BoundingBox;

    #[test]
    fn test_block_initialization() {
        let block = BlockBase::new(BlockType::Dialog);

        assert_eq!(block.block_type, BlockType::Dialog);
        assert!(block.raw_text().is_empty());
        assert_eq!(block.sort_order(), 0);
        assert_eq!(block.parent_uuid(), Uuid::nil().as_bytes());

        // Di default, un blocco testo normale deve essere lineare nel flusso del copione
        assert!(block.is_linear());
    }

    #[test]
    fn test_block_cover_exceptions_linearity() {
        // I blocchi specifici di copertina o grafici devono dichiararsi non lineari
        let block_author = BlockBase::new(BlockType::AuthorName);
        let block_version = BlockBase::new(BlockType::ScriptVersion);

        assert!(!block_author.is_linear());
        assert!(!block_version.is_linear());
    }

    #[test]
    fn test_text_modification_and_time_propagation() {
        let mut block = BlockBase::new(BlockType::StageDirection);
        let initial_update = block.updated_at();

        // Introduciamo un piccolissimo delay per far avanzare il timer di sistema
        thread::sleep(Duration::from_millis(2));

        block.set_raw_text("Apre la porta lentamente.".to_string());

        assert_eq!(block.raw_text(), "Apre la porta lentamente.");
        // Modificare il testo DEVE forzare l'aggiornamento del timestamp
        assert!(block.updated_at() > initial_update);
    }

    #[test]
    fn test_layered_element_interdependence() {
        let mut block = BlockBase::new(BlockType::CharacterName);
        let initial_update = block.updated_at();

        thread::sleep(Duration::from_millis(2));

        // Otteniamo l'accesso mutabile al layer visivo e modifichiamo la BoundingBox
        {
            let visual_layer = block.visual_layer_mut();
            visual_layer.bounds = BoundingBox::new(10.0, 20.0, 100.0, 12.0);
            visual_layer.channel_mask = [true, false, false, true]; // ScriptText e Revision accesi
        }

        // 1. Controlliamo se le modifiche geometriche persistono
        assert_eq!(block.visual_layer().bounds.x, 10.0);
        assert!(!block.visual_layer().is_active_in_channel(1)); // DirectorNotes spento
        assert!(block.visual_layer().is_active_in_channel(3));  // Revision attivo

        // 2. Controlliamo che l'interessenza abbia propagato il touch temporale anche sulla modifica grafica
        assert!(block.updated_at() > initial_update);
    }

    #[test]
    fn test_parent_and_ordering_hierarchy() {
        let mut block = BlockBase::new(BlockType::Transition);
        let parent_uuid = Uuid::new_v4();

        block.set_parent_uuid(*parent_uuid.as_bytes());
        block.set_sort_order(5);

        assert_eq!(block.parent_uuid(), parent_uuid.as_bytes());
        assert_eq!(block.sort_order(), 5);
    }

    #[test]
    fn test_channel_enabled_with_layer_type() {
        use crate::common::LayerType;
        let mut visual = VisualLayer::new();

        // Spegniamo le note di regia per simulare l'impaginazione pulita del testo
        visual.channel_mask[LayerType::DirectorNotes.to_index()] = false;

        assert!(visual.is_channel_enabled(LayerType::ScriptText));
        assert!(!visual.is_channel_enabled(LayerType::DirectorNotes)); // Correttamente spento!
    }
}