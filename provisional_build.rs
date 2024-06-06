use std::env;

use foundry_compilers::{Project, ProjectPathsConfig};

fn main() {
    let project = Project::builder()
    .paths(ProjectPathsConfig::hardhat(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "./contracts")).unwrap())
    .build()
    .unwrap();
    let output = project.compile().unwrap();

    // Tell Cargo that if a source file changes, to rerun this build script.
    project.rerun_if_sources_changed();
}