//! Comandi che mutano il `Document`. Un `Command` non tocca mai i campi
//! delle entità direttamente: passa sempre dai metodi che `Document` e
//! `CharacterStore` già espongono, così le regole di validazione (unicità
//! alias, prevenzione cicli, ecc.) restano in un solo posto.
//!
//! Ogni comando produce, applicandosi, il proprio inverso (`Undo`): è
//! quello che il bus tiene in cronologia per l'undo. Il redo non ricalcola
//! nulla di nuovo: rilancia lo stesso `Command`, con lo stesso `id` già
//! deciso alla costruzione — per questo ogni variante "crea X" porta con
//! sé un `ElementId` invece di lasciarlo generare da `apply`. Così l'id è
//! noto subito al chiamante (utile per comporre un batch dove un comando
//! fa riferimento all'id che un comando precedente nello stesso batch sta
//! per creare, prima che nessuno dei due sia stato eseguito), ed è lo
//! stesso sia alla prima applicazione sia a ogni redo successivo.
//!
//! Un comando copre un solo aspetto per volta, salvo dove abbiamo scelto
//! deliberatamente di raggruppare: i comandi che toccano `DocMeta`
//! raggruppano per sezione (`SetDocAuthorship`, `SetDocProduction`, ...)
//! invece di un comando per campo, perché è così che quei campi verranno
//! probabilmente editati (un form a blocchi nel CoverEditor) e perché un
//! comando per campo avrebbe fatto esplodere l'enum a 15+ varianti.

use crate::core::{
    Annotation, AnchorTarget, Authorship, Block, BlockPayload, Character, CoverImage, Entity,
    Layer, LayerId, Node, Production, TreeElement,
};
use crate::errors::{CoreError, CoreResult};
use crate::lake::Document;
use crate::types::{
    CharacterAlias, Color, Copyright, ElementId, NodeKind, ProjectStatus, SortKey, Timestamp,
    Version, ROOT_ID,
};

#[derive(Debug, PartialEq)]
pub enum Command {
    /// `parent` non è opzionale: nessuna chiamata legittima a questo
    /// comando crea il Root (esiste già, creato una volta sola da
    /// `Document::new`, mai tramite comando), quindi ogni nodo inserito
    /// dal bus ha sempre un parent concreto. Stessa logica di `MoveNode`.
    InsertNode {
        id: ElementId,
        kind: NodeKind,
        parent: ElementId,
        sort_order: SortKey,
    },
    RemoveNode {
        id: ElementId,
    },
    SetNodeTitle {
        id: ElementId,
        title: Option<String>,
    },
    /// Cambia parent e/o posizione di un Node. `new_parent` non è
    /// opzionale: solo il Root ha parent `None`, il Root non si può
    /// spostare (rifiutato da `apply`), quindi ogni bersaglio valido di
    /// una move è un id concreto.
    MoveNode {
        id: ElementId,
        new_parent: ElementId,
        new_sort_order: SortKey,
    },
    /// `parent` non è opzionale, stessa ragione di `InsertNode`: un Block
    /// non è mai il Root, quindi ha sempre un parent concreto.
    InsertBlock {
        id: ElementId,
        payload: BlockPayload,
        layer_id: LayerId,
        parent: ElementId,
        sort_order: SortKey,
    },
    RemoveBlock {
        id: ElementId,
    },
    MoveBlock {
        id: ElementId,
        new_parent: ElementId,
        new_sort_order: SortKey,
    },
    InsertLayer {
        id: ElementId,
        name: String,
        color: Color,
        order: SortKey,
    },
    RemoveLayer {
        id: ElementId,
    },
    SetLayerVisible {
        id: ElementId,
        visible: bool,
    },
    SetLayerLocked {
        id: ElementId,
        locked: bool,
    },
    SetLayerPrintable {
        id: ElementId,
        printable: bool,
    },
    SetLayerOrder {
        id: ElementId,
        new_order: SortKey,
    },
    InsertCharacter {
        id: ElementId,
        name: String,
        alias: CharacterAlias,
    },
    RemoveCharacter {
        id: ElementId,
    },
    /// Cambia `Character.name` (il nome per esteso), non l'alias: i due
    /// campi sono deliberatamente scollegati — correggere un errore nel
    /// nome non deve forzare una decisione sull'alias, che resta una
    /// scorciatoia stabile scelta a mano. Un form che vuole cambiare
    /// entrambi in un colpo manda questo comando e `RenameCharacterAlias`
    /// nello stesso batch: l'atomicità arriva gratis dal bus, non serve
    /// un comando combinato.
    RenameCharacter {
        id: ElementId,
        new_name: String,
    },
    RenameCharacterAlias {
        id: ElementId,
        new_alias: CharacterAlias,
    },
    SetCharacterActor {
        id: ElementId,
        actor: Option<String>,
    },
    SetCharacterCostumeNotes {
        id: ElementId,
        notes: Option<String>,
    },
    SetCharacterEntranceNote {
        id: ElementId,
        note: Option<String>,
    },
    SetCharacterExitNote {
        id: ElementId,
        note: Option<String>,
    },
    /// Cambia l'ancora di un'annotazione (a cosa è agganciata), non la
    /// sua posizione in un elenco: nome diverso da "Move" apposta, perché
    /// la semantica è diversa da un riordino fra fratelli.
    ReanchorAnnotation {
        id: ElementId,
        new_anchor: AnchorTarget,
    },

