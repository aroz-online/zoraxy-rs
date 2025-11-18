use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum EventName {
    /// emitted when a blacklisted IP is blocked
    BlacklistedIpBlocked,
    /// emitted when the blacklist is toggled for an access rule
    BlacklistToggled,
    /// emitted when a new access ruleset is created
    AccessRuleCreated,
    /// A custom event emitted by a plugin, with the intention of being broadcast
    /// to the designated recipient(s)
    CustomEvent,
    // Add more events as needed
}

impl std::fmt::Display for EventName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_str = match self {
            Self::BlacklistedIpBlocked => "blacklistedIpBlocked",
            Self::BlacklistToggled => "blacklistToggled",
            Self::AccessRuleCreated => "accessRuleCreated",
            Self::CustomEvent => "customEvent",
        };
        write!(f, "{event_str}")
    }
}

/// `BlacklistedIPBlockedEvent` represents an event when a blacklisted IP is blocked
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BlacklistedIPBlockedEvent {
    pub ip: String,
    pub comment: String,
    pub requested_url: String,
    pub hostname: String,
    pub user_agent: String,
    pub method: String,
}

/// `BlacklistToggledEvent` represents an event when the blacklist is disabled for an access rule
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct BlacklistToggledEvent {
    pub rule_id: String,
    pub enabled: bool, // Whether the blacklist is enabled or disabled
}

/// `AccessRuleCreatedEvent` represents an event when a new access ruleset is created
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct AccessRuleCreatedEvent {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub blacklist_enabled: bool,
    pub whitelist_enabled: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CustomEvent {
    pub source_plugin: String,
    pub recipients: Vec<String>,
    pub payload: HashMap<String, serde_json::Value>,
}

/// `EventPayload` enum for all event payloads
#[derive(Debug, Clone)]
pub enum EventPayload {
    BlacklistedIPBlocked(BlacklistedIPBlockedEvent),
    BlacklistToggled(BlacklistToggledEvent),
    AccessRuleCreated(AccessRuleCreatedEvent),
    Custom(CustomEvent),
}

impl EventPayload {
    #[must_use]
    pub const fn get_name(&self) -> EventName {
        match self {
            Self::BlacklistedIPBlocked(_) => EventName::BlacklistedIpBlocked,
            Self::BlacklistToggled(_) => EventName::BlacklistToggled,
            Self::AccessRuleCreated(_) => EventName::AccessRuleCreated,
            Self::Custom(_) => EventName::CustomEvent,
        }
    }

    #[must_use]
    pub fn get_event_source(&self) -> String {
        match self {
            Self::BlacklistedIPBlocked(_) => "proxy-access".to_string(),
            Self::BlacklistToggled(_) | Self::AccessRuleCreated(_) => "accesslist-api".to_string(),
            Self::Custom(e) => e.source_plugin.clone(),
        }
    }

    /// Deserialize `EventPayload` from JSON value based on `EventName`
    ///
    /// # Errors
    /// If deserialization fails, returns `serde_json::Error`
    pub fn from_json(
        value: serde_json::Value,
        event_name: &EventName,
    ) -> Result<Self, serde_json::Error> {
        match event_name {
            EventName::BlacklistedIpBlocked => {
                let data = serde_json::from_value(value)?;
                Ok(Self::BlacklistedIPBlocked(data))
            }
            EventName::BlacklistToggled => {
                let data = serde_json::from_value(value)?;
                Ok(Self::BlacklistToggled(data))
            }
            EventName::AccessRuleCreated => {
                let data = serde_json::from_value(value)?;
                Ok(Self::AccessRuleCreated(data))
            }
            EventName::CustomEvent => {
                let data = serde_json::from_value(value)?;
                Ok(Self::Custom(data))
            }
        }
    }
}

/// Event represents a system event
#[derive(Debug, Clone)]
pub struct Event {
    pub name: EventName,
    pub timestamp: i64, // Unix timestamp
    pub uuid: String,   // UUID for the event
    pub data: EventPayload,
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct EventFields {
            name: EventName,
            timestamp: i64,
            uuid: String,
            data: serde_json::Value,
        }

        let fields = EventFields::deserialize(deserializer)?;
        let payload =
            EventPayload::from_json(fields.data, &fields.name).map_err(serde::de::Error::custom)?;
        Ok(Self {
            name: fields.name,
            timestamp: fields.timestamp,
            uuid: fields.uuid,
            data: payload,
        })
    }
}
