use crate::components::Component;
use crate::components::game::GameComponent;
use crate::components::login::LoginComponent;
use crate::state::AppState;

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
