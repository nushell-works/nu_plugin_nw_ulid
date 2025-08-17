use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_ulid::UlidPlugin;

fn main() {
    serve_plugin(&UlidPlugin, MsgPackSerializer {})
}
