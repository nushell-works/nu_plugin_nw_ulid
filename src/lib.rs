use nu_plugin::{Plugin, PluginCommand};

mod commands;
use commands::*;

pub struct UlidPlugin;

impl Plugin for UlidPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(UlidInfoCommand),
            Box::new(UlidUuidGenerateCommand),
            Box::new(UlidUuidValidateCommand),
            Box::new(UlidUuidParseCommand),
            Box::new(UlidTimeNowCommand),
            Box::new(UlidTimeParseCommand),
            Box::new(UlidTimeMillisCommand),
            Box::new(UlidEncodeBase32Command),
            Box::new(UlidDecodeBase32Command),
            Box::new(UlidEncodeHexCommand),
            Box::new(UlidDecodeHexCommand),
            Box::new(UlidHashSha256Command),
            Box::new(UlidHashSha512Command),
            Box::new(UlidHashBlake3Command),
            Box::new(UlidHashRandomCommand),
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
        assert_eq!(commands.len(), 15);

        // Test key commands to ensure they're registered correctly
        let command_names: Vec<&str> = commands.iter().map(|cmd| cmd.name()).collect();
        assert!(command_names.contains(&"ulid info"));
        assert!(command_names.contains(&"ulid uuid generate"));
        assert!(command_names.contains(&"ulid time now"));
        assert!(command_names.contains(&"ulid encode base32"));
        assert!(command_names.contains(&"ulid hash sha256"));
    }
}
