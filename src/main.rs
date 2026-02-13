//! Binary entry point for the nu_plugin_nw_ulid plugin.

use nu_plugin::{MsgPackSerializer, serve_plugin};
use nu_plugin_nw_ulid::UlidPlugin;

fn main() {
    serve_plugin(&UlidPlugin, MsgPackSerializer {})
}
