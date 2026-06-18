use crate::state::{AppState, ChatScope};

pub fn handle_server_event(state: &mut AppState, evt: api_client::client::event::ServerEvent) {
    use api_client::client::event::*;
    match evt {
        ServerEvent::Connect(name) => {
            state
                .game
                .push_game_output(format!("-> {} connected.", name));
            state.game.online_players += 1;
            state.ui.push_notification(
                format!("{} connected", name),
                crate::state::NotificationType::Info,
                16,
            );
        }
        ServerEvent::Quit(name) => {
            state.game.push_game_output(format!("<- {} quit.", name));
            state.game.online_players = state.game.online_players.saturating_sub(1);
            state.ui.push_notification(
                format!("{} disconnected", name),
                crate::state::NotificationType::Info,
                16,
            );
        }
        ServerEvent::Room(RoomEvent::PresenceEnter(name)) => {
            state
                .game
                .push_game_output(format!("[Room] {} entered.", name));
        }
        ServerEvent::Room(RoomEvent::PresenceLeave(name)) => {
            state
                .game
                .push_game_output(format!("[Room] {} left.", name));
        }
        ServerEvent::Room(RoomEvent::Chat(msg)) => {
            state
                .game
                .push_chat(ChatScope::Room, msg.sender, msg.message);
        }
        ServerEvent::Group(GroupEvent::Chat(msg)) => {
            state
                .game
                .push_chat(ChatScope::Group, msg.sender, msg.message);
        }
        ServerEvent::Group(GroupEvent::Invite(name)) => {
            state
                .game
                .push_game_output(format!("[Group] {} invited you.", name));
            state.ui.push_notification(
                format!("{} invited you to group", name),
                crate::state::NotificationType::Info,
                20,
            );
        }
        ServerEvent::Group(GroupEvent::Join(name)) => {
            state
                .game
                .push_game_output(format!("[Group] {} joined.", name));
        }
        ServerEvent::Group(GroupEvent::Leave(name)) => {
            state
                .game
                .push_game_output(format!("[Group] {} left.", name));
        }
        ServerEvent::GlobalChat(msg) => {
            state
                .game
                .push_chat(ChatScope::Global, msg.sender, msg.message);
        }
        ServerEvent::PrivateChat(msg) => {
            state
                .game
                .push_chat(ChatScope::Private, msg.sender, msg.message);
        }
        ServerEvent::Stats(count) => {
            state.game.online_players = count;
        }
        ServerEvent::Unknown(u) => {
            state.game.push_game_output(format!("Unknown event: {}", u));
        }
    }
}