    // --- DocMeta: raggruppati per sezione, vedi il commento in cima al file. ---
    SetDocTitle(Option<String>),
    SetDocSynopsis(Option<String>),
    SetDocSetting(Option<String>),
    SetDocCover(Option<CoverImage>),
    SetDocAuthorship(Authorship),
    SetDocProduction(Production),
    SetDocRights(Option<Copyright>),
    SetDocCredits(Vec<(String, String)>),
    SetDocStatus {
        status: ProjectStatus,
        distribution_note: Option<String>,
    },
    /// Decisione deliberata dell'autore: passa dal bus con undo, a
    /// differenza di `Document::bump_version_sub` (bookkeeping automatico,
    /// fuori dal bus). Per convenzione azzera `sub` — vedi `types::Version`.
    SetDocVersionMajor(u32),
    /// Correzione manuale di `sub`, distinta dall'incremento automatico.
    SetDocVersionSub(u32),
}

/// L'inverso di un `Command` già applicato: quello che serve per tornare
/// indietro. Non è pensato per essere costruito a mano — lo produce
/// `Command::apply`, e lo consuma solo `CommandBus`.
#[derive(Debug, PartialEq)]
pub(crate) enum Undo {
    /// Inverso di `Command::InsertNode`: rimuove semplicemente il nodo
    /// appena creato. Nessuna cascata necessaria — al momento
    /// dell'inserimento il nodo non può ancora avere figli né annotazioni
    /// ancorate, e se un batch inserisce anche quelli in comandi
    /// successivi, l'undo del batch li disfa in ordine inverso comando
    /// per comando, quindi arrivano già rimossi prima di questo.
    RemoveNode(ElementId),
    /// Inverso di `Command::RemoveNode`: rimozione a cascata, quindi
    /// ripristino a cascata. Oltre al nodo stesso, tutto il suo
    /// sottoalbero (Node e Block discendenti) e tutte le `Annotation`
    /// ancorate a uno qualunque di questi elementi — senza la cascata,
    /// rimuovere un Atto lascerebbe le sue Scene orfane nello store, e le
    /// annotazioni sul contenuto rimosso resterebbero agganciate a id
    /// ormai inesistenti.
    RemoveNodeCascade {
        node: Node,
        child_nodes: Vec<Node>,
        child_blocks: Vec<Block>,
        annotations: Vec<Annotation>,
    },
    SetNodeTitle {
        id: ElementId,
        title: Option<String>,
    },
    MoveNode {
        id: ElementId,
        parent: Option<ElementId>,
        sort_order: SortKey,
    },
    /// Inverso di `Command::InsertBlock`, stesso ragionamento di
    /// `RemoveNode` qui sopra: nessuna cascata al momento dell'inserimento.
    RemoveBlock(ElementId),
    /// Inverso di `Command::RemoveBlock`. Un Block non ha mai figli nel
    /// nostro modello, quindi qui basta l'elenco delle annotazioni:
    /// nessun sottoalbero da ripristinare.
    RemoveBlockCascade {
        block: Block,
        annotations: Vec<Annotation>,
    },
    MoveBlock {
        id: ElementId,
        parent: Option<ElementId>,
        sort_order: SortKey,
    },
    RemoveLayer(ElementId),
    ReinsertLayer(Layer),
    SetLayerVisible { id: ElementId, visible: bool },
    SetLayerLocked { id: ElementId, locked: bool },
    SetLayerPrintable { id: ElementId, printable: bool },
    SetLayerOrder { id: ElementId, order: SortKey },
    RemoveCharacter(ElementId),
    ReinsertCharacter(Character),
    RenameCharacter { id: ElementId, name: String },
    RenameCharacterAlias { id: ElementId, alias: CharacterAlias },
    SetCharacterActor { id: ElementId, actor: Option<String> },
    SetCharacterCostumeNotes { id: ElementId, notes: Option<String> },
    SetCharacterEntranceNote { id: ElementId, note: Option<String> },
    SetCharacterExitNote { id: ElementId, note: Option<String> },
    ReanchorAnnotation { id: ElementId, anchor: AnchorTarget },

