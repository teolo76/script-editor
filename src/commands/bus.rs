//! Il bus dei comandi: applica batch atomici al `Document`, mantiene la
//! cronologia undo/redo e produce un `Delta` invece di restituire il
//! documento intero.

use crate::commands::command::{Command, Delta, Undo};
use crate::errors::CoreResult;
use crate::lake::Document;
use crate::types::Timestamp;

/// Un batch già applicato: i comandi originali (per il redo) accoppiati
/// al loro inverso (per l'undo), più l'istante in cui il batch fu
/// applicato — così un redo non ha bisogno che il chiamante fornisca di
/// nuovo un timestamp, riusa quello originale.
struct HistoryEntry {
    at: Timestamp,
    commands: Vec<(Command, Undo)>,
}

#[derive(Default)]
pub struct CommandBus {
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Applica un batch di comandi atomicamente: se uno fallisce, disfa
    /// (in ordine inverso) quanto già applicato nello stesso batch e
    /// restituisce l'errore, senza toccare la history né la revisione del
    /// documento. Un batch riuscito produce una sola voce di cronologia e
    /// un solo avanzamento di revisione, qualunque sia il numero di
    /// comandi al suo interno.
    pub fn apply(&mut self, doc: &mut Document, commands: Vec<Command>, at: Timestamp) -> CoreResult<Delta> {
        let mut applied: Vec<(Command, Undo)> = Vec::with_capacity(commands.len());
        for command in commands {
            match command.apply(doc, at) {
                Ok(undo) => applied.push((command, undo)),
                Err(err) => {
                    for (_, undo) in applied.into_iter().rev() {
                        // Un fallimento qui, durante il rollback di un
                        // batch mai andato a buon fine, indicherebbe a sua
                        // volta uno stato incoerente: lo ignoriamo perché
                        // l'obiettivo è tornare il più vicino possibile
                        // allo stato di partenza, non propagare un secondo
                        // errore sopra al primo.
                        let _ = undo.revert(doc);
                    }
                    return Err(err);
                }
            }
        }
        let delta = Delta::forward(&applied);
        self.undo_stack.push(HistoryEntry { at, commands: applied });
        self.redo_stack.clear();
        doc.bump_revision();
        Ok(delta)
    }

    /// Disfa l'ultimo batch applicato. `Ok(None)` se non c'è nulla da
    /// disfare.
    ///
    /// Nota su un caso limite non gestito: se il revert di un comando a
    /// metà batch fallisse, l'entry andrebbe persa (né in undo né in redo
    /// stack) lasciando il documento parzialmente disfatto. In una
    /// cronologia lineare corretta questo non dovrebbe accadere (vedi
    /// `Undo::revert`); se succede in pratica, meriterebbe un rollback del
    /// rollback che qui non implementiamo.
    pub fn undo(&mut self, doc: &mut Document) -> CoreResult<Option<Delta>> {
        let Some(entry) = self.undo_stack.pop() else {
            return Ok(None);
        };
        for (_, undo) in entry.commands.iter().rev() {
            undo.revert(doc)?;
        }
        let delta = Delta::backward(&entry.commands);
        self.redo_stack.push(entry);
        doc.bump_revision();
        Ok(Some(delta))
    }

