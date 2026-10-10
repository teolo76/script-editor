// src/lake/document.rs

//! Il documento nella sua interezza: tutte le entità (Node, Block,
//! Annotation, Layer, Character), ciascuna nel proprio arena, più i
//! metadati di progetto (titolo, sinossi, copertina) e la revisione
//! globale usata da cache di layout e command bus.
//!
//! I riferimenti incrociati passano sempre per `ElementId`, mai per
//! puntatori diretti: spostare o rimuovere un elemento non richiede di
//! aggiornare riferimenti sparsi in altre strutture.
use crate::core::{
    DocMeta,
    Node,
    Block,
    Character,
    Annotation,
    Layer,
    Entity,
    TreeElement
};
use crate::errors::CoreResult;
use crate::lake::{ CharacterStore, EntityStore };
use crate::types::{ ElementId, CharacterAlias, Revision, SortKey, Timestamp, Version, ROOT_ID };

/// Riferimento a un figlio nell'albero, che sia un `Node` o un `Block`:
/// serve perché i figli di un dato elemento non sono sempre dello stesso
/// tipo (i figli di un Atto sono Scene, i figli di una Scena sono Blocchi).
#[derive(Debug)]
pub enum ElementRef<'a>{
    Node(&'a Node),
    Block(&'a Block),
}

impl<'a> ElementRef<'a>{
    pub fn id(&self) -> ElementId {
        match self {
            ElementRef::Node(n) => n.id(),
            ElementRef::Block(b) => b.id(),
        }
    }
    pub fn sort_order(&self) -> &SortKey {
        match self {
            ElementRef::Node(n) => n.sort_order(),
            ElementRef::Block(b) => b.sort_order(),
        }
    }
}

pub struct Document {
    nodes: EntityStore<Node>,
    blocks: EntityStore<Block>,
    annotations: EntityStore<Annotation>,
    layers: EntityStore<Layer>,
    characters: CharacterStore,
    revision: Revision,
    pub metadata: DocMeta,
}

impl Document {
    /// Documento vuoto, con solo il nodo Root.
    pub fn new(at: Timestamp) -> Self {
        let mut nodes = EntityStore::new();
        nodes.insert(Node::root(at));

        Self {
            nodes,
            blocks: EntityStore::new(),
            annotations: EntityStore::new(),
            layers: EntityStore::new(),
            characters: CharacterStore::new(),
            metadata: DocMeta::default(),
            revision: Revision::INITIAL,
        }
    }

    pub fn root_id(&self) -> ElementId {
        ROOT_ID
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    /// Avanza la revisione del documento. Da chiamare una volta per batch
    /// di comandi applicato (mai per singola mutazione interna): è quello
    /// che permette al command bus di taggare gli eventi di delta e alla
    /// cache di layout di sapere cosa invalidare.
    pub fn bump_revision(&mut self) -> Revision {
        self.revision = self.revision.next();
        self.revision
    }

    /// Avanza `meta.version.sub`. Deliberatamente **non** un `Command`: è
    /// un timbro di bookkeeping (tipicamente al salvataggio), non
    /// contenuto su cui l'utente si aspetta di poter fare undo — disfare
    /// una modifica al testo non deve "desalvare" il documento. L'app
    /// decide quando chiamarlo (al salvataggio, o su qualunque altro
    /// trigger); `core` non impone una policy. Non tocca `revision`: la
    /// cache di layout non deve invalidarsi solo perché si è salvato.
    pub fn bump_version_sub(&mut self) -> Version {
        self.metadata.version.minor += 1;
        self.metadata.version
    }

    // --- Nodes ---

    pub fn insert_node(&mut self, node: Node) -> ElementId {
        let id = node.id();
        self.nodes.insert(node);
        id
    }

    pub fn node(&self, id: ElementId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: ElementId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn remove_node(&mut self, id: ElementId) -> Option<Node> {
        self.nodes.remove(id)
    }

    // --- Blocks ---

    pub fn insert_block(&mut self, block: Block) -> ElementId {
        let id = block.id();
        self.blocks.insert(block);
        id
    }

    pub fn block(&self, id: ElementId) -> Option<&Block> {
        self.blocks.get(id)
    }

    pub fn block_mut(&mut self, id: ElementId) -> Option<&mut Block> {
        self.blocks.get_mut(id)
    }

    pub fn remove_block(&mut self, id: ElementId) -> Option<Block> {
        self.blocks.remove(id)
    }

    // --- Annotations ---

    pub fn insert_annotation(&mut self, annotation: Annotation) -> ElementId {
        let id = annotation.id();
        self.annotations.insert(annotation);
        id
    }

    pub fn annotation(&self, id: ElementId) -> Option<&Annotation> {
        self.annotations.get(id)
    }

    pub fn annotation_mut(&mut self, id: ElementId) -> Option<&mut Annotation> {
        self.annotations.get_mut(id)
    }

    pub fn remove_annotation(&mut self, id: ElementId) -> Option<Annotation> {
        self.annotations.remove(id)
    }

    // --- Layers ---

    pub fn insert_layer(&mut self, layer: Layer) -> ElementId {
        let id = layer.id();
        self.layers.insert(layer);
        id
    }

    pub fn layer(&self, id: ElementId) -> Option<&Layer> {
        self.layers.get(id)
    }

    pub fn layer_mut(&mut self, id: ElementId) -> Option<&mut Layer> {
        self.layers.get_mut(id)
    }

    pub fn remove_layer(&mut self, id: ElementId) -> Option<Layer> {
        self.layers.remove(id)
    }

    // --- Characters ---

    /// Fallisce se l'alias del personaggio è già usato da un altro
    /// personaggio (vedi `CharacterStore::insert`).
    pub fn insert_character(&mut self, character: Character) -> CoreResult<ElementId> {
        self.characters.insert(character)
    }

    pub fn character(&self, id: ElementId) -> Option<&Character> {
        self.characters.get(id)
    }

    pub fn character_mut(&mut self, id: ElementId) -> Option<&mut Character> {
        self.characters.get_mut(id)
    }

    pub fn remove_character(&mut self, id: ElementId) -> Option<Character> {
        self.characters.remove(id)
    }

    /// Rinomina l'alias di un personaggio già inserito.
    pub fn rename_character_alias(&mut self, id: ElementId, alias: CharacterAlias) -> CoreResult<()> {
        self.characters.rename_alias(id, alias)
    }

    /// Risolve un alias digitato in fase di stesura nell'id del
    /// personaggio corrispondente.
    pub fn resolve_character_alias(&mut self, alias: &CharacterAlias) -> Option<ElementId> {
        self.characters.resolve_alias(alias)
    }

    /// Personaggi senza attore assegnato ("fantasmi").
    pub fn ghost_characters(&self) -> impl Iterator<Item=&Character> {
        self.characters.ghosts()
    }

    pub fn characters(&self) -> impl Iterator<Item=&Character> {
        self.characters.iter()
    }

    // --- Query di attraversamento ---

    /// Figli diretti di `parent` (Node e/o Block insieme), ordinati per
    /// `sort_order`. Non assume a priori quale tipo aspettarsi: tratta
    /// Node e Block allo stesso modo perché entrambi implementano
    /// `TreeElement`.
    pub fn children_of(&self, parent: ElementId) -> Vec<ElementRef<'_>> {
        let mut children: Vec<ElementRef<'_>> = self
            .nodes
            .iter()
            .filter(|n| n.parent() == Some(parent))
            .map(ElementRef::Node)
            .chain(
                self.blocks
                    .iter()
                    .filter(|b| b.parent() == Some(parent))
                    .map(ElementRef::Block)
            )
            .collect();
        children.sort_by(|a, b| a.sort_order().cmp(b.sort_order()));
        children
    }

    /// Annotazioni ancorate a `element`, che sia l'intero elemento o un
    /// intervallo al suo interno.
    pub fn annotations_for(&self, element: ElementId) -> Vec<&Annotation> {
        self.annotations
            .iter()
            .filter(|a| a.anchored_element() == element)
            .collect()
    }

    /// Blocchi assegnati a un dato layer.
    pub fn blocks_on_layer(&self, layer: ElementId) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| b.layer == layer)
            .collect()
    }

    /// Blocchi `CharacterName` che referenziano un dato personaggio.
    /// Utile per "in quali punti del copione appare questo personaggio".
    pub fn blocks_for_character(&self, character: ElementId) -> Vec<&Block> {
        self.blocks
            .iter()
            .filter(|b| {
                b.as_text()
                    .and_then(|t| t.character_id)
                    .map(|id|  id == character)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Tutti gli id (Node e/o Block) nel sottoalbero radicato in `id`,
    /// `id` stesso escluso. Usato per la rimozione a cascata di un nodo
    /// (`Command::RemoveNode`): senza questo, cancellare un Atto
    /// lascerebbe le sue Scene orfane nello store, invisibili da qualunque
    /// traversal da Root ma mai effettivamente liberate.
    ///
    /// `children_of` funziona indifferentemente su Node e Block come
    /// parent candidato: se `current` è un Block (che non ha mai figli nel
    /// nostro modello), restituisce semplicemente una lista vuota, quindi
    /// non serve distinguere i due casi qui.
    pub fn descendant_ids(&self, id: ElementId) -> Vec<ElementId> {
        let mut result = Vec::new();
        let mut stack: Vec<ElementId> = self
            .children_of(id)
            .iter()
            .map(ElementRef::id)
            .collect();

        while let Some(current) = stack.pop() {
            result.push(current);
            stack.extend(
                self.children_of(current)
                .iter()
                .map(ElementRef::id));
        }
        result
    }

    /// Il parent di `id`, che sia un Node o un Block. Usato solo da
    /// `is_descendant_of` per risalire la catena degli antenati.
    fn parent_of(&self, parent: ElementId) -> Option<ElementId> {
        if let Some(node) = self.nodes.get(parent) {
            node.parent()
        } else if let Some(block) = self.blocks.get(parent) {
            block.parent()
        } else { None }
    }

    /// `true` se `candidate` è un discendente (a qualunque profondità) di
    /// `ancestor`. Usato da `Command::MoveNode`/`MoveBlock` per rifiutare
    /// uno spostamento che creerebbe un ciclo (spostare un nodo dentro un
    /// suo stesso discendente). Non considera `candidate == ancestor`
    /// discendente di se stesso: quel caso (spostare un elemento dentro se
    /// stesso) va controllato separatamente dal chiamante.
    pub fn is_descendant_of(&self, candidate: ElementId, ancestor: ElementId) -> bool {
        let mut current = self.parent_of(candidate);
        while let Some(parent_id) = current {
            if parent_id == ancestor {
                return true;
            }
            current = self.parent_of(parent_id);
        }
        false
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AnchorTarget, LayerId, TextBlock};
    use crate::types::{BlockKind, CharacterAlias, Color, NodeKind};

    fn t(secs: u64) -> Timestamp {
        Timestamp::from_unix_seconds(secs)
    }

    fn alias(s: &str) -> CharacterAlias {
        CharacterAlias::new(s).unwrap()
    }

    #[test]
    fn new_document_has_only_root() {
        let doc = Document::new(t(0));
        let root = doc.node(doc.root_id()).expect("il Root deve esistere");
        assert_eq!(root.kind, NodeKind::Root);
        assert_eq!(root.parent(), None);
    }

    #[test]
    fn children_of_mixes_nodes_and_blocks_sorted() {
        let mut doc = Document::new(t(0));
        let root_id = doc.root_id();
        let layer_id: LayerId = ElementId::new();

        let key_b = SortKey::initial();
        let key_a = SortKey::between(None, Some(&key_b));
        let key_c = key_b.after();

        let act = Node::new(NodeKind::Act, Some(root_id), key_b.clone(), t(0));
        let act_id = doc.insert_node(act);

        let block = Block::new(
            TextBlock::new(BlockKind::StageDirection, "Buio in sala."),
            layer_id,
            Some(root_id),
            key_a.clone(),
            t(0),
        );
        let block_id = doc.insert_block(block);

        let act2 = Node::new(NodeKind::Act, Some(root_id), key_c.clone(), t(0));
        let act2_id = doc.insert_node(act2);

        let children = doc.children_of(root_id);
        let ids: Vec<ElementId> = children.iter().map(ElementRef::id).collect();
        assert_eq!(ids, vec![block_id, act_id, act2_id]);
    }

    #[test]
    fn annotations_for_filters_by_anchored_element() {
        let mut doc = Document::new(t(0));
        let layer_id: LayerId = ElementId::new();
        let block = Block::new(
            TextBlock::new(BlockKind::Dialogue, "Ciao."),
            layer_id,
            Some(doc.root_id()),
            SortKey::initial(),
            t(0),
        );
        let block_id = doc.insert_block(block);

        let other_id = ElementId::new();

        doc.insert_annotation(Annotation::new(
            layer_id,
            AnchorTarget::Element(block_id),
            "Nota di regia",
            t(0),
        ));
        doc.insert_annotation(Annotation::new(
            layer_id,
            AnchorTarget::Element(other_id),
            "Non pertinente",
            t(0),
        ));

        let found = doc.annotations_for(block_id);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].body, "Nota di regia");
    }

    #[test]
    fn blocks_on_layer_filters_correctly() {
        let mut doc = Document::new(t(0));
        let layer_a: LayerId = ElementId::new();
        let layer_b: LayerId = ElementId::new();

        doc.insert_block(Block::new(
            TextBlock::new(BlockKind::Dialogue, "Su A"),
            layer_a,
            Some(doc.root_id()),
            SortKey::initial(),
            t(0),
        ));
        doc.insert_block(Block::new(
            TextBlock::new(BlockKind::Dialogue, "Su B"),
            layer_b,
            Some(doc.root_id()),
            SortKey::initial(),
            t(0),
        ));

        assert_eq!(doc.blocks_on_layer(layer_a).len(), 1);
        assert_eq!(doc.blocks_on_layer(layer_b).len(), 1);
    }

    #[test]
    fn bump_revision_is_monotonic() {
        let mut doc = Document::new(t(0));
        let r0 = doc.revision();
        let r1 = doc.bump_revision();
        let r2 = doc.bump_revision();
        assert!(r0 < r1);
        assert!(r1 < r2);
    }

    #[test]
    fn layer_roundtrip() {
        let mut doc = Document::new(t(0));
        let layer = Layer::new("Note di regia", Color::rgb(200, 50, 50), SortKey::initial(), t(0));
        let id = doc.insert_layer(layer);
        assert_eq!(doc.layer(id).unwrap().name, "Note di regia");
        assert!(doc.remove_layer(id).is_some());
        assert!(doc.layer(id).is_none());
    }

    #[test]
    fn character_insert_and_resolve_through_document() {
        let mut doc = Document::new(t(0));
        let id = doc
            .insert_character(Character::new("Giulietta", alias("GIU"), t(0)))
            .unwrap();
        assert_eq!(doc.resolve_character_alias(&alias("GIU")), Some(id));
        assert_eq!(doc.character(id).unwrap().name, "Giulietta");
    }

    #[test]
    fn character_duplicate_alias_is_rejected_through_document() {
        let mut doc = Document::new(t(0));
        doc.insert_character(Character::new("Giulietta", alias("GIU"), t(0))).unwrap();
        let err = doc
            .insert_character(Character::new("Giuseppe", alias("GIU"), t(0)))
            .unwrap_err();
        assert!(matches!(err, crate::errors::CoreError::DuplicateAlias(_)));
    }

    #[test]
    fn blocks_for_character_finds_character_name_blocks() {
        let mut doc = Document::new(t(0));
        let layer_id: LayerId = ElementId::new();
        let character_id = doc
            .insert_character(Character::new("Romeo", alias("ROM"), t(0)))
            .unwrap();

        let name_block = Block::new(
            TextBlock::new(BlockKind::CharacterName, "ROM").with_character(character_id),
            layer_id,
            Some(doc.root_id()),
            SortKey::initial(),
            t(0),
        );
        let name_block_id = doc.insert_block(name_block);

        // Una battuta qualunque, senza character_id: non deve comparire.
        doc.insert_block(Block::new(
            TextBlock::new(BlockKind::Dialogue, "Ma soft, che luce filtra da quella finestra?"),
            layer_id,
            Some(doc.root_id()),
            SortKey::initial(),
            t(0),
        ));

        let found = doc.blocks_for_character(character_id);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id(), name_block_id);
    }

    #[test]
    fn descendant_ids_collects_whole_subtree() {
        let mut doc = Document::new(t(0));
        let root_id = doc.root_id();
        let layer_id: LayerId = ElementId::new();

        let act_id = doc.insert_node(Node::new(NodeKind::Act, Some(root_id), SortKey::initial(), t(0)));
        let scene_id = doc.insert_node(Node::new(NodeKind::Scene, Some(act_id), SortKey::initial(), t(0)));
        let block_id = doc.insert_block(Block::new(
            TextBlock::new(BlockKind::Dialogue, "Ciao."),
            layer_id,
            Some(scene_id),
            SortKey::initial(),
            t(0),
        ));

        let mut descendants = doc.descendant_ids(act_id);
        descendants.sort();
        let mut expected = vec![scene_id, block_id];
        expected.sort();
        assert_eq!(descendants, expected);

        // Il root non compare mai nel sottoalbero di un suo discendente.
        assert!(!doc.descendant_ids(scene_id).contains(&root_id));
    }

    #[test]
    fn is_descendant_of_detects_nested_relationship() {
        let mut doc = Document::new(t(0));
        let root_id = doc.root_id();
        let act_id = doc.insert_node(Node::new(NodeKind::Act, Some(root_id), SortKey::initial(), t(0)));
        let scene_id = doc.insert_node(Node::new(NodeKind::Scene, Some(act_id), SortKey::initial(), t(0)));

        assert!(doc.is_descendant_of(scene_id, act_id));
        assert!(doc.is_descendant_of(scene_id, root_id));
        assert!(!doc.is_descendant_of(act_id, scene_id));
        // Un elemento non è mai considerato discendente di se stesso da
        // questa funzione: quel controllo spetta al chiamante.
        assert!(!doc.is_descendant_of(act_id, act_id));
    }

    #[test]
    fn ghost_characters_are_visible_from_document() {
        let mut doc = Document::new(t(0));
        doc.insert_character(Character::new("Voce fuori campo", alias("VOC"), t(0))).unwrap();
        let mut cast = Character::new("Romeo", alias("ROM"), t(0));
        cast.actor = Some("Mario Rossi".to_string());
        doc.insert_character(cast).unwrap();

        let ghosts: Vec<&str> = doc.ghost_characters().map(|c| c.name.as_str()).collect();
        assert_eq!(ghosts, vec!["Voce fuori campo"]);
    }
}