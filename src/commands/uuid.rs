use nu_plugin::{EngineInterface, EvaluatedCall, PluginCommand};
use nu_protocol::{
    Category, Example, LabeledError, PipelineData, Signature, Span, SyntaxShape, Type, Value,
};
use uuid::Uuid;

use crate::UlidPlugin;

pub struct UlidUuidGenerateCommand;

impl PluginCommand for UlidUuidGenerateCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid uuid generate"
    }

    fn usage(&self) -> &str {
        "Generate a random UUID v4"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .input_output_types(vec![(Type::Nothing, Type::String)])
            .category(Category::Generators)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid uuid generate",
                description: "Generate a random UUID v4",
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
        let uuid = Uuid::new_v4();
        Ok(PipelineData::Value(
            Value::string(uuid.to_string(), call.head),
            None,
        ))
    }
}

pub struct UlidUuidValidateCommand;

impl PluginCommand for UlidUuidValidateCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid uuid validate"
    }

    fn usage(&self) -> &str {
        "Validate if a string is a valid UUID"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("uuid", SyntaxShape::String, "The UUID string to validate")
            .input_output_types(vec![(Type::Nothing, Type::Bool)])
            .category(Category::Strings)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid uuid validate '550e8400-e29b-41d4-a716-446655440000'",
                description: "Validate a UUID string",
                result: Some(Value::bool(true, Span::test_data())),
            },
            Example {
                example: "ulid uuid validate 'invalid-uuid'",
                description: "Validate an invalid UUID string",
                result: Some(Value::bool(false, Span::test_data())),
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
        let uuid_str: String = call.req(0)?;
        let is_valid = Uuid::parse_str(&uuid_str).is_ok();
        
        Ok(PipelineData::Value(
            Value::bool(is_valid, call.head),
            None,
        ))
    }
}

pub struct UlidUuidParseCommand;

impl PluginCommand for UlidUuidParseCommand {
    type Plugin = UlidPlugin;

    fn name(&self) -> &str {
        "ulid uuid parse"
    }

    fn usage(&self) -> &str {
        "Parse a UUID string and extract its components"
    }

    fn signature(&self) -> Signature {
        Signature::build(self.name())
            .required("uuid", SyntaxShape::String, "The UUID string to parse")
            .input_output_types(vec![(Type::Nothing, Type::Record(vec![].into()))])
            .category(Category::Strings)
    }

    fn examples(&self) -> Vec<Example<'_>> {
        vec![
            Example {
                example: "ulid uuid parse '550e8400-e29b-41d4-a716-446655440000'",
                description: "Parse a UUID and show its components",
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
        let uuid_str: String = call.req(0)?;
        
        match Uuid::parse_str(&uuid_str) {
            Ok(uuid) => {
                let bytes = uuid.as_bytes();
                let version = uuid.get_version_num();
                let variant = match uuid.get_variant() {
                    uuid::Variant::NCS => "NCS",
                    uuid::Variant::RFC4122 => "RFC4122",
                    uuid::Variant::Microsoft => "Microsoft",
                    uuid::Variant::Future => "Future",
                    _ => "Unknown",
                };

                let record = Value::record(
                    [
                        ("uuid".into(), Value::string(uuid.to_string(), call.head)),
                        ("version".into(), Value::int(version as i64, call.head)),
                        ("variant".into(), Value::string(variant, call.head)),
                        ("hyphenated".into(), Value::string(uuid.hyphenated().to_string(), call.head)),
                        ("simple".into(), Value::string(uuid.simple().to_string(), call.head)),
                        ("urn".into(), Value::string(uuid.urn().to_string(), call.head)),
                        ("bytes".into(), Value::binary(bytes.to_vec(), call.head)),
                    ]
                    .into_iter()
                    .collect(),
                    call.head,
                );

                Ok(PipelineData::Value(record, None))
            }
            Err(e) => Err(LabeledError::new("Invalid UUID")
                .with_label(format!("Failed to parse UUID: {}", e), call.head)),
        }
    }
}