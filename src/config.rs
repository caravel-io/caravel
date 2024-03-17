use serde::Deserialize;

/*
Example toml config:

listen_port = '8080'
listen_address = '0.0.0.0'

interval = '30'
manifest = '/path/to/manifest.lua' # or 'https://url/to/manifest.lua'
splay = '0'

*/

#[derive(Deserialize, Debug, Clone)]
pub struct AgentConfig {
    // Listen mode
    pub listen_port: Option<u64>,
    pub listen_address: Option<String>,
    pub disable_listen: Option<bool>,

    // Pull mode
    pub interval: Option<u64>,
    pub splay: Option<u64>,
    pub manifest: Option<String>,
}

impl AgentConfig {
    // Default values should be set here
    pub fn new() -> AgentConfig {
        AgentConfig {
            listen_port: Some(1336),
            listen_address: Some("0.0.0.0".to_string()),
            disable_listen: Some(false),

            interval: Some(30),
            splay: Some(0),
            manifest: None,
        }
    }

    // This function merges in a config that was brought in via TOML
    pub fn merge_with(&mut self, other: &AgentConfig) {
        if let Some(listen_port) = other.listen_port {
            self.listen_port = Some(listen_port);
        }
        if let Some(listen_address) = &other.listen_address {
            self.listen_address = Some(listen_address.clone());
        }
        if let Some(disable_listen) = &other.disable_listen {
            self.disable_listen = Some(disable_listen.clone());
        }

        if let Some(interval) = other.interval {
            self.interval = Some(interval);
        }
        if let Some(splay) = other.splay {
            self.splay = Some(splay);
        }
        if let Some(manifest) = &other.manifest {
            self.manifest = Some(manifest.clone());
        }
    }

    // This one applies environment variables after all other configs
    pub fn merge_environment(&mut self) {
        if let Ok(port) = std::env::var("CARAVEL_AGENT_PORT") {
            self.listen_port = Some(port.parse().unwrap());
        }
        if let Ok(address) = std::env::var("CARAVEL_AGENT_ADDRESS") {
            self.listen_address = Some(address);
        }
        if let Ok(dl) = std::env::var("CARAVEL_AGENT_DISABLE_LISTEN") {
            let valids = vec!["true", "false", "1", "0"];
            if valids.contains(&dl.as_str()) {
                if dl == "true" || dl == "1" {
                    self.disable_listen = Some(true);
                } else {
                    self.disable_listen = Some(false);
                }
            }
        }

        if let Ok(interval) = std::env::var("CARAVEL_AGENT_INTERVAL") {
            self.interval = Some(interval.parse().unwrap());
        }
        if let Ok(splay) = std::env::var("CARAVEL_AGENT_SPLAY") {
            self.splay = Some(splay.parse().unwrap());
        }
        if let Ok(manifest) = std::env::var("CARAVEL_AGENT_MANIFEST") {
            self.manifest = Some(manifest);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_default_config() {
        let mut config = super::AgentConfig::new();
        let toml = r#""#;
        config.merge_with(&toml::from_str(toml).unwrap());
        assert_eq!(config.listen_port, Some(1336));
        assert_eq!(config.listen_address, Some("0.0.0.0".to_string()));
        assert_eq!(config.disable_listen, Some(false));
        assert_eq!(config.interval, Some(30));
        assert_eq!(config.splay, Some(0));
        assert_eq!(config.manifest, None);
    }

    #[test]
    fn test_toml_config() {
        let mut config = super::AgentConfig::new();
        let toml = r#"
        listen_port = 8080
        listen_address = '0.0.0.0'
        disable_listen = true
        interval = 60
        splay = 10
        manifest = '/path/to/manifest.lua'
        "#;
        config.merge_with(&toml::from_str(toml).unwrap());
        assert_eq!(config.listen_port, Some(8080));
        assert_eq!(config.listen_address, Some("0.0.0.0".to_string()));
        assert_eq!(config.disable_listen, Some(true));
        assert_eq!(config.interval, Some(60));
        assert_eq!(config.splay, Some(10));
        assert_eq!(config.manifest, Some("/path/to/manifest.lua".to_string()));
    }

    #[test]
    fn test_env_config() {
        let mut config = super::AgentConfig::new();
        std::env::set_var("CARAVEL_AGENT_PORT", "8080");
        std::env::set_var("CARAVEL_AGENT_ADDRESS", "1.1.1.1");
        std::env::set_var("CARAVEL_AGENT_DISABLE_LISTEN", "true");
        std::env::set_var("CARAVEL_AGENT_INTERVAL", "60");
        std::env::set_var("CARAVEL_AGENT_SPLAY", "10");
        std::env::set_var("CARAVEL_AGENT_MANIFEST", "/path/to/manifest.lua");
        config.merge_environment();
        std::env::remove_var("CARAVEL_AGENT_PORT");
        std::env::remove_var("CARAVEL_AGENT_ADDRESS");
        std::env::remove_var("CARAVEL_AGENT_DISABLE_LISTEN");
        std::env::remove_var("CARAVEL_AGENT_INTERVAL");
        std::env::remove_var("CARAVEL_AGENT_SPLAY");
        std::env::remove_var("CARAVEL_AGENT_MANIFEST");
        assert_eq!(config.listen_port, Some(8080));
        assert_eq!(config.listen_address, Some("1.1.1.1".to_string()));
        assert_eq!(config.disable_listen, Some(true));
        assert_eq!(config.interval, Some(60));
        assert_eq!(config.splay, Some(10));
        assert_eq!(config.manifest, Some("/path/to/manifest.lua".to_string()));
    }
}
