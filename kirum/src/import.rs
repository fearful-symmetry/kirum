
use std::path::Path;

use anyhow::{Result, Context};
use crate::{cli::{Ingest, self}, entries::RawLexicalEntry, ingest::{self, json, lines}, files::add_tree_file, new};

/// import and ingest a file, create a kirum tree file from the result
pub fn ingest_from_cli(overrides: Option<Vec<String>>, directory: String, out: String, command: Ingest) -> Result<()> {
    let lex_override = match overrides {
        Some(raw) => ingest::overrides::parse(raw)?,
        None => RawLexicalEntry::default()
    };
    let new_tree = match command{
        cli::Ingest::Json { file } => {
            json::ingest(file, lex_override).context("error parsing json file")?
        },
        cli::Ingest::Lines { file } => {
            lines::ingest(file, lex_override).context("error parsing line file")?
        }
    };
    // check to see if we're in a new project or not
    let base = Path::new(&directory).join("tree");
    if base.exists(){
        info!("project already exists in {}, adding file", directory);
        add_tree_file(directory, out, new_tree).context("error adding file to existing project")?;
    } else {
        info!("creating new project at {}", directory);
        new::create_new_project(&directory).context("error creating new project")?;
        add_tree_file(directory, out, new_tree).context("error adding file to new project")?;
    }



    Ok(())
}