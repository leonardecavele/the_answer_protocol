use crate::error::CommandError;
use crate::protocol::command::Command;
use crate::protocol::response::ServerResponse;

#[derive(Debug, Clone)]
pub struct DropCommand {
    pub item_identifier: String,
}

#[derive(Debug, Clone)]
pub struct DropResponse {
    pub item_identifier: String,
}

impl Command for DropCommand {
    type ResponseData = DropResponse;

    fn encode(&self) -> String {
        format!("DROP {}", self.item_identifier)
    }

    fn parse_response(&self, response: ServerResponse) -> Result<Self::ResponseData, CommandError> {
        if response.arguments.len() != 1 {
            return Err(CommandError {
                code: None,
                message: "invalid arguments".to_string(),
            });
        }

        let item_identifier = match response.arguments[0].strip_prefix("dropped=") {
            Some(id) => id.to_string(),
            None => {
                return Err(CommandError {
                    code: None,
                    message: "invalid arguments: missing 'dropped=' prefix".to_string(),
                });
            }
        };

        Ok(DropResponse { item_identifier })
    }

    fn refine_error(&self, error: &mut CommandError) {
        error.with_message(match error.code {
            Some(404) => Some("item not in inventory".to_string()),
            _ => None,
        })
    }

    fn from_str(args: &str) -> Option<Self> {
        if args.trim().is_empty() {
            return None;
        }
        Some(Self {
            item_identifier: args.trim().to_string(),
        })
    }
}
