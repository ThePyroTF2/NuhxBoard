use std::{
    fs::{self, File},
    path::Path,
};

use schemars::schema_for;
use types::{config::Layout, style::Style};

fn main() {
    let layout_schema = schema_for!(Layout);
    let style_schema = schema_for!(Style);

    let schema_dir = Path::new("schemas");

    fs::create_dir_all(schema_dir).unwrap();

    let style_file = File::create(schema_dir.join("style.json")).unwrap();
    serde_json::to_writer_pretty(style_file, &style_schema).unwrap();

    let layout_file = File::create(schema_dir.join("layout.json")).unwrap();
    serde_json::to_writer_pretty(layout_file, &layout_schema).unwrap();
}