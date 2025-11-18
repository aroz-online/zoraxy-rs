mod configuration;
mod events;
mod introspection;

pub use configuration::*;
pub use events::*;
pub use introspection::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[repr(u16)]
pub enum ControlStatusCode {
    /// Traffic captured by plugin, ask Zoraxy not to process the traffic
    Captured = 280,
    /// Traffic not handled by plugin, ask Zoraxy to process the traffic
    UnHandled = 284,
    /// Error occurred while processing the traffic, ask Zoraxy to process the traffic and log the error
    Error = 580,
}