    SetDocTitle(Option<String>),
    SetDocSynopsis(Option<String>),
    SetDocSetting(Option<String>),
    SetDocCover(Option<CoverImage>),
    SetDocAuthorship(Authorship),
    SetDocProduction(Production),
    SetDocRights(Option<Copyright>),
    SetDocCredits(Vec<(String, String)>),
    SetDocStatus {
        status: ProjectStatus,
        distribution_note: Option<String>,
    },
    /// Ripristina l'intera coppia (major, sub) precedente. Riusato sia da
    /// `SetDocVersionMajor` sia da `SetDocVersionSub`: entrambi mutano lo
    /// stesso `Version`, quindi l'inverso è sempre "rimetti il Version di
    /// prima", non un campo alla volta.
    SetDocVersion(Version),
}

/// Categoria di cambiamento subito da un elemento, usata per costruire il
/// `Delta` restituito dal bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeKind {
    Created,
    Updated,
    Removed,
}

/// Riepilogo di cosa è cambiato in un batch: gli id creati, modificati e
/// rimossi, più un flag per i cambiamenti a `DocMeta`. `DocMeta` non ha un
/// proprio `ElementId` (è singleton per documento), quindi non entra nelle
/// altre liste — un ascoltatore che se ne cura guarda `meta_changed`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Delta {
    pub created: Vec<ElementId>,
    pub updated: Vec<ElementId>,
    pub removed: Vec<ElementId>,
    pub meta_changed: bool,
}

impl Delta {
    fn push(&mut self, id: ElementId, kind: ChangeKind) {
        match kind {
            ChangeKind::Created => self.created.push(id),
            ChangeKind::Updated => self.updated.push(id),
            ChangeKind::Removed => self.removed.push(id),
        }
    }

    /// Delta di un'applicazione "in avanti" (prima esecuzione o redo).
    /// Consulta anche `Undo`, non solo `Command`: un comando di rimozione
    /// a cascata (`RemoveNodeCascade`/`RemoveBlockCascade`) tocca più id
    /// di quanti `Command` da solo ne conosca, e solo `Undo` — costruito
    /// durante `apply`, quando il sottoalbero era ancora visibile — sa
    /// quali sono davvero.
    pub(crate) fn forward(commands: &[(Command, Undo)]) -> Self {
        let mut delta = Self::default();
        for (command, undo) in commands {
            match command.classify() {
                Some((id, kind)) => {
                    delta.push(id, kind);
                    for extra in undo.cascaded_ids() {
                        delta.push(extra, kind);
                    }
                }
                None => delta.meta_changed = true,
            }
        }
        delta
    }

    /// Delta di un undo: l'effetto visibile è l'opposto della
    /// classificazione del comando originale (un Insert diventa una
    /// rimozione agli occhi di chi osserva il documento, e viceversa; un
    /// Update resta un Update).
    pub(crate) fn backward(commands: &[(Command, Undo)]) -> Self {
        let mut delta = Self::default();
        for (command, undo) in commands {
            match command.classify() {
                Some((id, kind)) => {
                    let inverted = match kind {
                        ChangeKind::Created => ChangeKind::Removed,
                        ChangeKind::Removed => ChangeKind::Created,
                        ChangeKind::Updated => ChangeKind::Updated,
                    };
                    delta.push(id, inverted);
                    for extra in undo.cascaded_ids() {
                        delta.push(extra, inverted);
                    }
                }
                None => delta.meta_changed = true,
            }
        }
        delta
    }
}

impl Command {
    // --- Costruttori. Per i comandi di creazione generano subito l'id e
    // lo restituiscono insieme al comando, prima ancora che questo giri
    // nel bus.

    pub fn insert_node(kind: NodeKind, parent: ElementId, sort_order: SortKey) -> (Self, ElementId) {
        let id = ElementId::new();
        (Command::InsertNode { id, kind, parent, sort_order }, id)
    }

    pub fn remove_node(id: ElementId) -> Self {
        Command::RemoveNode { id }
    }

    pub fn set_node_title(id: ElementId, title: Option<String>) -> Self {
        Command::SetNodeTitle { id, title }
    }

    pub fn move_node(id: ElementId, new_parent: ElementId, new_sort_order: SortKey) -> Self {
        Command::MoveNode { id, new_parent, new_sort_order }
    }

