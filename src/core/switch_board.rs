//! src/core/switch_board.rs

use serde::{ Serialize, Deserialize };

// Package Files & Modules
use crate::common::LayerChannel;

/// Quadro elettrico centrale del copione. Gestisce l'abilitazione binaria
/// globale dei canali di visualizzazione. Vive nello Stack e implementa Copy.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub struct SwitchBoard {
    pub channel_mask: [bool; 4],
}

impl SwitchBoard {
    /// Costruttore standard: di default tutti i canali sono attivi (All ON)
    pub fn new() -> Self {
        Self {
            channel_mask: [true, true, true, true],
        }
    }
    /// Attiva un intero canale logico a livello globale
    pub fn enable_channel(&mut self, channel: LayerChannel) {
        self.channel_mask[channel.to_index()] = true;
    }
    /// Disattiva un intero canale logico a livello globale
    pub fn disable_channel(&mut self, channel: LayerChannel) {
        self.channel_mask[channel.to_index()] = false;
    }
    /// Interroga lo stato globale di un canale
    pub fn is_channel_enabled(&self, channel: LayerChannel) -> bool {
        self.channel_mask[channel.to_index()]
    }
    /// Ripristina l'ambiente riaccendendo tutti i canali con un click
    pub fn reset(&mut self) {
        self.channel_mask = [true, true, true, true];
    }
}

impl Default for SwitchBoard {
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

    #[test]
    fn test_switchboard_toggle_and_copy() {
        let mut board = SwitchBoard::new();

        // Di default tutto acceso
        assert!(board.is_channel_enabled(LayerChannel::ScriptText));
        assert!(board.is_channel_enabled(LayerChannel::DirectorNotes));

        // Spegnimento selettivo (anche del testo principale, come da brainstorming)
        board.disable_channel(LayerChannel::ScriptText);
        board.disable_channel(LayerChannel::DirectorNotes);

        assert!(!board.is_channel_enabled(LayerChannel::ScriptText));
        assert!(!board.is_channel_enabled(LayerChannel::DirectorNotes));

        // Verifica tratto Copy puro
        let mut board_copia = board;
        board_copia.enable_channel(LayerChannel::ScriptText);

        // La copia è modificata, l'originale resta spento nello Stack
        assert!(board_copia.is_channel_enabled(LayerChannel::ScriptText));
        assert!(!board.is_channel_enabled(LayerChannel::ScriptText));
    }
}