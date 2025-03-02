use std::{collections::HashMap, path::PathBuf};
use dsync::{GenerationConfig, TableOptions};

pub fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");
    
    dsync::generate_files(
        PathBuf::from_iter([dir, "src/schema.rs"]).as_path(), 
        PathBuf::from_iter([dir, "src/models"]).as_path(), 
        GenerationConfig {
           connection_type: "diesel::pg::PgConnection".to_string(),
           options: Default::default(),
        }
    ).ok();
}