    pub fn insert_block(
        payload: impl Into<BlockPayload>,
        layer_id: LayerId,
        parent: ElementId,
        sort_order: SortKey,
    ) -> (Self, ElementId) {
        let id = ElementId::new();
        (
            Command::InsertBlock {
                id,
                payload: payload.into(),
                layer_id,
                parent,
                sort_order,
            },
            id,
        )
    }

    pub fn remove_block(id: ElementId) -> Self {
        Command::RemoveBlock { id }
    }

    pub fn move_block(id: ElementId, new_parent: ElementId, new_sort_order: SortKey) -> Self {
        Command::MoveBlock { id, new_parent, new_sort_order }
    }

    pub fn insert_layer(name: impl Into<String>, color: Color, order: SortKey) -> (Self, ElementId) {
        let id = ElementId::new();
        (
            Command::InsertLayer {
                id,
                name: name.into(),
                color,
                order,
            },
            id,
        )
    }

    pub fn remove_layer(id: ElementId) -> Self {
        Command::RemoveLayer { id }
    }

    pub fn set_layer_visible(id: ElementId, visible: bool) -> Self {
        Command::SetLayerVisible { id, visible }
    }

    pub fn set_layer_locked(id: ElementId, locked: bool) -> Self {
        Command::SetLayerLocked { id, locked }
    }

    pub fn set_layer_printable(id: ElementId, printable: bool) -> Self {
        Command::SetLayerPrintable { id, printable }
    }

    pub fn set_layer_order(id: ElementId, new_order: SortKey) -> Self {
        Command::SetLayerOrder { id, new_order }
    }

    pub fn insert_character(name: impl Into<String>, alias: CharacterAlias) -> (Self, ElementId) {
        let id = ElementId::new();
        (
            Command::InsertCharacter {
                id,
                name: name.into(),
                alias,
            },
            id,
        )
    }

    pub fn remove_character(id: ElementId) -> Self {
        Command::RemoveCharacter { id }
    }

    pub fn rename_character(id: ElementId, new_name: impl Into<String>) -> Self {
        Command::RenameCharacter { id, new_name: new_name.into() }
    }

    pub fn rename_character_alias(id: ElementId, new_alias: CharacterAlias) -> Self {
        Command::RenameCharacterAlias { id, new_alias }
    }

    pub fn set_character_actor(id: ElementId, actor: Option<String>) -> Self {
        Command::SetCharacterActor { id, actor }
    }

    pub fn set_character_costume_notes(id: ElementId, notes: Option<String>) -> Self {
        Command::SetCharacterCostumeNotes { id, notes }
    }

    pub fn set_character_entrance_note(id: ElementId, note: Option<String>) -> Self {
        Command::SetCharacterEntranceNote { id, note }
    }

    pub fn set_character_exit_note(id: ElementId, note: Option<String>) -> Self {
        Command::SetCharacterExitNote { id, note }
    }

    pub fn reanchor_annotation(id: ElementId, new_anchor: AnchorTarget) -> Self {
        Command::ReanchorAnnotation { id, new_anchor }
    }

    pub fn set_doc_title(title: Option<String>) -> Self {
        Command::SetDocTitle(title)
    }

    pub fn set_doc_synopsis(synopsis: Option<String>) -> Self {
        Command::SetDocSynopsis(synopsis)
    }

    pub fn set_doc_setting(setting: Option<String>) -> Self {
        Command::SetDocSetting(setting)
    }

    pub fn set_doc_cover(cover: Option<CoverImage>) -> Self {
        Command::SetDocCover(cover)
    }

    pub fn set_doc_authorship(authorship: Authorship) -> Self {
        Command::SetDocAuthorship(authorship)
    }

    pub fn set_doc_production(production: Production) -> Self {
        Command::SetDocProduction(production)
    }

    pub fn set_doc_rights(copyright: Option<Copyright>) -> Self {
        Command::SetDocRights(copyright)
    }

    pub fn set_doc_credits(credits: Vec<(String, String)>) -> Self {
        Command::SetDocCredits(credits)
    }

    pub fn set_doc_status(status: ProjectStatus, distribution_note: Option<String>) -> Self {
        Command::SetDocStatus { status, distribution_note }
    }

    pub fn set_doc_version_major(major: u32) -> Self {
        Command::SetDocVersionMajor(major)
    }

    pub fn set_doc_version_sub(sub: u32) -> Self {
        Command::SetDocVersionSub(sub)
    }

