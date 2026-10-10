// src/core/layer.rs

//! Layer: canale di contenuti/annotazioni che si può accendere e spegnere,
//! sul modello di AutoCAD. Ogni `Block` e ogni `Annotation` dichiarano il
//! proprio `layer_id`; nascondere un layer nella UI è un filtro in lettura
//! sull'elenco dei blocchi/annotazioni visibili, non una modifica al
//! contenuto.
//!
//! `Layer` non implementa `TreeElement`: non vive nell'albero del copione,
//! è una voce in un registro piatto (l'elenco dei layer del progetto).
use crate::core::{ Entity, Meta };
use crate::types::{Color, ElementId, SortKey, Timestamp};

#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    pub meta: Meta,
    pub name: String,
    pub color: Color,

    /// Mostrato a schermo nell'editor.
    pub visible: bool,

    /// Impedisce la modifica del contenuto assegnato a questo layer,
    /// indipendentemente da `visible`.
    pub locked: bool,

    /// Incluso nell'output di stampa/esportazione. Deliberatamente separato
    /// da `visible`: si può voler vedere le note di regia in editing senza
    /// stamparle, o avere un layer pensato solo per la stampa.
    pub printable: bool,

    /// Posizione nel pannello layer. Non è imparentata con `sort_order` di
    /// Node/Block: qui ordina solo la lista piatta dei layer di progetto,
    /// non una gerarchia.
    pub order: SortKey,
}

impl Layer {
    pub fn new(name: impl Into<String>, color: Color, order: SortKey, at: Timestamp) -> Self {
        Self {
            meta: Meta::new(at),
            name: name.into(),
            color,
            visible: true,
            locked: false,
            printable: true,
            order,
        }
    }

    /// Come `new`, ma con id esplicito: usato dal command bus per il redo,
    /// che deve ricreare il layer con lo stesso id della prima
    /// applicazione.
    pub fn with_id(
        id: ElementId,
        name: impl Into<String>,
        color: Color,
        order: SortKey, 
        at: Timestamp
    ) -> Self {
        Self {
            meta: Meta::with_id(id, at),
            name: name.into(),
            color,
            visible: true,
            locked: false,
            printable: true,
            order,
        }
    }
}

impl Entity for Layer {
    fn meta(&self) -> &Meta { &self.meta }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_layer_defaults_to_visible_unlocked_printable() {
        let layer = Layer::new(
            "Note di regia",
            Color::rgb(200, 50, 50),
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        assert!(layer.visible);
        assert!(!layer.locked);
        assert!(layer.printable);
        assert_eq!(layer.name, "Note di regia");
    }

    #[test]
    fn visible_and_printable_are_independent() {
        let mut layer = Layer::new(
            "Solo editing",
            Color::rgb(0, 0, 0),
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        layer.printable = false;
        assert!(layer.visible);
        assert!(!layer.printable);
    }
}