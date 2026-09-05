use crate::app::runtime::App;
use crate::states::game::{DialogueState, END_OF_DIALOGUE_TAG};
use client_api::commands::TalkResponse;

impl App {
    pub fn on_talked_to(&mut self, response: TalkResponse, npc_id: String) {
        let mut text = response.dialogue;
        let ends_dialog = text.contains(END_OF_DIALOGUE_TAG);
        let ends_dialog_only = ends_dialog && text.starts_with(END_OF_DIALOGUE_TAG);
        let was_open = self.state.game.overlays.is_open::<DialogueState>();

        if ends_dialog_only {
            if was_open {
                self.state.game.close_dialogue();
                return;
            }

            text = "**nothing**".to_string();
        } else if ends_dialog {
            text = text.replace(END_OF_DIALOGUE_TAG, "").trim().to_string();
        }

        let npc_name = self.state.game.manifest.npc_name(&npc_id);
        self.state.game.inspected_npc = Some(npc_id.clone());

        if !was_open {
            self.state
                .game
                .log_action(format!("You talked to {}.", npc_name));
        }

        if !ends_dialog_only {
            self.state
                .game
                .log_action(format!("[{}] says: \"{}\"", npc_name, text));
        }

        if let Some(dialogue) = self.state.game.overlays.get_mut::<DialogueState>() {
            dialogue.add(text, ends_dialog);
        } else {
            self.state
                .game
                .open_dialogue(DialogueState::new(npc_id, npc_name, text, ends_dialog));
        }
    }
}
