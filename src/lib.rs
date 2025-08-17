use nu_plugin::{Plugin, PluginCommand};

mod commands;
mod error;
mod security;
mod ulid_engine;

use commands::*;
pub use error::*;
pub use security::*;
pub use ulid_engine::*;

pub struct UlidPlugin;

impl Plugin for UlidPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            // Core ULID commands
            Box::new(UlidGenerateCommand),
            Box::new(UlidValidateCommand),
            Box::new(UlidParseCommand),
            Box::new(UlidInspectCommand),
            Box::new(UlidSortCommand),
            Box::new(UlidSecurityAdviceCommand),
            // Plugin info
            Box::new(UlidInfoCommand),
            // UUID utilities
            Box::new(UlidUuidGenerateCommand),
            Box::new(UlidUuidValidateCommand),
            Box::new(UlidUuidParseCommand),
            // Time utilities
            Box::new(UlidTimeNowCommand),
            Box::new(UlidTimeParseCommand),
            Box::new(UlidTimeMillisCommand),
            // Encoding utilities
            Box::new(UlidEncodeBase32Command),
            Box::new(UlidDecodeBase32Command),
            Box::new(UlidEncodeHexCommand),
            Box::new(UlidDecodeHexCommand),
            // Hashing utilities
            Box::new(UlidHashSha256Command),
            Box::new(UlidHashSha512Command),
            Box::new(UlidHashBlake3Command),
            Box::new(UlidHashRandomCommand),
            // Streaming utilities
            Box::new(UlidStreamCommand),
            Box::new(UlidGenerateStreamCommand),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_version() {
        let plugin = UlidPlugin;
        assert!(!plugin.version().is_empty());
    }

    #[test]
    fn test_plugin_commands() {
        let plugin = UlidPlugin;
        let commands = plugin.commands();
        assert_eq!(commands.len(), 23);

        // Test key commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"ulid generate"));
        assert!(command_names.contains(&"ulid validate"));
        assert!(command_names.contains(&"ulid parse"));
        assert!(command_names.contains(&"ulid inspect"));
        assert!(command_names.contains(&"ulid sort"));
        assert!(command_names.contains(&"ulid security-advice"));
        assert!(command_names.contains(&"ulid info"));
        assert!(command_names.contains(&"ulid uuid generate"));
        assert!(command_names.contains(&"ulid time now"));
        assert!(command_names.contains(&"ulid encode base32"));
        assert!(command_names.contains(&"ulid hash sha256"));
        assert!(command_names.contains(&"ulid stream"));
        assert!(command_names.contains(&"ulid generate-stream"));
    }
}