    /// Riapplica l'ultimo batch disfatto. `Ok(None)` se non c'è nulla da
    /// rifare. Riusa il comando originale (stesso id, stesso timestamp
    /// della prima applicazione): non ricalcola nulla, l'inverso resta
    /// quello già in `entry`.
    pub fn redo(&mut self, doc: &mut Document) -> CoreResult<Option<Delta>> {
        let Some(entry) = self.redo_stack.pop() else {
            return Ok(None);
        };
        for (command, _) in &entry.commands {
            command.apply(doc, entry.at)?;
        }
        let delta = Delta::forward(&entry.commands);
        self.undo_stack.push(entry);
        doc.bump_revision();
        Ok(Some(delta))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{AnchorTarget, Annotation, Authorship, LayerId, Production, TextBlock, TreeElement};
    use crate::errors::CoreError;
    use crate::types::{BlockKind, CharacterAlias, Color, Copyright, ElementId, NodeKind, ProjectStatus, SortKey, Version};

    fn t(secs: u64) -> Timestamp {
        Timestamp::from_unix_seconds(secs)
    }

    fn alias(s: &str) -> CharacterAlias {
        CharacterAlias::new(s).unwrap()
    }

    #[test]
    fn insert_node_then_undo_removes_it() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (cmd, id) = Command::insert_node(NodeKind::Act, doc.root_id(), SortKey::initial());

        let delta = bus.apply(&mut doc, vec![cmd], t(0)).unwrap();
        assert_eq!(delta.created, vec![id]);
        assert!(doc.node(id).is_some());

        let undo_delta = bus.undo(&mut doc).unwrap().unwrap();
        assert_eq!(undo_delta.removed, vec![id]);
        assert!(doc.node(id).is_none());
        assert!(bus.can_redo());
        assert!(!bus.can_undo());
    }

    #[test]
    fn redo_restores_same_id() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (cmd, id) = Command::insert_node(NodeKind::Scene, doc.root_id(), SortKey::initial());

        bus.apply(&mut doc, vec![cmd], t(0)).unwrap();
        bus.undo(&mut doc).unwrap();
        assert!(doc.node(id).is_none());

