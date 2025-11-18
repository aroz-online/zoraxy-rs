/// `ConfigureSpec` Payload
///
/// Zoraxy will start your plugin with -configure flag,
/// the plugin shell read this payload as JSON and configure itself
/// by the supplied values like starting a web server at given port
/// that listens to 127.0.0.1:port
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigureSpec {
    /// Port to listen
    pub port: u16,
    /// Runtime Constant values
    #[serde(rename = "runtime_const")]
    pub runtime_constants: RuntimeConstants,
    /// API key for accessing Zoraxy APIs, if the plugin has permitted endpoints
    pub api_key: Option<String>,
    /// The port that Zoraxy is running on, used for making API calls to Zoraxy
    pub zoraxy_port: u16,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct RuntimeConstants {
    /// Zoraxy Version
    pub zoraxy_version: String,
    /// Zoraxy UUID
    pub zoraxy_uuid: String,
    /// Whether the Zoraxy is a development build or not
    pub development_build: bool,
}
