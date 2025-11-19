use std::collections::HashMap;

use crate::EventName;

/// `IntroSpect` Payload
///
/// When the plugin is initialized with -introspect flag,
/// the plugin shell returns this payload as JSON and exits.
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct IntroSpect {
    /// Plugin metadata
    #[serde(flatten)]
    metadata: PluginMetadata,
    /// Static Capture Settings
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    static_capture_settings: Option<StaticCaptureSettings>,
    /// Dynamic Capture Settings
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    dynamic_capture_settings: Option<DynamicCaptureSettings>,
    /// UI Path for your plugin
    /// e.g. /ui
    /// Will proxy the whole subpath tree to Zoraxy Web UI as plugin UI
    #[serde(skip_serializing_if = "Option::is_none")]
    ui_path: Option<String>,
    /// Subscriptions Settings
    #[serde(flatten)]
    subscriptions: Option<SubscriptionsSettings>,

    /// API Access Control
    /// List of API endpoints this plugin can access,
    /// and a description of why the plugin needs to access this endpoint
    #[serde(skip_serializing_if = "Vec::is_empty")]
    permitted_api_endpoints: Vec<PermittedApiEndpoint>,
}

impl IntroSpect {
    /// Create a new `IntroSpect` with default values
    #[must_use]
    pub const fn new(metadata: PluginMetadata) -> Self {
        Self {
            metadata,
            static_capture_settings: None,
            dynamic_capture_settings: None,
            ui_path: None,
            subscriptions: None,
            permitted_api_endpoints: vec![],
        }
    }

    /// Set the metadata of the `IntroSpect`
    #[must_use]
    pub fn with_metadata(mut self, metadata: PluginMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a permitted API endpoint to the `IntroSpect`
    #[must_use]
    pub fn add_permitted_api_endpoint(mut self, endpoint: PermittedApiEndpoint) -> Self {
        self.permitted_api_endpoints.push(endpoint);
        self
    }

    /// Set the static capture settings of the `IntroSpect`
    #[must_use]
    pub fn with_static_capture_settings(mut self, settings: StaticCaptureSettings) -> Self {
        self.static_capture_settings = Some(settings);
        self
    }

    /// Set the dynamic capture settings of the `IntroSpect`
    #[must_use]
    pub fn with_dynamic_capture_settings(mut self, settings: DynamicCaptureSettings) -> Self {
        self.dynamic_capture_settings = Some(settings);
        self
    }

    /// Set the UI path of the `IntroSpect`
    #[must_use]
    pub fn with_ui_path<S: AsRef<str>>(mut self, ui_path: S) -> Self {
        self.ui_path = Some(ui_path.as_ref().to_string());
        self
    }

    /// Set the subscriptions settings of the `IntroSpect`
    #[must_use]
    pub fn with_subscriptions(mut self, subscriptions: SubscriptionsSettings) -> Self {
        self.subscriptions = Some(subscriptions);
        self
    }
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct PluginMetadata {
    /// Unique ID of your plugin
    /// recommended to use reverse domain name notation
    /// e.g. "com.yourdomain.pluginname"
    id: String,
    /// Human readable name of your plugin
    name: String,
    /// Author name of your plugin
    author: String,
    /// Author contact information, like email
    #[serde(skip_serializing_if = "String::is_empty")]
    contact: String,
    /// Description of your plugin
    description: String,
    /// URL of your plugin
    /// e.g. project homepage or repository
    url: String,
    /// Type of your plugin (e.g. Router(0) or Utilities(1))
    #[serde(rename = "type")]
    plugin_type: PluginType,
    /// Major version of your plugin
    version_major: u8,
    /// Minor version of your plugin
    version_minor: u8,
    /// Patch version of your plugin
    version_patch: u8,
}

impl PluginMetadata {
    /// Create a new `PluginMetadata` with default values
    #[must_use]
    pub const fn new(plugin_type: PluginType) -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            plugin_type,
            version_major: 0,
            version_minor: 0,
            version_patch: 0,
            description: String::new(),
            author: String::new(),
            contact: String::new(),
            url: String::new(),
        }
    }

    /// Set the ID of the `PluginMetadata`
    #[must_use]
    pub fn with_id<S: AsRef<str>>(mut self, id: S) -> Self {
        self.id = id.as_ref().to_string();
        self
    }

    /// Set the name of the `PluginMetadata`
    #[must_use]
    pub fn with_name<S: AsRef<str>>(mut self, name: S) -> Self {
        self.name = name.as_ref().to_string();
        self
    }

    /// Set the version of the `PluginMetadata`
    #[must_use]
    pub const fn with_version(mut self, version: (u8, u8, u8)) -> Self {
        self.version_major = version.0;
        self.version_minor = version.1;
        self.version_patch = version.2;
        self
    }

    /// Set the description of the `PluginMetadata`
    #[must_use]
    pub fn with_description<S: AsRef<str>>(mut self, description: S) -> Self {
        self.description = description.as_ref().to_string();
        self
    }

    /// Set the author of the `PluginMetadata`
    #[must_use]
    pub fn with_author<S: AsRef<str>>(mut self, author: S) -> Self {
        self.author = author.as_ref().to_string();
        self
    }

