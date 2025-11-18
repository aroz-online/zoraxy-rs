use anyhow::{Result, bail};

/// `RecvExecuteConfigureSpec` Function
///
/// This function will read the configure spec from Zoraxy
/// and return the `ConfigureSpec` object
///
/// Place this function after `ServeIntroSpect` function in your plugin main function
///
/// # Arguments
/// * `args` - A vector of strings representing the command line arguments
///
/// # Returns
/// * `Result<crate::types::ConfigureSpec>` - The `ConfigureSpec` object wrapped in a Result
///
/// # Errors
/// * This function will return an error if -configure flag is not present
/// * This function will return an error if deserialization of `ConfigureSpec` fails
pub fn recv_configuration_spec(args: Vec<String>) -> Result<crate::types::ConfigureSpec> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "-configure" {
            let Some(spec_json) = args.into_iter().nth(i + 1) else {
                bail!("No configure spec provided after -configure");
            };
            let spec: crate::types::ConfigureSpec = serde_json::from_str(&spec_json)?;
            return Ok(spec);
        } else if arg.starts_with("-configure=") {
            let spec_json = arg.trim_start_matches("-configure=");
            let spec: crate::types::ConfigureSpec = serde_json::from_str(spec_json)?;
            return Ok(spec);
        }
    }

    bail!("no -configure flag found");
}

/// `ServeIntroSpect` Function
///
/// This function will check if the plugin is initialized with -introspect flag,
/// if so, it will print the intro spect and exit
///
/// Place this function at the beginning of your plugin main function
///
/// # Arguments
/// * `args` - A vector of strings representing the command line arguments
/// * `intro_spect` - A reference to the `IntroSpect` object to be printed
///
/// # Returns
/// * `Result<String>` - The intro spect as a JSON string if -introspect flag is present, otherwise Err
///
/// # Errors
/// * This function will return an error if -introspect flag is not present
///
/// # Panics
/// * This function will panic if serialization of `intro_spect` fails
pub fn serve_intro_spect(
    args: &[String],
    intro_spect: &crate::types::IntroSpect,
) -> Result<String> {
    if let Some(arg) = args.get(1)
        && arg == "-introspect"
    {
        let intro_spect_json =
            serde_json::to_string_pretty(intro_spect).expect("Failed to serialize IntroSpect");
        Ok(intro_spect_json)
    } else {
        bail!("no -introspect flag found");
    }
}

/// `ServeAndRecvSpec` Function
///
/// This function will serve the intro spect and return the configure spec
/// See the `ServeIntroSpect` and `RecvConfigureSpec` for more details
///
/// # Arguments
/// * `args` - A vector of strings representing the command line arguments
/// * `intro_spect` - A reference to the `IntroSpect` object to be printed
/// # Returns
/// * `Result<crate::types::ConfigureSpec>` - The `ConfigureSpec` object  wrapped in a Result
/// # Errors
/// * This function will return an error if receiving the configure spec fails
/// # Exits
/// * This function will exit the process if it serves the intro spect
pub fn serve_and_recv_spec(
    args: Vec<String>,
    intro_spect: &crate::types::IntroSpect,
) -> Result<crate::types::ConfigureSpec> {
    serve_intro_spect(&args, intro_spect).map_or_else(
        |_| recv_configuration_spec(args),
        |intro_spect_json| {
            println!("{intro_spect_json}");
            std::process::exit(0);
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::{PluginMetadata, PluginType, types::IntroSpect};

    use super::*;

    fn helloworld_intro_spect() -> IntroSpect {
        let metadata = PluginMetadata::new(PluginType::Utilities)
            .with_id("com.example.helloworld")
            .with_name("Hello World Plugin")
            .with_author("foobar")
            .with_contact("admin@example.com")
            .with_description("A simple hello world plugin")
            .with_url("https://example.com")
            .with_version((1, 0, 0));
        IntroSpect::new(metadata).with_ui_path("/")
    }
    const fn helloworld_expected_json() -> &'static str {
        r#"{
  "id": "com.example.helloworld",
  "name": "Hello World Plugin",
  "author": "foobar",
  "contact": "admin@example.com",
  "description": "A simple hello world plugin",
  "url": "https://example.com",
  "type": 1,
  "version_major": 1,
  "version_minor": 0,
  "version_patch": 0,
  "ui_path": "/"
}"#
    }

    #[test]
    fn test_serve_intro_spect_helloworld() {
        let args = vec!["plugin".to_string(), "-introspect".to_string()];

        let result = serve_intro_spect(&args, &helloworld_intro_spect());
        assert!(result.is_ok());
        let intro_spect_json = result.unwrap();

        pretty_assertions::assert_eq!(intro_spect_json, helloworld_expected_json());
    }

    #[test]
    fn test_serve_intro_spect_no_flag() {
        let intro_spect = helloworld_intro_spect();
        let args = vec!["plugin".to_string()];

        let result = serve_intro_spect(&args, &intro_spect);
        assert!(result.is_err());
    }

    #[test]
    fn test_recv_configuration_spec() {
        let spec = r#"{
            "port": 8080,
            "runtime_const": {
                "zoraxy_version": "1.2.3",
                "zoraxy_uuid": "abcd-efgh-ijkl",
                "development_build": true
            },
            "api_key": "my_api_key",
            "zoraxy_port": 9090
        }"#;

        let args = vec![
            "plugin".to_string(),
            "-configure".to_string(),
            spec.to_string(),
        ];

        let result = recv_configuration_spec(args);
        assert!(result.is_ok());
        let config_spec = result.unwrap();

        assert_eq!(config_spec.port, 8080);
        assert_eq!(config_spec.runtime_constants.zoraxy_version, "1.2.3");
        assert_eq!(config_spec.runtime_constants.zoraxy_uuid, "abcd-efgh-ijkl");
        assert!(config_spec.runtime_constants.development_build);
        assert_eq!(config_spec.api_key.unwrap(), "my_api_key");
        assert_eq!(config_spec.zoraxy_port, 9090);
    }
}