        let redo_delta = bus.redo(&mut doc).unwrap().unwrap();
        assert_eq!(redo_delta.created, vec![id]);
        assert!(doc.node(id).is_some());
        assert!(bus.can_undo());
        assert!(!bus.can_redo());
    }

    #[test]
    fn batch_rolls_back_atomically_on_failure() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();

        let (insert_ok, ok_id) = Command::insert_character("Giulietta", alias("GIU"));
        let (insert_fail, _) = Command::insert_character("Giuseppe", alias("GIU"));

        let err = bus.apply(&mut doc, vec![insert_ok, insert_fail], t(0)).unwrap_err();
        assert!(matches!(err, CoreError::DuplicateAlias(_)));
        // Il primo comando, benché valido da solo, deve essere stato disfatto:
        // il batch è tutto o niente.
        assert!(doc.character(ok_id).is_none());
        assert!(!bus.can_undo());
    }

    #[test]
    fn set_layer_visible_undo_restores_previous_value() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (cmd, id) = Command::insert_layer("Note di regia", Color::rgb(200, 50, 50), SortKey::initial());
        bus.apply(&mut doc, vec![cmd], t(0)).unwrap();
        assert!(doc.layer(id).unwrap().visible);

        bus.apply(&mut doc, vec![Command::set_layer_visible(id, false)], t(0)).unwrap();
        assert!(!doc.layer(id).unwrap().visible);

        bus.undo(&mut doc).unwrap();
        assert!(doc.layer(id).unwrap().visible);
    }

    #[test]
    fn rename_character_alias_undo_restores_old_alias() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (cmd, id) = Command::insert_character("Romeo", alias("ROM"));
        bus.apply(&mut doc, vec![cmd], t(0)).unwrap();

        bus.apply(&mut doc, vec![Command::rename_character_alias(id, alias("ROX"))], t(0))
            .unwrap();
        assert_eq!(doc.resolve_character_alias(&alias("ROX")), Some(id));

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.resolve_character_alias(&alias("ROM")), Some(id));
        assert_eq!(doc.resolve_character_alias(&alias("ROX")), None);
    }

    #[test]
    fn rename_character_undo_restores_old_name() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (cmd, id) = Command::insert_character("Romeo", alias("ROM"));
        bus.apply(&mut doc, vec![cmd], t(0)).unwrap();

        bus.apply(&mut doc, vec![Command::rename_character(id, "Romeo Montecchi")], t(0))
            .unwrap();
        assert_eq!(doc.character(id).unwrap().name, "Romeo Montecchi");
        // L'alias non è toccato da questo comando: resta decoupled dal nome.
        assert_eq!(doc.resolve_character_alias(&alias("ROM")), Some(id));

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.character(id).unwrap().name, "Romeo");
    }

    #[test]
    fn batch_bumps_revision_once_and_undoes_as_one_unit() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let r0 = doc.revision();

        let (cmd1, node_id) = Command::insert_node(NodeKind::Act, doc.root_id(), SortKey::initial());
        let (cmd2, layer_id) = Command::insert_layer("Base", Color::rgb(0, 0, 0), SortKey::initial());
        let cmd3 = Command::set_node_title(node_id, Some("Atto I".to_string()));

        bus.apply(&mut doc, vec![cmd1, cmd2, cmd3], t(0)).unwrap();
        assert_eq!(doc.revision(), r0.next());
        assert_eq!(doc.node(node_id).unwrap().title.as_deref(), Some("Atto I"));

        // Una singola entry di history: un solo undo disfa tutti e tre i comandi.
        bus.undo(&mut doc).unwrap();
        assert!(doc.node(node_id).is_none());
        assert!(doc.layer(layer_id).is_none());
    }

    #[test]
    fn remove_block_then_undo_reinserts_with_same_content() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let layer_id = crate::core::LayerId::new();

        let (insert, block_id) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Ma soft!"),
            layer_id,
            doc.root_id(),
            SortKey::initial(),
        );
        bus.apply(&mut doc, vec![insert], t(0)).unwrap();

        bus.apply(&mut doc, vec![Command::remove_block(block_id)], t(0)).unwrap();
        assert!(doc.block(block_id).is_none());

        bus.undo(&mut doc).unwrap();
        let restored = doc.block(block_id).expect("il blocco deve essere tornato");
        assert_eq!(restored.as_text().unwrap().text, "Ma soft!");
    }

    // --- MoveNode / MoveBlock ---

    #[test]
    fn move_node_reparents_and_undo_restores_previous_position() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();

        let (insert_act1, act1) = Command::insert_node(NodeKind::Act, root_id, SortKey::initial());
        let (insert_act2, act2) = Command::insert_node(NodeKind::Act, root_id, SortKey::initial().after());
        let (insert_scene, scene) = Command::insert_node(NodeKind::Scene, act1, SortKey::initial());
        bus.apply(&mut doc, vec![insert_act1, insert_act2, insert_scene], t(0)).unwrap();

        assert_eq!(doc.node(scene).unwrap().parent(), Some(act1));

        bus.apply(&mut doc, vec![Command::move_node(scene, act2, SortKey::initial())], t(0))
            .unwrap();
        assert_eq!(doc.node(scene).unwrap().parent(), Some(act2));

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.node(scene).unwrap().parent(), Some(act1));
    }

    #[test]
    fn move_node_rejects_cycle_into_own_descendant() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();

        let (insert_act, act) = Command::insert_node(NodeKind::Act, root_id, SortKey::initial());
        let (insert_scene, scene) = Command::insert_node(NodeKind::Scene, act, SortKey::initial());
        bus.apply(&mut doc, vec![insert_act, insert_scene], t(0)).unwrap();

        // Spostare l'Atto dentro la propria Scena creerebbe un ciclo.
        let err = bus
            .apply(&mut doc, vec![Command::move_node(act, scene, SortKey::initial())], t(0))
            .unwrap_err();
        assert!(matches!(err, CoreError::CyclicMove(id) if id == act));
        // Nulla deve essere cambiato: l'Atto è ancora dove era.
        assert_eq!(doc.node(act).unwrap().parent(), Some(root_id));
    }

    #[test]
    fn move_node_rejects_moving_into_itself() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (insert_act, act) = Command::insert_node(NodeKind::Act, doc.root_id(), SortKey::initial());
        bus.apply(&mut doc, vec![insert_act], t(0)).unwrap();

        let err = bus
            .apply(&mut doc, vec![Command::move_node(act, act, SortKey::initial())], t(0))
            .unwrap_err();
        assert!(matches!(err, CoreError::CyclicMove(id) if id == act));
    }

    #[test]
    fn move_node_rejects_moving_root() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();
        let (insert_act, act) = Command::insert_node(NodeKind::Act, root_id, SortKey::initial());
        bus.apply(&mut doc, vec![insert_act], t(0)).unwrap();

        let err = bus
            .apply(&mut doc, vec![Command::move_node(root_id, act, SortKey::initial())], t(0))
            .unwrap_err();
        assert_eq!(err, CoreError::CannotMoveRoot);
    }

    #[test]
    fn move_block_reparents_across_scenes_and_undo_restores() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();
        let layer_id: LayerId = ElementId::new();

        let (insert_scene1, scene1) = Command::insert_node(NodeKind::Scene, root_id, SortKey::initial());
        let (insert_scene2, scene2) = Command::insert_node(NodeKind::Scene, root_id, SortKey::initial().after());
        bus.apply(&mut doc, vec![insert_scene1, insert_scene2], t(0)).unwrap();

        let (insert_block, block_id) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Una battuta."),
            layer_id,
            scene1,
            SortKey::initial(),
        );
        bus.apply(&mut doc, vec![insert_block], t(0)).unwrap();

        bus.apply(&mut doc, vec![Command::move_block(block_id, scene2, SortKey::initial())], t(0))
            .unwrap();
        assert_eq!(doc.block(block_id).unwrap().parent(), Some(scene2));

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.block(block_id).unwrap().parent(), Some(scene1));
    }

    // --- Rimozione a cascata ---

    #[test]
    fn remove_node_cascades_children_and_annotations_and_undo_restores_everything() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();
        let layer_id: LayerId = ElementId::new();

        let (insert_act, act) = Command::insert_node(NodeKind::Act, root_id, SortKey::initial());
        bus.apply(&mut doc, vec![insert_act], t(0)).unwrap();
        let (insert_scene, scene) = Command::insert_node(NodeKind::Scene, act, SortKey::initial());
        bus.apply(&mut doc, vec![insert_scene], t(0)).unwrap();
        let (insert_block, block_id) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Battuta nell'atto da rimuovere."),
            layer_id,
            scene,
            SortKey::initial(),
        );
        bus.apply(&mut doc, vec![insert_block], t(0)).unwrap();

        // Un'annotazione sul blocco e una sull'atto stesso: entrambe devono
        // sparire con la rimozione a cascata.
        let annotation_on_block = Annotation::new(
            layer_id,
            AnchorTarget::Element(block_id),
            "Nota sul blocco",
            t(0),
        );
        let annotation_on_act = Annotation::new(layer_id, AnchorTarget::Element(act), "Nota sull'atto", t(0));
        let ann_block_id = doc.insert_annotation(annotation_on_block);
        let ann_act_id = doc.insert_annotation(annotation_on_act);

        let delta = bus.apply(&mut doc, vec![Command::remove_node(act)], t(0)).unwrap();

        assert!(doc.node(act).is_none());
        assert!(doc.node(scene).is_none());
        assert!(doc.block(block_id).is_none());
        assert!(doc.annotation(ann_block_id).is_none());
        assert!(doc.annotation(ann_act_id).is_none());

        // Il delta deve riportare tutto quello che è sparito, non solo l'atto.
        let mut removed = delta.removed.clone();
        removed.sort();
        let mut expected = vec![act, scene, block_id, ann_block_id, ann_act_id];
        expected.sort();
        assert_eq!(removed, expected);

        bus.undo(&mut doc).unwrap();
        assert!(doc.node(act).is_some());
        assert!(doc.node(scene).is_some());
        assert_eq!(doc.block(block_id).unwrap().as_text().unwrap().text, "Battuta nell'atto da rimuovere.");
        assert!(doc.annotation(ann_block_id).is_some());
        assert!(doc.annotation(ann_act_id).is_some());
    }

    #[test]
    fn remove_node_rejects_removing_root() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let root_id = doc.root_id();
        let err = bus.apply(&mut doc, vec![Command::remove_node(root_id)], t(0)).unwrap_err();
        assert_eq!(err, CoreError::CannotRemoveRoot);
    }

    #[test]
    fn remove_block_cascades_annotations_and_undo_restores_them() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let layer_id: LayerId = ElementId::new();

        let (insert_block, block_id) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Battuta annotata."),
            layer_id,
            doc.root_id(),
            SortKey::initial(),
        );
        bus.apply(&mut doc, vec![insert_block], t(0)).unwrap();

        let ann_id = doc.insert_annotation(Annotation::new(
            layer_id,
            AnchorTarget::Element(block_id),
            "Nota",
            t(0),
        ));

        bus.apply(&mut doc, vec![Command::remove_block(block_id)], t(0)).unwrap();
        assert!(doc.block(block_id).is_none());
        assert!(doc.annotation(ann_id).is_none());

        bus.undo(&mut doc).unwrap();
        assert!(doc.block(block_id).is_some());
        assert!(doc.annotation(ann_id).is_some());
    }

    // --- ReanchorAnnotation / SetLayerOrder ---

    #[test]
    fn reanchor_annotation_changes_anchor_and_undo_restores_it() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let layer_id: LayerId = ElementId::new();
        let (insert_a, block_a) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Blocco A"),
            layer_id,
            doc.root_id(),
            SortKey::initial(),
        );
        let (insert_b, block_b) = Command::insert_block(
            TextBlock::new(BlockKind::Dialogue, "Blocco B"),
            layer_id,
            doc.root_id(),
            SortKey::initial().after(),
        );
        bus.apply(&mut doc, vec![insert_a, insert_b], t(0)).unwrap();

        let ann_id = doc.insert_annotation(Annotation::new(layer_id, AnchorTarget::Element(block_a), "Nota", t(0)));

        bus.apply(
            &mut doc,
            vec![Command::reanchor_annotation(ann_id, AnchorTarget::Element(block_b))],
            t(0),
        )
            .unwrap();
        assert_eq!(doc.annotation(ann_id).unwrap().anchored_element(), block_b);

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.annotation(ann_id).unwrap().anchored_element(), block_a);
    }

    #[test]
    fn set_layer_order_undo_restores_previous_order() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (insert, id) = Command::insert_layer("Base", Color::rgb(0, 0, 0), SortKey::initial());
        bus.apply(&mut doc, vec![insert], t(0)).unwrap();
        let original_order = doc.layer(id).unwrap().order.clone();

        bus.apply(&mut doc, vec![Command::set_layer_order(id, original_order.after())], t(0))
            .unwrap();
        assert_ne!(doc.layer(id).unwrap().order, original_order);

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.layer(id).unwrap().order, original_order);
    }

    // --- Note del personaggio ---

    #[test]
    fn character_note_commands_undo_restores_previous_notes() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        let (insert, id) = Command::insert_character("Romeo", alias("ROM"));
        bus.apply(&mut doc, vec![insert], t(0)).unwrap();

        bus.apply(
            &mut doc,
            vec![
                Command::set_character_costume_notes(id, Some("Farsetto blu".to_string())),
                Command::set_character_entrance_note(id, Some("Da sinistra".to_string())),
                Command::set_character_exit_note(id, Some("A destra, di corsa".to_string())),
            ],
            t(0),
        )
            .unwrap();

        let character = doc.character(id).unwrap();
        assert_eq!(character.costume_notes.as_deref(), Some("Farsetto blu"));
        assert_eq!(character.entrance_notes.as_deref(), Some("Da sinistra"));
        assert_eq!(character.exit_notes.as_deref(), Some("A destra, di corsa"));

        bus.undo(&mut doc).unwrap();
        let character = doc.character(id).unwrap();
        assert!(character.costume_notes.is_none());
        assert!(character.entrance_notes.is_none());
        assert!(character.exit_notes.is_none());
    }

    // --- DocMeta ---

    #[test]
    fn doc_meta_commands_set_meta_changed_flag_and_undo_restores_values() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();

        let delta = bus
            .apply(
                &mut doc,
                vec![
                    Command::set_doc_title(Some("Romeo e Giulietta".to_string())),
                    Command::set_doc_synopsis(Some("Due amanti a Verona.".to_string())),
                ],
                t(0),
            )
            .unwrap();

        assert!(delta.meta_changed);
        assert!(delta.created.is_empty() && delta.updated.is_empty() && delta.removed.is_empty());
        assert_eq!(doc.metadata.title.as_deref(), Some("Romeo e Giulietta"));

        bus.undo(&mut doc).unwrap();
        assert!(doc.metadata.title.is_none());
        assert!(doc.metadata.synopsis.is_none());
    }

    #[test]
    fn doc_meta_grouped_commands_replace_whole_section_and_undo_restores() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();

        bus.apply(
            &mut doc,
            vec![Command::set_doc_authorship(Authorship {
                authors: vec!["Autore Uno".to_string(), "Autore Due".to_string()],
                translator: None,
                based_on: None,
            })],
            t(0),
        )
            .unwrap();
        assert_eq!(doc.metadata.authorship.authors.len(), 2);

        bus.apply(
            &mut doc,
            vec![Command::set_doc_production(Production {
                director: Some("Regista Tal Dei Tali".to_string()),
                commissioned_by: None,
                premiere: None,
            })],
            t(0),
        )
            .unwrap();
        assert_eq!(doc.metadata.production.director.as_deref(), Some("Regista Tal Dei Tali"));

        bus.apply(
            &mut doc,
            vec![Command::set_doc_rights(Some(Copyright {
                holder: "Autore Uno".to_string(),
                year: 2026,
                statement: None,
            }))],
            t(0),
        )
            .unwrap();
        assert_eq!(doc.metadata.copyright.as_ref().unwrap().year, 2026);

        bus.apply(
            &mut doc,
            vec![Command::set_doc_status(ProjectStatus::Final, Some("Non distribuire".to_string()))],
            t(0),
        )
            .unwrap();
        assert_eq!(doc.metadata.status, ProjectStatus::Final);

        // Tre undo, uno per ciascuno degli ultimi tre batch: production, rights, status.
        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.metadata.status, ProjectStatus::Draft);
        bus.undo(&mut doc).unwrap();
        assert!(doc.metadata.copyright.is_none());
        bus.undo(&mut doc).unwrap();
        assert!(doc.metadata.production.director.is_none());
        // L'authorship del primo batch è ancora lì: non l'abbiamo disfatta.
        assert_eq!(doc.metadata.authorship.authors.len(), 2);
    }

    #[test]
    fn set_doc_version_major_resets_sub_and_undo_restores_exact_pair() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();

        // Portiamo sub a un valore non-zero fuori dal bus, come farebbe
        // l'app a un salvataggio, poi alziamo il major dal bus.
        doc.bump_version_sub();
        doc.bump_version_sub();
        assert_eq!(doc.metadata.version, Version::new(0, 2));

        bus.apply(&mut doc, vec![Command::set_doc_version_major(2)], t(0)).unwrap();
        assert_eq!(doc.metadata.version, Version::new(2, 0));

        bus.undo(&mut doc).unwrap();
        // L'undo deve ripristinare la coppia esatta di prima, non solo major.
        assert_eq!(doc.metadata.version, Version::new(0, 2));
    }

    #[test]
    fn set_doc_version_sub_manual_override_is_undoable() {
        let mut doc = Document::new(t(0));
        let mut bus = CommandBus::new();
        bus.apply(&mut doc, vec![Command::set_doc_version_sub(7)], t(0)).unwrap();
        assert_eq!(doc.metadata.version, Version::new(0, 7));

        bus.undo(&mut doc).unwrap();
        assert_eq!(doc.metadata.version, Version::new(0, 0));
    }

    #[test]
    fn bump_version_sub_bypasses_bus_entirely() {
        let mut doc = Document::new(t(0));
        let bus = CommandBus::new();
        let r0 = doc.revision();

        // Chiamata diretta, non tramite CommandBus: nessuna entry di
        // history, nessun avanzamento di Revision (che riguarda solo il
        // contenuto, non il bookkeeping di salvataggio).
        doc.bump_version_sub();

        assert_eq!(doc.metadata.version, Version::new(0, 1));
        assert_eq!(doc.revision(), r0);
        assert!(!bus.can_undo());
    }
}