    /// L'id principale a cui si riferisce questo comando, e come lo
    /// cambia — usato per costruire il `Delta` del batch. `None` per i
    /// comandi che toccano `DocMeta`, che non ha un proprio `ElementId`
    /// (è l'unico caso in cui `classify` restituisce `None`, ed è la
    /// convenzione che `Delta::forward`/`backward` usano per capire
    /// quando marcare `meta_changed` invece di spingere un id).
    pub(crate) fn classify(&self) -> Option<(ElementId, ChangeKind)> {
        use Command::*;
        match self {
            InsertNode { id, .. }
            | InsertBlock { id, .. }
            | InsertLayer { id, .. }
            | InsertCharacter { id, .. } => Some((*id, ChangeKind::Created)),
            RemoveNode { id } | RemoveBlock { id } | RemoveLayer { id } | RemoveCharacter { id } => {
                Some((*id, ChangeKind::Removed))
            }
            SetNodeTitle { id, .. }
            | MoveNode { id, .. }
            | MoveBlock { id, .. }
            | SetLayerVisible { id, .. }
            | SetLayerLocked { id, .. }
            | SetLayerPrintable { id, .. }
            | SetLayerOrder { id, .. }
            | RenameCharacter { id, .. }
            | RenameCharacterAlias { id, .. }
            | SetCharacterActor { id, .. }
            | SetCharacterCostumeNotes { id, .. }
            | SetCharacterEntranceNote { id, .. }
            | SetCharacterExitNote { id, .. }
            | ReanchorAnnotation { id, .. } => Some((*id, ChangeKind::Updated)),
            SetDocTitle(_)
            | SetDocSynopsis(_)
            | SetDocSetting(_)
            | SetDocCover(_)
            | SetDocAuthorship(_)
            | SetDocProduction(_)
            | SetDocRights(_)
            | SetDocCredits(_)
            | SetDocStatus { .. }
            | SetDocVersionMajor(_)
            | SetDocVersionSub(_) => None,
        }
    }

