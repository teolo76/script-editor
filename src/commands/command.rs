// src/command/command.rs

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
//!
//! Lo stesso principio, un livello più in alto, raggruppa `Command` per
//! **dominio**: `Node`, `Block`, `Layer`, `Character`, `Annotation`,
//! `DocMeta`. Ogni dominio ha il proprio sotto-enum di comando/inverso e
//! la propria logica di apply/revert in un file a parte sotto
//! `commands/` (`node.rs`, `block.rs`, ...) — qui restano solo i
//! costruttori pubblici (thin wrapper, nessuna logica) e il dispatch.
//! Resta un solo `Command`, un solo `Undo`, un solo `CommandBus`: la
//! cronologia undo/redo è e deve restare unica per documento. Più bus
//! separati (uno per dominio) fratturerebbero quella cronologia — un
//! Ctrl+Z diventerebbe ambiguo su quale dei due disfare — quindi qui
//! raggruppiamo solo il *codice*, non il runtime. Un bus separato resta
//! corretto solo per un *documento diverso* con un proprio ciclo di vita
//! di undo (es. un ipotetico editor di temi), non per un'altra area dello
//! stesso documento.

use crate::core::{AnchorTarget, Authorship, BlockPayload, CoverImage, LayerId, Production};
use crate::errors::CoreResult;
use crate::lake::Document;
use crate::types::{
    CharacterAlias, Color, Copyright, ElementId, NodeKind, ProjectStatus, SortKey, Timestamp,
};

use super::annotation_cmd::{self as annotation_mod, AnnotationCommand, AnnotationUndo};
use super::block_cmd::{self as block_mod, BlockCommand, BlockUndo};
use super::character_cmd::{self as character_mod, CharacterCommand, CharacterUndo};
use super::doc_meta_cmd::{self as doc_meta_mod, DocMetaCommand, DocMetaUndo};
use super::layer_cmd::{self as layer_mod, LayerCommand, LayerUndo};
use super::node_cmd::{self as node_mod, NodeCommand, NodeUndo};

#[derive(Debug, PartialEq)]
pub enum Command {
    Node(NodeCommand),
    Block(BlockCommand),
    Layer(LayerCommand),
    Character(CharacterCommand),
    Annotation(AnnotationCommand),
    DocMeta(DocMetaCommand),
}

/// L'inverso di un `Command` già applicato: quello che serve per tornare
/// indietro. Non è pensato per essere costruito a mano — lo produce
/// `Command::apply`, e lo consuma solo `CommandBus`.
#[derive(Debug, PartialEq)]
pub(crate) enum Undo {
    Node(NodeUndo),
    Block(BlockUndo),
    Layer(LayerUndo),
    Character(CharacterUndo),
    Annotation(AnnotationUndo),
    DocMeta(DocMetaUndo),
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
    /// a cascata, o uno che sposta annotazioni insieme al testo, tocca
    /// più id di quanti `Command` da solo ne conosca, e solo `Undo` —
    /// costruito durante `apply`, quando lo stato precedente era ancora
    /// visibile — sa quali sono davvero.
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
    // nel bus. Thin wrapper: nessuna logica qui, solo l'annidamento nel
    // sotto-enum del dominio giusto.

    pub fn insert_node(kind: NodeKind, parent: ElementId, sort_order: SortKey) -> (Self, ElementId) {
        let id = ElementId::new();
        (Command::Node(NodeCommand::Insert { id, kind, parent, sort_order }), id)
    }

    pub fn remove_node(id: ElementId) -> Self {
        Command::Node(NodeCommand::Remove { id })
    }

    pub fn set_node_title(id: ElementId, title: Option<String>) -> Self {
        Command::Node(NodeCommand::SetTitle { id, title })
    }

    pub fn move_node(id: ElementId, new_parent: ElementId, new_sort_order: SortKey) -> Self {
        Command::Node(NodeCommand::Move { id, new_parent, new_sort_order })
    }

    pub fn insert_block(
        payload: impl Into<BlockPayload>,
        layer_id: LayerId,
        parent: ElementId,
        sort_order: SortKey,
    ) -> (Self, ElementId) {
        let id = ElementId::new();
        (
            Command::Block(BlockCommand::Insert { id, payload: payload.into(), layer_id, parent, sort_order }),
            id,
        )
    }

    pub fn remove_block(id: ElementId) -> Self {
        Command::Block(BlockCommand::Remove { id })
    }

