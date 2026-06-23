use crate::app::App;
use crate::events::{AppEvent, GameEvent, UiEvent};
use crate::state::NotificationType;
use tokio::sync::mpsc;

pub async fn handle(event: GameEvent, _app: &mut App, tx: &mpsc::Sender<AppEvent>) {
    match event {
        GameEvent::CommandError(err) => {
            let message = match err.code {
                Some(code) => format!("[{}] Command error: {}", code, err.message),
                None => format!("Command error: {}", err.message),
            };
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                message,
                NotificationType::Error,
                16,
            ))).await;
        }

        GameEvent::UnknowCommand(err) => {
            let _ = tx.send(AppEvent::Ui(UiEvent::Notification(
                format!("Unknow command: {}", err),
                NotificationType::Error,
                16,
            ))).await;
        }

        _ => {}
    }
}