
use std::path::Path;
use anyhow::{Result, Context};
use crate::{cli::{Ingest, self}, entries::{RawLexicalEntry, TransformGraph}, ingest::{self, json, lines}, files::{add_tree_file, add_ety_file, handle_directory, read_transform_files}, new};

/// import and ingest a file, create a kirum tree file from the result
pub fn ingest_from_cli(overrides: Option<Vec<String>>, directory: String, out: String, command: Ingest) -> Result<()> {
    let lex_override = match overrides {
        Some(raw) => ingest::overrides::parse(raw)?,
        None => RawLexicalEntry::default()
    };
    let (new_tree, mut new_trans) = match command{
        cli::Ingest::Json { file } => {
            json::ingest(&file, lex_override).context(format!("error parsing json file {}", file))?
        },
        cli::Ingest::Lines { file } => {
            (lines::ingest(&file, lex_override).context(format!("error parsing line file {}", file))?, TransformGraph::default())
        }
    };
    // check to see if we're in a new project or not
    let base = Path::new(&directory).join("tree");
    if base.exists(){
        info!("project already exists in {}, adding file", directory);
    } else {
        info!("creating new project at {}", directory);
        new::create_project_directory(&directory).context("error creating new project")?;
    }
    add_tree_file(&directory, &out, new_tree).context("error added ingested tree file")?;
    if !new_trans.transforms.is_empty() {
        info!("found etymology data, adding...");
        // if an existing project exists, diff any ety rules, only write new ones
        if base.exists(){
            info!("existing transform files found. Only new transform rules will be written.");
            let project = handle_directory(&directory)?;
            let transforms = read_transform_files(&project.transforms)?;
            for (name, _) in transforms {
                new_trans.transforms.remove(&name);
            }
        }
        if !new_trans.transforms.is_empty() {
            add_ety_file(directory, &out, new_trans).context("error adding ingested etymology file")?;
        } else {
            info!("no new transforms found.")
        }
        
    }


    Ok(())
}