    pub fn move_block(id: ElementId, new_parent: ElementId, new_sort_order: SortKey) -> Self {
        Command::Block(BlockCommand::Move { id, new_parent, new_sort_order })
    }

    pub fn insert_text(block_id: ElementId, offset: usize, text: impl Into<String>) -> Self {
        Command::Block(BlockCommand::InsertText { block_id, offset, text: text.into() })
    }

    pub fn delete_text(block_id: ElementId, start: usize, end: usize) -> Self {
        Command::Block(BlockCommand::DeleteText { block_id, start, end })
    }

    pub fn insert_layer(name: impl Into<String>, color: Color, order: SortKey) -> (Self, ElementId) {
        let id = ElementId::new();
        (Command::Layer(LayerCommand::Insert { id, name: name.into(), color, order }), id)
    }

    pub fn remove_layer(id: ElementId) -> Self {
        Command::Layer(LayerCommand::Remove { id })
    }

    pub fn set_layer_visible(id: ElementId, visible: bool) -> Self {
        Command::Layer(LayerCommand::SetVisible { id, visible })
    }

    pub fn set_layer_locked(id: ElementId, locked: bool) -> Self {
        Command::Layer(LayerCommand::SetLocked { id, locked })
    }

    pub fn set_layer_printable(id: ElementId, printable: bool) -> Self {
        Command::Layer(LayerCommand::SetPrintable { id, printable })
    }

    pub fn set_layer_order(id: ElementId, new_order: SortKey) -> Self {
        Command::Layer(LayerCommand::SetOrder { id, new_order })
    }

    pub fn insert_character(name: impl Into<String>, alias: CharacterAlias) -> (Self, ElementId) {
        let id = ElementId::new();
        (Command::Character(CharacterCommand::Insert { id, name: name.into(), alias }), id)
    }

    pub fn remove_character(id: ElementId) -> Self {
        Command::Character(CharacterCommand::Remove { id })
    }

    pub fn rename_character(id: ElementId, new_name: impl Into<String>) -> Self {
        Command::Character(CharacterCommand::Rename { id, new_name: new_name.into() })
    }

    pub fn rename_character_alias(id: ElementId, new_alias: CharacterAlias) -> Self {
        Command::Character(CharacterCommand::RenameAlias { id, new_alias })
    }

    pub fn set_character_actor(id: ElementId, actor: Option<String>) -> Self {
        Command::Character(CharacterCommand::SetActor { id, actor })
    }

    pub fn set_character_costume_notes(id: ElementId, notes: Option<String>) -> Self {
        Command::Character(CharacterCommand::SetCostumeNotes { id, notes })
    }

    pub fn set_character_entrance_note(id: ElementId, note: Option<String>) -> Self {
        Command::Character(CharacterCommand::SetEntranceNote { id, note })
    }

    pub fn set_character_exit_note(id: ElementId, note: Option<String>) -> Self {
        Command::Character(CharacterCommand::SetExitNote { id, note })
    }

    pub fn insert_annotation(layer_id: LayerId, anchor: AnchorTarget, body: impl Into<String>) -> (Self, ElementId) {
        let id = ElementId::new();
        (Command::Annotation(AnnotationCommand::Insert { id, layer_id, anchor, body: body.into() }), id)
    }

    pub fn remove_annotation(id: ElementId) -> Self {
        Command::Annotation(AnnotationCommand::Remove { id })
    }

    pub fn reanchor_annotation(id: ElementId, new_anchor: AnchorTarget) -> Self {
        Command::Annotation(AnnotationCommand::Reanchor { id, new_anchor })
    }

    pub fn set_doc_title(title: Option<String>) -> Self {
        Command::DocMeta(DocMetaCommand::Title(title))
    }

    pub fn set_doc_synopsis(synopsis: Option<String>) -> Self {
        Command::DocMeta(DocMetaCommand::Synopsis(synopsis))
    }

    pub fn set_doc_setting(setting: Option<String>) -> Self {
        Command::DocMeta(DocMetaCommand::Setting(setting))
    }

    pub fn set_doc_cover(cover: Option<CoverImage>) -> Self {
        Command::DocMeta(DocMetaCommand::Cover(cover))
    }

    pub fn set_doc_authorship(authorship: Authorship) -> Self {
        Command::DocMeta(DocMetaCommand::Authorship(authorship))
    }