    /// Set the contact of the `PluginMetadata`
    #[must_use]
    pub fn with_contact<S: AsRef<str>>(mut self, contact: S) -> Self {
        self.contact = contact.as_ref().to_string();
        self
    }

    /// Set the URL of the `PluginMetadata`
    #[must_use]
    pub fn with_url<S: AsRef<str>>(mut self, url: S) -> Self {
        self.url = url.as_ref().to_string();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum PluginType {
    Router = 0,
    Utilities = 1,
}

impl serde::Serialize for PluginType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Static Capture Settings
///
/// Once plugin is enabled these rules always apply to the enabled HTTP Proxy rule
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct StaticCaptureSettings {
    /// Static capture paths of your plugin, see Zoraxy documentation for more details
    static_capture_paths: Vec<StaticCaptureRule>,
    /// Static capture ingress path of your plugin (e.g. `/s_handler`)
    static_capture_ingress: String,
}

impl StaticCaptureSettings {
    /// Create a new `StaticCaptureSettings` with default values
    #[must_use]
    pub fn new<S: AsRef<str>>(static_capture_ingress: S) -> Self {
        Self {
            static_capture_ingress: static_capture_ingress.as_ref().to_string(),
            static_capture_paths: vec![],
        }
    }

    /// Add a static capture rule to the `StaticCaptureSettings`
    #[must_use]
    pub fn add_static_capture_path<S: AsRef<str>>(mut self, rule: S) -> Self {
        self.static_capture_paths.push(StaticCaptureRule::new(rule));
        self
    }
}

/// Static Capture Rule
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct StaticCaptureRule {
    capture_path: String,
}

impl StaticCaptureRule {
    /// Create a new `StaticCaptureRule` with default values
    #[must_use]
    pub fn new<S: AsRef<str>>(capture_path: S) -> Self {
        Self {
            capture_path: capture_path.as_ref().to_string(),
        }
    }
}

/// Dynamic Capture Settings
///
/// Once plugin is enabled, these rules will be captured and forward to plugin sniff
/// if the plugin sniff returns 280, the traffic will be captured
/// otherwise, the traffic will be forwarded to the next plugin
/// This is slower than static capture, but more flexible
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct DynamicCaptureSettings {
    /// Dynamic capture sniff path of your plugin (e.g. `/d_sniff`)
    dynamic_capture_sniff: String,
    /// Dynamic capture ingress path of your plugin (e.g. `/d_handler`)
    dynamic_capture_ingress: String,
}

impl DynamicCaptureSettings {
    /// Create a new `DynamicCaptureSettings` with default values
    #[must_use]
    pub fn new<S: AsRef<str>>(dynamic_capture_sniff: S, dynamic_capture_ingress: S) -> Self {
        Self {
            dynamic_capture_sniff: dynamic_capture_sniff.as_ref().to_string(),
            dynamic_capture_ingress: dynamic_capture_ingress.as_ref().to_string(),
        }
    }
}

/// Subscriptions Settings
///
/// Once plugin is enabled, Zoraxy will send subscription events to the plugin
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SubscriptionsSettings {
    /// Subscription event path of your plugin (e.g. `/notifyme`),
    /// a POST request with `SubscriptionEvent` as body will be sent to this path when the event is triggered
    subscription_path: String,
    /// Event subscriptions of your plugin,
    /// paired with comments describing how the event is used, see Zoraxy documentation for more details
    #[serde(rename = "subscriptions_events")]
    event_subscriptions: HashMap<EventName, String>,
}

impl SubscriptionsSettings {
    /// Create a new `SubscriptionsSettings` with default values
    #[must_use]
    pub fn new<S: AsRef<str>>(subscription_path: S) -> Self {
        Self {
            subscription_path: subscription_path.as_ref().to_string(),
            event_subscriptions: HashMap::new(),
        }
    }

    /// Add a subscription event to the `SubscriptionsSettings`
    #[must_use]
    pub fn add_event_subscription<S: AsRef<str>>(
        mut self,
        event: EventName,
        description: S,
    ) -> Self {
        self.event_subscriptions
            .insert(event, description.as_ref().to_string());
        self
    }
}
/// Permitted API Endpoint
///
/// An API endpoint that the plugin is allowed to access
#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub struct PermittedApiEndpoint {
    /// HTTP method for the API endpoint (e.g., GET, POST)
    method: String,
    ///The API endpoint that the plugin can access
    endpoint: String,
    ///The reason why the plugin needs to access this endpoint
    reason: Option<String>,
}

impl PermittedApiEndpoint {
    /// Create a new `PermittedApiEndpoint` with default values
    #[must_use]
    pub fn new<S: AsRef<str>>(method: S, endpoint: S) -> Self {
        Self {
            method: method.as_ref().to_string(),
            endpoint: endpoint.as_ref().to_string(),
            reason: None,
        }
    }

    /// Set the reason of the `PermittedApiEndpoint`
    #[must_use]
    pub fn with_reason<S: AsRef<str>>(mut self, reason: S) -> Self {
        self.reason = Some(reason.as_ref().to_string());
        self
    }
}
