use crate::game_manager::GameManager;

impl GameManager {
    pub fn send_diff_to_players(&mut self) -> std::io::Result<()> {
        match self.send_msg_to_client(self.get_tick_diff().dump())
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}
