use std::env;

use foundry_compilers::{Project, ProjectPathsConfig};

fn main() {
    let project = Project::builder()
    .paths(ProjectPathsConfig::hardhat(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "./contracts")).unwrap())
    .build()
    .unwrap();
    let output = project.compile().unwrap();
}