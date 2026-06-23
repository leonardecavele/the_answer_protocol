use crate::components::game::GameComponent;
use crate::components::login::LoginComponent;
use crate::components::Component;
use crate::events::{AppEvent, UiEvent};
use crate::state::AppState;
use crate::{events, MAX_EVENTS_BUS, TICK_RATE};
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tokio::sync::mpsc;

pub struct App {
    pub state: AppState,
    pub active_component: Box<dyn Component>,
}

impl App {
    pub fn new(ip: String, port: String) -> Self {
        Self {
            state: AppState::new(ip.clone(), port.clone()),
            active_component: Box::new(LoginComponent::new(ip, port)),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {

        // event_bus = EventBus::new();
        // thread to forward (tick + terminal event) to event_bus
        // main loop (draw + listen event_bus)




        let (app_event_sender, mut app_event_receiver) = mpsc::channel::<AppEvent>(MAX_EVENTS_BUS);

        let app_event_sender_clone = app_event_sender.clone();
        let event_task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(TICK_RATE);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        if app_event_sender_clone.send(AppEvent::Ui(UiEvent::Tick)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(evt)) = reader.next() => {
                        if app_event_sender_clone.send(AppEvent::Ui(UiEvent::TerminalEvent(evt))).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Main loop
        while !self.state.should_quit {
            terminal.draw(|f| {
                self.active_component.draw(&mut self.state, f, f.area());
            })?;

            if let Some(event) = app_event_receiver.recv().await {
                events::router::route(event, self, &app_event_sender).await;
            }
        }

        event_task.abort();

        Ok(())
    }
}

impl App {
    pub fn switch_to_game(&mut self) {
        self.active_component = Box::new(GameComponent::new());
    }

    pub fn switch_to_login(&mut self) {
        self.active_component = Box::new(LoginComponent::new(
            self.state.net.server_ip.clone(),
            self.state.net.server_port.clone(),
        ));
    }
}
