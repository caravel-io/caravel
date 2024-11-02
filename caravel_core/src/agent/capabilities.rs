use crate::agent::events::{Event, EventError, EventType};
use crate::agent::util::generate_token;

pub async fn gather_capabilities() -> Result<Event, EventError> {
    let et = EventType::Capabilities {
        token: generate_token(),
    };
    let response = Event::new(et).message("sample capabilities response".to_string());
    return Ok(response);
}
