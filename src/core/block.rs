// src/core/block.rs

//! Blocco di contenuto: foglia dell'albero, testuale o non testuale.
//!
//! Un `Block` è sempre associato a un layer (`layer_id`): è così che il
//! toggle "accendi/spegni layer" della UI diventa un semplice filtro in
//! lettura, senza toccare il contenuto. La natura tecnica del contenuto
//! (testo vs asset) è nel `payload`; il ruolo drammaturgico di un testo
//! (battuta, didascalia, parenthetical...) è in `TextBlock::kind`.

use crate::core::{ BaseElement, Meta, Entity, TreeElement };
use crate::types:: {
    BlockKind,
    Dimension,
    ElementId,
    Placement,
    SortKey,
    Timestamp
};

/// Id di un layer. Alias di `ElementId`: i layer condividono lo stesso
/// spazio di identificatori di Node/Block/Annotation (vedi la nota in
/// `types::ElementId`), utile per riferimenti incrociati senza dover
/// distinguere "a quale tipo di entità punta questo id".
pub type LayerId = ElementId;

/// Blocco non testuale: immagine, logo, altro asset grafico.
///
/// Niente `BoundingBox` qui: la posizione e le dimensioni effettive sono
/// sempre un risultato del motore di layout (in `LayoutResult`),
/// mai un dato intrinseco del blocco. Qui teniamo solo
/// l'intenzione: dimensioni desiderate e come collocare l'asset.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetBlock {
    /// Riferimento alla risorsa. Per ora un path/URI grezzo; un asset
    /// manager con hash e deduplica arriverà più avanti.
    pub src: String,

    /// Dimensioni desiderate; se `None` su un asse, il layout usa le
    /// dimensioni naturali della risorsa scalate in proporzione.
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub placement: Placement,
}

/// Blocco di testo: battuta, didascalia, parenthetical, nome personaggio...
/// `kind` guida stile e layer di default; il testo per ora è piatto — gli
/// span inline (corsivo, riferimenti a personaggi citati nel testo) sono
/// un passo successivo, quando affronteremo il rich text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    pub kind: BlockKind,
    pub text: String,
    /// Riferimento al personaggio, significativo solo quando
    /// `kind == BlockKind::CharacterName`. Il riferimento persistente è
    /// sempre questo `ElementId`, mai l'alias digitato in fase di stesura:
    /// se l'alias cambia, questo campo resta valido perché punta al
    /// personaggio, non alla scorciatoia usata per nominarlo. `None` per
    /// qualunque altro `kind`, o per un nome ancora privo di
    /// corrispondenza in anagrafica (in quel caso l'editor segnalerà il
    /// blocco come "personaggio non risolto", non un errore).
    pub character_id: Option<ElementId>,
}

impl TextBlock {
    pub fn new(kind: BlockKind, text: impl Into<String>) -> Self {
        Self{ kind, text: text.into(), character_id: None }
    }

    #[must_use]
    pub fn with_character(mut self, character_id: ElementId) -> Self {
        self.character_id = Some(character_id);
        self
    }
}

/// Natura tecnica del contenuto di un blocco.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockPayload {
    Text(TextBlock),
    Asset(AssetBlock),
}

impl From<AssetBlock> for BlockPayload {
    fn from(block: AssetBlock) -> Self {
        BlockPayload::Asset(block)
    }
}

impl From<TextBlock> for BlockPayload {
    fn from(block: TextBlock) -> Self {
        BlockPayload::Text(block)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    base: BaseElement,
    pub layer: LayerId,
    pub payload: BlockPayload,
}

impl Block {
    /// Costruttore unico per qualunque tipo di payload: `impl Into<BlockPayload>`
    /// accetta sia un `TextBlock`/`AssetBlock` grazie ai `From` sopra, sia un
    /// `BlockPayload` già costruito. Aggiungere una nuova variante (es. una
    /// futura `TableBlock`) richiede solo un nuovo `From`, non un nuovo
    /// costruttore su `Block`.
    pub fn new(
        payload: impl Into<BlockPayload>,
        layer: LayerId,
        parent: Option<ElementId>,
        order: SortKey,
        at: Timestamp,
    ) -> Self {
        Self {
            base: BaseElement::new(parent, order, at),
            layer,
            payload: payload.into(),
        }
    }

    /// Come `new`, ma con id esplicito: usato dal command bus per il redo,
    /// che deve ricreare il blocco con lo stesso id della prima
    /// applicazione.
    pub fn with_id(
        id: ElementId,
        payload: impl Into<BlockPayload>,
        layer: LayerId,
        parent: Option<ElementId>,
        order: SortKey,
        at: Timestamp,
    ) -> Self {
        Self {
            base: BaseElement::with_id(id, parent, order, at),
            layer,
            payload: payload.into(),
        }
    }

    pub fn as_asset(&self) -> Option<&AssetBlock> {
        match &self.payload {
            BlockPayload::Asset(asset) => Some(asset),
            BlockPayload::Text(_) => None,
        }
    }

    pub fn as_text(&self) -> Option<&TextBlock> {
        match &self.payload {
            BlockPayload::Text(text) => Some(text),
            BlockPayload::Asset(_) => None,
        }
    }

    /// Cambia parent e posizione fra fratelli. Visibile solo all'interno
    /// del crate: va chiamato esclusivamente da
    /// `commands::Command::MoveBlock`, mai direttamente — stesso principio
    /// di `Node::reparent`.
    pub(crate) fn reparent(&mut self, parent: Option<ElementId>, order: SortKey) {
        self.base.parent = parent;
        self.base.sort_order = order;
    }
}

impl Entity for Block {
    fn meta(&self) -> &Meta {
        self.base.meta()
    }
}

impl TreeElement for Block {
    fn parent(&self) -> Option<ElementId> {
        self.base.parent
    }
    fn sort_order(&self) -> &SortKey {
        &self.base.sort_order
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_block_roundtrip() {
        let block = Block::new(
            TextBlock::new(BlockKind::Dialogue, "Ciao, mondo."),
            LayerId::new(),
            None,
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        let text = block.as_text().expect("deve essere un TextBlock");
        assert_eq!(text.text, "Ciao, mondo.");
        assert_eq!(text.kind, BlockKind::Dialogue);
        assert!(text.character_id.is_none());
        assert!(block.as_asset().is_none());
    }

    #[test]
    fn character_name_block_carries_character_id() {
        let character_id = ElementId::new();
        let block = Block::new(
            TextBlock::new(BlockKind::CharacterName, "GIU").with_character(character_id),
            LayerId::new(),
            None,
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        assert_eq!(block.as_text().unwrap().character_id, Some(character_id));
    }

    #[test]
    fn asset_block_roundtrip() {
        let asset = AssetBlock {
            src: "logo.png".to_string(),
            width: Some(Dimension::px(120.0)),
            height: None,
            placement: Placement::Flow,
        };
        let block = Block::new(
            asset,
            LayerId::new(),
            None,
            SortKey::initial(),
            Timestamp::from_unix_seconds(0),
        );
        assert!(block.as_text().is_none());
        let asset = block.as_asset().expect("deve essere un AssetBlock");
        assert_eq!(asset.src, "logo.png");
    }
}