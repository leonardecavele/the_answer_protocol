use crate::app::{App, ChatScope, GameFocus, ACTIONS};
use crate::events::AppEvent;
use std::sync::Arc;
use tokio::sync::mpsc;
use tui_input::Input;

pub async fn execute_action(app: &mut App, tx: &mpsc::UnboundedSender<AppEvent>) {
    let action = ACTIONS[app.selected_action];
    match action {
        "WHO" => {
            handle_command(app, "who".to_string(), tx.clone()).await;
        }
        "LOOK" => {
            handle_command(app, "look".to_string(), tx.clone()).await;
        }
        "ATTACK" => {
            app.input = Input::from("attack ".to_string());
            app.game_focus = GameFocus::Input;
        }
        "QUEST" => {
            app.input = Input::from("quest ".to_string());
            app.game_focus = GameFocus::Input;
        }
        "QUIT" => {
            app.should_quit = true;
        }
        _ => {}
    }
}

pub async fn handle_command(app: &mut App, cmd_line: String, tx: mpsc::UnboundedSender<AppEvent>) {
    let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    app.push_game_output(format!("> {}", cmd_line));

    if cmd == "quit" {
        app.should_quit = true;
        return;
    }

    if let Some(client_arc) = &app.client {
        let client_arc = Arc::clone(client_arc);
        let cmd = cmd.to_string();
        let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let mut c = client_arc.lock().await;

            macro_rules! handle_res {
                ($res:expr) => {
                    match $res {
                        Ok(Ok(data)) => {
                            let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                        }
                        Ok(Err(e)) => {
                            let _ = tx_clone
                                .send(AppEvent::CommandError(format!("Command Error: {:?}", e)));
                        }
                        Err(e) => {
                            let _ = tx_clone
                                .send(AppEvent::CommandError(format!("Network Error: {:?}", e)));
                        }
                    }
                };
            }

            match cmd.as_str() {
                "look" => handle_res!(c.look().await),
                "who" => match c.who().await {
                    Ok(Ok(data)) => {
                        let count = data.player_count as u32;
                        let _ = tx_clone.send(AppEvent::UpdateOnlinePlayers(count));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(format!("Command Error: {:?}", e)));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::CommandError(format!("Network Error: {:?}", e)));
                    }
                },
                "chat_global" => match c.chat_global(args.join(" ")).await {
                    Ok(Ok(_res)) => {
                        let msg = args.join(" ");
                        let _ = tx_clone.send(AppEvent::LocalChatSent(ChatScope::Global, msg));
                        // let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", res)));
                    }
                    Ok(Err(e)) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Command Error: {:?}", e))); }
                    Err(e) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Network Error: {:?}", e))); }
                },
                "chat_private" if args.len() >= 2 => {
                    let to = args[0].clone();
                    let msg = args[1..].join(" ");
                    match c.chat_private(to.clone(), msg.clone()).await {
                        Ok(Ok(_res)) => {
                            let _ = tx_clone.send(AppEvent::LocalChatSent(ChatScope::Private, format!("to {}: {}", to, msg)));
                            // let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", res)));
                        }
                        Ok(Err(e)) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Command Error: {:?}", e))); }
                        Err(e) => { let _ = tx_clone.send(AppEvent::CommandError(format!("Network Error: {:?}", e))); }
                    }
                }
                "group_create" => handle_res!(c.group_create().await),
                "group_invite" if args.len() == 1 => {
                    handle_res!(c.group_invite(args[0].clone()).await)
                }
                "group_join" if args.len() == 1 => {
                    handle_res!(c.group_join(args[0].clone()).await)
                }
                "group_leave" => handle_res!(c.group_leave().await),
                "take" if args.len() == 1 => handle_res!(c.take(args[0].clone()).await),
                "drop" if args.len() == 1 => handle_res!(c.drop_item(args[0].clone()).await),
                "inventory" => match c.inventory().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::InventoryUpdate(data.inventory.clone()));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone
                            .send(AppEvent::CommandError(format!("Command Error: {:?}", e)));
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppEvent::CommandError(format!("Network Error: {:?}", e)));
                    }
                },
                "talk" if !args.is_empty() => {
                    handle_res!(c.talk(args.join(" ")).await)
                }
                "attack" if !args.is_empty() => {
                    handle_res!(c.attack(args.join(" ")).await)
                }
                "status" => handle_res!(c.status().await),
                "quest" if !args.is_empty() => {
                    handle_res!(c.quest(args.join(" ")).await)
                }
                "quests" => handle_res!(c.quests().await),
                _ => {
                    let _ =
                        tx_clone.send(AppEvent::CommandError(format!("Unknown command: {}", cmd)));
                }
            }
        });
    } else {
        app.push_game_output("Not connected.".to_string());
    }
}
