use crate::events::AppEvent;
use crate::state::{AppState, ChatScope};
use std::sync::Arc;
use tokio::sync::mpsc;

pub fn handle_command(state: &mut AppState, cmd_line: String, tx: mpsc::UnboundedSender<AppEvent>) {
    let parts: Vec<&str> = cmd_line.trim().split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    state.game.push_game_output(format!("> {}", cmd_line));

    if cmd == "quit" {
        state.should_quit = true;
        return;
    }

    if let Some(client_arc) = &state.net.client {
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
                            let _ = tx_clone.send(AppEvent::CommandError(e));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(AppEvent::TapError(e));
                        }
                    }
                };
            }

            match cmd.as_str() {
                "look" => match c.look().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::UpdateRoomContext {
                            room_id: data.room.id.clone(),
                            room_display_name: data.room.name.clone(),
                            npcs: data.npcs.clone(),
                        });
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "who" => match c.who().await {
                    Ok(Ok(data)) => {
                        let count = data.player_count as u32;
                        let _ = tx_clone.send(AppEvent::UpdateOnlinePlayers(count));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "chat_global" => match c.chat_global(args.join(" ")).await {
                    Ok(Ok(_res)) => {
                        let msg = args.join(" ");
                        let _ = tx_clone.send(AppEvent::LocalChatSent(ChatScope::Global, msg));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "chat_private" if args.len() >= 2 => {
                    let to = args[0].clone();
                    let msg = args[1..].join(" ");
                    match c.chat_private(to.clone(), msg.clone()).await {
                        Ok(Ok(_res)) => {
                            let _ = tx_clone.send(AppEvent::LocalChatSent(
                                ChatScope::Private,
                                format!("to {}: {}", to, msg),
                            ));
                        }
                        Ok(Err(e)) => {
                            let _ = tx_clone.send(AppEvent::CommandError(e));
                        }
                        Err(e) => {
                            let _ = tx_clone.send(AppEvent::TapError(e));
                        }
                    }
                }
                "group_create" => match c.group_create().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::UpdateGroup(Some(data.group_id.clone())));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "group_invite" if args.len() == 1 => {
                    handle_res!(c.group_invite(args[0].clone()).await)
                }
                "group_join" if args.len() == 1 => match c.group_join(args[0].clone()).await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::UpdateGroup(Some(data.group_id.clone())));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "group_leave" => match c.group_leave().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::UpdateGroup(None));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "take" if args.len() == 1 => handle_res!(c.take(args[0].clone()).await),
                "drop" if args.len() == 1 => handle_res!(c.drop_item(args[0].clone()).await),
                "move" if args.len() == 1 => {
                    handle_res!(c.r#move(args[0].clone().to_uppercase()).await)
                }
                "inventory" => match c.inventory().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::InventoryUpdate(data.inventory.clone()));
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "talk" if !args.is_empty() => {
                    handle_res!(c.talk(args.join(" ")).await)
                }
                "attack" if !args.is_empty() => {
                    handle_res!(c.attack(args.join(" ")).await)
                }
                "status" => match c.status().await {
                    Ok(Ok(data)) => {
                        let _ = tx_clone.send(AppEvent::UpdateStatus {
                            hp: data.player_status.hp,
                            max_hp: data.player_status.max_hp,
                        });
                        let _ = tx_clone.send(AppEvent::CommandResult(format!("{:#?}", data)));
                    }
                    Ok(Err(e)) => {
                        let _ = tx_clone.send(AppEvent::CommandError(e));
                    }
                    Err(e) => {
                        let _ = tx_clone.send(AppEvent::TapError(e));
                    }
                },
                "quest" if !args.is_empty() => {
                    handle_res!(c.quest(args.join(" ")).await)
                }
                "quests" => handle_res!(c.quests().await),
                _ => {
                    let _ = tx_clone.send(AppEvent::UnknowCommand(cmd));
                }
            }
        });
    } else {
        state.game.push_game_output("Not connected.".to_string());
    }
}
