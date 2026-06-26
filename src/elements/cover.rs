//! src/elements/cover.rs

use uuid::Uuid;
use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::common::CoverImage;
use crate::common::constants::{MAX_CHARACTERS_PER_SCRIPT, MAX_COVER_BLOCKS};

// Funzione helper per permettere a Serde di inizializzare l'array fisso nello Stack
fn default_cover_blocks() -> [Uuid; MAX_COVER_BLOCKS] { [Uuid::nil(); MAX_COVER_BLOCKS] }

/// Contenitore speciale per la prima pagina dell'opera.
/// Aggrega i blocchi di testo geometrici e l'eventuale risorsa grafica.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Cover<'a> {
    // Array statico di UUID che puntano a strutture Block esterne (posizionate tramite visual)
    #[serde(skip, default = "default_cover_blocks")]
    pub text_blocks: [Uuid; MAX_COVER_BLOCKS],
    pub text_count: usize,

    // Elemento grafico opzionale
    pub image_asset: Option<CoverImage<'a>>,
}

impl<'a> Cover<'a> {
    pub fn new() -> Self {
        Self {
            text_blocks: [Uuid::nil(); 8],
            text_count: 0,
            image_asset: None,
        }
    }
    /// Aggancia un blocco di testo (es: Heading1 per il Titolo) alla copertina tramite UUID
    pub fn add_text_block(&mut self, block_id: Uuid) -> Result<(), &'static str> {
        if self.text_count >= 8 {
            return Err("Capacità massima di elementi testuali per la copertina raggiunta (8)");
        }
        self.text_blocks[self.text_count] = block_id;
        self.text_count += 1;
        Ok(())
    }

    /// Restituisce gli ID dei blocchi di testo associati alla copertina
    pub fn text_block_ids(&self) -> &[Uuid] {
        &self.text_blocks[..self.text_count]
    }

    /// Imposta o rimuove l'immagine di copertina
    pub fn set_image(&mut self, image: Option<CoverImage<'a>>) {
        self.image_asset = image;
    }
}

impl<'a> Default for Cover<'a> {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// UNIT TESTS DI VALIDAZIONE
// =========================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::VisualLayer;
    use crate::common::BoundingBox;

    #[test]
    fn test_cover_composition_and_copy() {
        let mut cover = Cover::new();

        // Associazione di blocchi testuali simulati (Titolo e Autore)
        let title_block_id = Uuid::new_v4();
        let author_block_id = Uuid::new_v4();
        assert!(cover.add_text_block(title_block_id).is_ok());
        assert!(cover.add_text_block(author_block_id).is_ok());
        assert_eq!(cover.text_block_ids().len(), 2);

        // Configurazione dell'immagine
        let mut visual_img = VisualLayer::new();
        visual_img.bounds = BoundingBox { x: 0.4, y: 0.4, width: 0.2, height: 0.2 };
        let img = CoverImage::new("assets/logo.png", visual_img);
        cover.set_image(Some(img));

        assert!(cover.image_asset.is_some());
        assert_eq!(cover.image_asset.unwrap().asset_path, "assets/logo.png");

        // Verifica invarianza tratto Copy
        let copia_cover = cover;
        assert_eq!(copia_cover.text_count, cover.text_count);
        assert_eq!(copia_cover.image_asset.unwrap().asset_path, cover.image_asset.unwrap().asset_path);
    }
}