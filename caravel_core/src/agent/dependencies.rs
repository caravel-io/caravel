use crate::agent::events::{Event, EventError, EventType};

pub async fn receive_dependencies() -> Result<Event, EventError> {
    // Somehow this should be a tarball stream that gets untarred and installed
    // into the correct location to be loaded by caravel
    let response = Event::new(EventType::Assembled).message("received dependencies".to_string());
    return Ok(response);
}