    /// Applica il comando al documento tramite i metodi già esposti da
    /// `Document`/`CharacterStore`, e restituisce il comando inverso.
    pub(crate) fn apply(&self, doc: &mut Document, at: Timestamp) -> CoreResult<Undo> {
        match self {
            Command::InsertNode { id, kind, parent, sort_order } => {
                // `Node::with_id` richiede `Option<ElementId>` perché deve
                // ospitare anche il caso del Root (parent `None`); qui il
                // comando garantisce già un parent concreto, quindi
                // l'avvolgimento in `Some` è solo al confine fra le due
                // rappresentazioni, non una scappatoia per reintrodurre
                // l'ambiguità che abbiamo appena eliminato dal `Command`.
                let node = Node::with_id(*id, kind.clone(), Some(*parent), sort_order.clone(), at);
                doc.insert_node(node);
                Ok(Undo::RemoveNode(*id))
            }
            Command::RemoveNode { id } => {
                if *id == ROOT_ID {
                    return Err(CoreError::CannotRemoveRoot);
                }
                let descendant_ids = doc.descendant_ids(*id);
                let mut all_ids = descendant_ids.clone();
                all_ids.push(*id);

                let mut removed_annotations = Vec::new();
                for eid in &all_ids {
                    removed_annotations.extend(doc.annotations_for(*eid).into_iter().cloned());
                }
                for ann in &removed_annotations {
                    doc.remove_annotation(ann.id());
                }

                let mut child_nodes = Vec::new();
                let mut child_blocks = Vec::new();
                for eid in &descendant_ids {
                    if let Some(n) = doc.remove_node(*eid) {
                        child_nodes.push(n);
                    } else if let Some(b) = doc.remove_block(*eid) {
                        child_blocks.push(b);
                    }
                }

                let node = doc.remove_node(*id).ok_or(CoreError::ElementNotFound(*id))?;
                Ok(Undo::RemoveNodeCascade { node, child_nodes, child_blocks, annotations: removed_annotations })
            }
            Command::SetNodeTitle { id, title } => {
                let node = doc.node_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = node.title.clone();
                node.title = title.clone();
                Ok(Undo::SetNodeTitle { id: *id, title: previous })
            }
            Command::MoveNode { id, new_parent, new_sort_order } => {
                if *id == ROOT_ID {
                    return Err(CoreError::CannotMoveRoot);
                }
                if new_parent == id {
                    return Err(CoreError::CyclicMove(*id));
                }
                if doc.node(*new_parent).is_none() {
                    return Err(CoreError::ElementNotFound(*new_parent));
                }
                if doc.is_descendant_of(*new_parent, *id) {
                    return Err(CoreError::CyclicMove(*id));
                }
                let node = doc.node(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous_parent = node.parent();
                let previous_sort_order = node.sort_order().clone();
                doc.node_mut(*id)
                    .expect("verificato sopra")
                    .reparent(Some(*new_parent), new_sort_order.clone());
                Ok(Undo::MoveNode { id: *id, parent: previous_parent, sort_order: previous_sort_order })
            }
            Command::InsertBlock { id, payload, layer_id, parent, sort_order } => {
                // Stesso confine Option/concreto di `InsertNode` qui sopra.
                let block = Block::with_id(*id, payload.clone(), *layer_id, Some(*parent), sort_order.clone(), at);
                doc.insert_block(block);
                Ok(Undo::RemoveBlock(*id))
            }
            Command::RemoveBlock { id } => {
                let removed_annotations: Vec<Annotation> =
                    doc.annotations_for(*id).into_iter().cloned().collect();
                for ann in &removed_annotations {
                    doc.remove_annotation(ann.id());
                }
                let block = doc.remove_block(*id).ok_or(CoreError::ElementNotFound(*id))?;
                Ok(Undo::RemoveBlockCascade { block, annotations: removed_annotations })
            }
            Command::MoveBlock { id, new_parent, new_sort_order } => {
                if doc.node(*new_parent).is_none() {
                    return Err(CoreError::ElementNotFound(*new_parent));
                }
                let block = doc.block(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous_parent = block.parent();
                let previous_sort_order = block.sort_order().clone();
                doc.block_mut(*id)
                    .expect("verificato sopra")
                    .reparent(Some(*new_parent), new_sort_order.clone());
                Ok(Undo::MoveBlock { id: *id, parent: previous_parent, sort_order: previous_sort_order })
            }
            Command::InsertLayer { id, name, color, order } => {
                let layer = Layer::with_id(*id, name.clone(), *color, order.clone(), at);
                doc.insert_layer(layer);
                Ok(Undo::RemoveLayer(*id))
            }
            Command::RemoveLayer { id } => {
                let removed = doc.remove_layer(*id).ok_or(CoreError::ElementNotFound(*id))?;
                Ok(Undo::ReinsertLayer(removed))
            }
            Command::SetLayerVisible { id, visible } => {
                let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = layer.visible;
                layer.visible = *visible;
                Ok(Undo::SetLayerVisible { id: *id, visible: previous })
            }
            Command::SetLayerLocked { id, locked } => {
                let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = layer.locked;
                layer.locked = *locked;
                Ok(Undo::SetLayerLocked { id: *id, locked: previous })
            }
            Command::SetLayerPrintable { id, printable } => {
                let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = layer.printable;
                layer.printable = *printable;
                Ok(Undo::SetLayerPrintable { id: *id, printable: previous })
            }
            Command::SetLayerOrder { id, new_order } => {
                let layer = doc.layer_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = layer.order.clone();
                layer.order = new_order.clone();
                Ok(Undo::SetLayerOrder { id: *id, order: previous })
            }
            Command::InsertCharacter { id, name, alias } => {
                let character = Character::with_id(*id, name.clone(), alias.clone(), at);
                doc.insert_character(character)?;
                Ok(Undo::RemoveCharacter(*id))
            }
            Command::RemoveCharacter { id } => {
                let removed = doc.remove_character(*id).ok_or(CoreError::ElementNotFound(*id))?;
                Ok(Undo::ReinsertCharacter(removed))
            }
            Command::RenameCharacter { id, new_name } => {
                let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = character.name.clone();
                character.name = new_name.clone();
                Ok(Undo::RenameCharacter { id: *id, name: previous })
            }
            Command::RenameCharacterAlias { id, new_alias } => {
                let previous = doc
                    .character(*id)
                    .ok_or(CoreError::ElementNotFound(*id))?
                    .alias()
                    .clone();
                doc.rename_character_alias(*id, new_alias.clone())?;
                Ok(Undo::RenameCharacterAlias { id: *id, alias: previous })
            }
            Command::SetCharacterActor { id, actor } => {
                let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = character.actor.clone();
                character.actor = actor.clone();
                Ok(Undo::SetCharacterActor { id: *id, actor: previous })
            }
            Command::SetCharacterCostumeNotes { id, notes } => {
                let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = character.costume_notes.clone();
                character.costume_notes = notes.clone();
                Ok(Undo::SetCharacterCostumeNotes { id: *id, notes: previous })
            }
            Command::SetCharacterEntranceNote { id, note } => {
                let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = character.entrance_notes.clone();
                character.entrance_notes = note.clone();
                Ok(Undo::SetCharacterEntranceNote { id: *id, note: previous })
            }
            Command::SetCharacterExitNote { id, note } => {
                let character = doc.character_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = character.exit_notes.clone();
                character.exit_notes = note.clone();
                Ok(Undo::SetCharacterExitNote { id: *id, note: previous })
            }
            Command::ReanchorAnnotation { id, new_anchor } => {
                let annotation = doc.annotation_mut(*id).ok_or(CoreError::ElementNotFound(*id))?;
                let previous = annotation.anchor;
                annotation.anchor = *new_anchor;
                Ok(Undo::ReanchorAnnotation { id: *id, anchor: previous })
            }
            Command::SetDocTitle(title) => {
                let previous = doc.metadata.title.clone();
                doc.metadata.title = title.clone();
                Ok(Undo::SetDocTitle(previous))
            }
            Command::SetDocSynopsis(synopsis) => {
                let previous = doc.metadata.synopsis.clone();
                doc.metadata.synopsis = synopsis.clone();
                Ok(Undo::SetDocSynopsis(previous))
            }
            Command::SetDocSetting(setting) => {
                let previous = doc.metadata.setting.clone();
                doc.metadata.setting = setting.clone();
                Ok(Undo::SetDocSetting(previous))
            }
            Command::SetDocCover(cover) => {
                let previous = doc.metadata.cover.clone();
                doc.metadata.cover = cover.clone();
                Ok(Undo::SetDocCover(previous))
            }
            Command::SetDocAuthorship(authorship) => {
                let previous = doc.metadata.authorship.clone();
                doc.metadata.authorship = authorship.clone();
                Ok(Undo::SetDocAuthorship(previous))
            }
            Command::SetDocProduction(production) => {
                let previous = doc.metadata.production.clone();
                doc.metadata.production = production.clone();
                Ok(Undo::SetDocProduction(previous))
            }
            Command::SetDocRights(copyright) => {
                let previous = doc.metadata.copyright.clone();
                doc.metadata.copyright = copyright.clone();
                Ok(Undo::SetDocRights(previous))
            }
            Command::SetDocCredits(credits) => {
                let previous = doc.metadata.credits.clone();
                doc.metadata.credits = credits.clone();
                Ok(Undo::SetDocCredits(previous))
            }
            Command::SetDocStatus { status, distribution_note } => {
                let previous_status = doc.metadata.status;
                let previous_note = doc.metadata.distribution_note.clone();
                doc.metadata.status = *status;
                doc.metadata.distribution_note = distribution_note.clone();
                Ok(Undo::SetDocStatus { status: previous_status, distribution_note: previous_note })
            }
            Command::SetDocVersionMajor(major) => {
                let previous = doc.metadata.version;
                doc.metadata.version = Version::new(*major, 0);
                Ok(Undo::SetDocVersion(previous))
            }
            Command::SetDocVersionSub(sub) => {
                let previous = doc.metadata.version;
                doc.metadata.version.minor = *sub;
                Ok(Undo::SetDocVersion(previous))
            }
        }
    }
}

impl Undo {
    /// Id aggiuntivi coinvolti da un comando "a cascata", oltre all'id
    /// principale restituito da `Command::classify` — con la stessa
    /// classificazione del comando che li ha generati. Per ora sempre
    /// `Removed`, perché le uniche cascate esistenti sono le rimozioni;
    /// se in futuro nascesse una cascata "in creazione" andrà rivista
    /// l'assunzione (oggi implicita in `Delta::forward`/`backward`) che
    /// gli id extra condividano sempre lo stesso `ChangeKind` del comando.
    fn cascaded_ids(&self) -> Vec<ElementId> {
        match self {
            Undo::RemoveNodeCascade { child_nodes, child_blocks, annotations, .. } => child_nodes
                .iter()
                .map(|n| n.id())
                .chain(child_blocks.iter().map(|b| b.id()))
                .chain(annotations.iter().map(|a| a.id()))
                .collect(),
            Undo::RemoveBlockCascade { annotations, .. } => annotations.iter().map(|a| a.id()).collect(),
            _ => Vec::new(),
        }
    }

    /// Applica l'inverso: riporta il documento allo stato precedente al
    /// comando originale. In una cronologia lineare corretta questo non
    /// dovrebbe mai fallire; se fallisce (es. `ReinsertCharacter` su un
    /// alias nel frattempo riassegnato da un percorso che ha bypassato il
    /// bus) segnala una history incoerente, non un caso d'uso normale.
    pub(crate) fn revert(&self, doc: &mut Document) -> CoreResult<()> {
        match self {
            Undo::RemoveNode(id) => {
                doc.remove_node(*id);
                Ok(())
            }
            Undo::RemoveNodeCascade { node, child_nodes, child_blocks, annotations } => {
                doc.insert_node(node.clone());
                for n in child_nodes {
                    doc.insert_node(n.clone());
                }
                for b in child_blocks {
                    doc.insert_block(b.clone());
                }
                for a in annotations {
                    doc.insert_annotation(a.clone());
                }
                Ok(())
            }
            Undo::SetNodeTitle { id, title } => {
                if let Some(node) = doc.node_mut(*id) {
                    node.title = title.clone();
                }
                Ok(())
            }
            Undo::MoveNode { id, parent, sort_order } => {
                if let Some(node) = doc.node_mut(*id) {
                    node.reparent(*parent, sort_order.clone());
                }
                Ok(())
            }
            Undo::RemoveBlock(id) => {
                doc.remove_block(*id);
                Ok(())
            }
            Undo::RemoveBlockCascade { block, annotations } => {
                doc.insert_block(block.clone());
                for a in annotations {
                    doc.insert_annotation(a.clone());
                }
                Ok(())
            }
            Undo::MoveBlock { id, parent, sort_order } => {
                if let Some(block) = doc.block_mut(*id) {
                    block.reparent(*parent, sort_order.clone());
                }
                Ok(())
            }
            Undo::RemoveLayer(id) => {
                doc.remove_layer(*id);
                Ok(())
            }
            Undo::ReinsertLayer(layer) => {
                doc.insert_layer(layer.clone());
                Ok(())
            }
            Undo::SetLayerVisible { id, visible } => {
                if let Some(layer) = doc.layer_mut(*id) {
                    layer.visible = *visible;
                }
                Ok(())
            }
            Undo::SetLayerLocked { id, locked } => {
                if let Some(layer) = doc.layer_mut(*id) {
                    layer.locked = *locked;
                }
                Ok(())
            }
            Undo::SetLayerPrintable { id, printable } => {
                if let Some(layer) = doc.layer_mut(*id) {
                    layer.printable = *printable;
                }
                Ok(())
            }
            Undo::SetLayerOrder { id, order } => {
                if let Some(layer) = doc.layer_mut(*id) {
                    layer.order = order.clone();
                }
                Ok(())
            }
            Undo::RemoveCharacter(id) => {
                doc.remove_character(*id);
                Ok(())
            }
            Undo::ReinsertCharacter(character) => {
                doc.insert_character(character.clone())?;
                Ok(())
            }
            Undo::RenameCharacter { id, name } => {
                if let Some(character) = doc.character_mut(*id) {
                    character.name = name.clone();
                }
                Ok(())
            }
            Undo::RenameCharacterAlias { id, alias } => {
                doc.rename_character_alias(*id, alias.clone())?;
                Ok(())
            }
            Undo::SetCharacterActor { id, actor } => {
                if let Some(character) = doc.character_mut(*id) {
                    character.actor = actor.clone();
                }
                Ok(())
            }
            Undo::SetCharacterCostumeNotes { id, notes } => {
                if let Some(character) = doc.character_mut(*id) {
                    character.costume_notes = notes.clone();
                }
                Ok(())
            }
            Undo::SetCharacterEntranceNote { id, note } => {
                if let Some(character) = doc.character_mut(*id) {
                    character.entrance_notes = note.clone();
                }
                Ok(())
            }
            Undo::SetCharacterExitNote { id, note } => {
                if let Some(character) = doc.character_mut(*id) {
                    character.exit_notes = note.clone();
                }
                Ok(())
            }
            Undo::ReanchorAnnotation { id, anchor } => {
                if let Some(annotation) = doc.annotation_mut(*id) {
                    annotation.anchor = *anchor;
                }
                Ok(())
            }
            Undo::SetDocTitle(title) => {
                doc.metadata.title = title.clone();
                Ok(())
            }
            Undo::SetDocSynopsis(synopsis) => {
                doc.metadata.synopsis = synopsis.clone();
                Ok(())
            }
            Undo::SetDocSetting(setting) => {
                doc.metadata.setting = setting.clone();
                Ok(())
            }
            Undo::SetDocCover(cover) => {
                doc.metadata.cover = cover.clone();
                Ok(())
            }
            Undo::SetDocAuthorship(authorship) => {
                doc.metadata.authorship = authorship.clone();
                Ok(())
            }
            Undo::SetDocProduction(production) => {
                doc.metadata.production = production.clone();
                Ok(())
            }
            Undo::SetDocRights(copyright) => {
                doc.metadata.copyright = copyright.clone();
                Ok(())
            }
            Undo::SetDocCredits(credits) => {
                doc.metadata.credits = credits.clone();
                Ok(())
            }
            Undo::SetDocStatus { status, distribution_note } => {
                doc.metadata.status = *status;
                doc.metadata.distribution_note = distribution_note.clone();
                Ok(())
            }
            Undo::SetDocVersion(version) => {
                doc.metadata.version = *version;
                Ok(())
            }
        }
    }
}