    pub fn set_doc_production(production: Production) -> Self {
        Command::DocMeta(DocMetaCommand::Production(production))
    }

    pub fn set_doc_rights(copyright: Option<Copyright>) -> Self {
        Command::DocMeta(DocMetaCommand::Rights(copyright))
    }

    pub fn set_doc_credits(credits: Vec<(String, String)>) -> Self {
        Command::DocMeta(DocMetaCommand::Credits(credits))
    }

    pub fn set_doc_status(status: ProjectStatus, distribution_note: Option<String>) -> Self {
        Command::DocMeta(DocMetaCommand::Status { status, distribution_note })
    }

    pub fn set_doc_version_major(major: u32) -> Self {
        Command::DocMeta(DocMetaCommand::VersionMajor(major))
    }

    pub fn set_doc_version_sub(sub: u32) -> Self {
        Command::DocMeta(DocMetaCommand::VersionSub(sub))
    }

    /// L'id principale a cui si riferisce questo comando, e come lo
    /// cambia — usato per costruire il `Delta` del batch. `None` per i
    /// comandi che toccano `DocMeta`, che non ha un proprio `ElementId`
    /// (è l'unico dominio per cui `classify` restituisce `None`, ed è la
    /// convenzione che `Delta::forward`/`backward` usano per capire
    /// quando marcare `meta_changed` invece di spingere un id).
    pub(crate) fn classify(&self) -> Option<(ElementId, ChangeKind)> {
        match self {
            Command::Node(c) => Some(node_mod::classify(c)),
            Command::Block(c) => Some(block_mod::classify(c)),
            Command::Layer(c) => Some(layer_mod::classify(c)),
            Command::Character(c) => Some(character_mod::classify(c)),
            Command::Annotation(c) => Some(annotation_mod::classify(c)),
            Command::DocMeta(_) => None,
        }
    }

    /// Applica il comando al documento, delegando al modulo del dominio
    /// giusto, e restituisce il comando inverso.
    pub(crate) fn apply(&self, doc: &mut Document, at: Timestamp) -> CoreResult<Undo> {
        match self {
            Command::Node(c) => node_mod::apply(c, doc, at).map(Undo::Node),
            Command::Block(c) => block_mod::apply(c, doc, at).map(Undo::Block),
            Command::Layer(c) => layer_mod::apply(c, doc, at).map(Undo::Layer),
            Command::Character(c) => character_mod::apply(c, doc, at).map(Undo::Character),
            Command::Annotation(c) => annotation_mod::apply(c, doc, at).map(Undo::Annotation),
            Command::DocMeta(c) => doc_meta_mod::apply(c, doc, at).map(Undo::DocMeta),
        }
    }
}

impl Undo {
    /// Id aggiuntivi coinvolti da un comando oltre all'id principale
    /// restituito da `Command::classify` — con la stessa classificazione
    /// del comando che li ha generati (vedi i commenti nei singoli
    /// moduli di dominio, es. `block::cascaded_ids`, su perché questo
    /// resta valido anche per i casi non-rimozione come `InsertText`).
    fn cascaded_ids(&self) -> Vec<ElementId> {
        match self {
            Undo::Node(u) => node_mod::cascaded_ids(u),
            Undo::Block(u) => block_mod::cascaded_ids(u),
            Undo::Layer(u) => layer_mod::cascaded_ids(u),
            Undo::Character(u) => character_mod::cascaded_ids(u),
            Undo::Annotation(u) => annotation_mod::cascaded_ids(u),
            Undo::DocMeta(_) => Vec::new(),
        }
    }

    /// Applica l'inverso: riporta il documento allo stato precedente al
    /// comando originale. In una cronologia lineare corretta questo non
    /// dovrebbe mai fallire; se fallisce segnala una history incoerente,
    /// non un caso d'uso normale.
    pub(crate) fn revert(&self, doc: &mut Document) -> CoreResult<()> {
        match self {
            Undo::Node(u) => node_mod::revert(u, doc),
            Undo::Block(u) => block_mod::revert(u, doc),
            Undo::Layer(u) => layer_mod::revert(u, doc),
            Undo::Character(u) => character_mod::revert(u, doc),
            Undo::Annotation(u) => annotation_mod::revert(u, doc),
            Undo::DocMeta(u) => doc_meta_mod::revert(u, doc),
        }
    }
}