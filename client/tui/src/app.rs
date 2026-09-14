use client_core::{App, ApplicationEvent, Assets, ClientError};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use tokio::signal::unix::{Signal, SignalKind, signal};

type TerminalScreen = Terminal<CrosstermBackend<io::Stdout>>;

pub struct TuiApp {
    app: App,
    terminal: TerminalScreen,
    device_events: EventStream,
    shutdown_signal: Signal,
}

impl TuiApp {
    pub fn new(ip: String, port: String, assets: Assets) -> Result<Self, ClientError> {
        let shutdown_signal =
            signal(SignalKind::terminate()).map_err(|err| ClientError::General(err.to_string()))?;

        Ok(Self {
            app: App::new(ip, port, assets),
            terminal: terminal_setup()?,
            device_events: EventStream::new(),
            shutdown_signal,
        })
    }

    pub async fn run(&mut self) -> Result<(), ClientError> {
        self.render()?;

        while !self.app.state.should_quit {
            self.next_events().await?;
            self.render()?;
        }

        Ok(())
    }

    pub fn restore(mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()
    }

    async fn next_events(&mut self) -> Result<(), ClientError> {
        tokio::select! {
            _shutdown_signal = self.shutdown_signal.recv() => {
                self.app.state.should_quit = true;
            }
            event = self.app.event_broker.next_event() => self.app.update(event?),
            Some(Ok(device_event)) = self.device_events.next() => {
                self.app.update(ApplicationEvent::DeviceEvent(device_event));
            }
        }

        while !self.app.state.should_quit {
            match self.app.try_next_event() {
                Ok(event) => self.app.update(event),
                Err(ClientError::EventChannelEmpty) => break,
                Err(error) => {
                    self.app.state.should_quit = true;
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<(), ClientError> {
        if self.app.state.should_quit {
            return Ok(());
        }

        self.terminal.draw(|frame| self.app.draw(frame))?;

        Ok(())
    }
}

fn terminal_setup() -> io::Result<TerminalScreen> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    Terminal::new(CrosstermBackend::new(stdout))
}
