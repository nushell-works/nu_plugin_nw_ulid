use nu_plugin::{EngineInterface, EvaluatedCall, Plugin, PluginCommand};
use nu_protocol::{Example, LabeledError, PipelineData, Signature, Value};

pub struct UlidPlugin;

impl Plugin for UlidPlugin {
    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            Box::new(UlidInfoCommand),
        ]
    }
}

pub struct UlidInfoCommand;

impl PluginCommand for UlidInfoCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid info"
    }

    fn description(&self) -> &str {
        "Display plugin metadata and diagnostics"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
    }

    fn examples(&self) -> Vec<Example> {
        vec![
            Example {
                example: "ulid info",
                description: "Show plugin information",
                result: None,
            },
        ]
    }

    fn run(
        &self,
        _plugin: &Self::Plugin,
        _engine: &EngineInterface,
        call: &EvaluatedCall,
        _input: PipelineData,
    ) -> Result<PipelineData, LabeledError> {
        let record = Value::record(
            [
                ("name".into(), Value::string("nu_plugin_ulid", call.head)),
                ("version".into(), Value::string(env!("CARGO_PKG_VERSION"), call.head)),
                ("description".into(), Value::string(env!("CARGO_PKG_DESCRIPTION"), call.head)),
                ("authors".into(), Value::string(env!("CARGO_PKG_AUTHORS"), call.head)),
                ("license".into(), Value::string(env!("CARGO_PKG_LICENSE"), call.head)),
                ("repository".into(), Value::string(env!("CARGO_PKG_REPOSITORY"), call.head)),
            ].into_iter().collect(),
            call.head,
        );

        Ok(PipelineData::Value(record, None))
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
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name(), "ulid info");
    }
}
