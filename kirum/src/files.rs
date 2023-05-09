use std::{path::{PathBuf, Path}, collections::HashMap};
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{LanguageTree, Lexis}, transforms::Transform};
use walkdir::{WalkDir, DirEntry};
use crate::entries::{RawTransform, RawLexicalEntry};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WordGraph {
    pub words: HashMap<String, RawLexicalEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformGraph {
    pub transforms: HashMap<String, RawTransform>
}

pub fn read_from_files(transforms:Vec<PathBuf>, graphs:Vec<PathBuf>) -> Result<LanguageTree>{
    //first merge all the files into one giant hashmap for the transforms and graph
    let mut transform_map: HashMap<String, RawTransform> = HashMap::new();
    for trans_file in &transforms {
        let trans_raw = std::fs::read_to_string(trans_file.clone()).context(format!("error reading etymology file {}", trans_file.display()))?;
        let transforms: TransformGraph = serde_json::from_str(&trans_raw).context(format!("error parsing etymology file {}", trans_file.display()))?;
        debug!("read in transform file: {}", trans_file.display());
        transform_map.extend(transforms.transforms);
    }

    let mut language_map: HashMap<String, RawLexicalEntry> = HashMap::new();
    for lang_file in &graphs{
        let graph_raw = std::fs::read_to_string(lang_file.clone()).context(format!("error reading tree file {}", lang_file.display()))?;
        let raw_graph: WordGraph = serde_json::from_str(&graph_raw).context(format!("error reading tree file {}", lang_file.display()))?;
        debug!("read in language file: {}", lang_file.display());
        language_map.extend(raw_graph.words);
    }
    
    if language_map.len() == 0 {
        return Err(anyhow!("specified language tree does not contain any data. Tree files used: {:?}", graphs));
    }

    let mut tree = LanguageTree::new();

    for (lex_name, node) in &language_map{
        let node_lex: Lexis = Lexis { id: lex_name.to_string(), ..node.clone().into() };

        if let Some(etymon) = node.etymology.clone(){
            for e in etymon.etymons{
                // fetch transform list
                let mut word_transforms: Vec<Transform> = Vec::new();
                for trans in e.transform{
                    let trans_raw = transform_map.get(&trans).context(format!("transform {} does not exist", trans))?;
                    let converted_trans: Transform = trans_raw.clone().into();
                    word_transforms.push(Transform { name: trans, ..converted_trans });
                }

                let ety_lex: RawLexicalEntry = language_map.get(&e.etymon).context(format!("etymon {} does not exist ", &e.etymon))?.clone();
                tree.connect_etymology(node_lex.clone(), Lexis { id: e.etymon, ..ety_lex.into()}, word_transforms, e.agglutination_order);
            }
        }
       
    }

    Ok(tree)
}

pub fn handle_directory(path: String) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let lang_dir = Path::new(&path);
    let lang_graph_dir = lang_dir.join("tree");
    let lang_transform_dir = lang_dir.join("etymology");

    let mut graphs: Vec<PathBuf> = Vec::new();
    let mut transforms: Vec<PathBuf> = Vec::new();

    debug!("using tree path: {}", lang_graph_dir.display());
    for entry in WalkDir::new(lang_graph_dir.clone()).into_iter().filter_entry(|p| check_path(p)){
        let path = entry?.path().to_path_buf();
        if !path.is_dir(){
            graphs.push(path);
        }
        
    }

    debug!("using etymology path: {}", lang_transform_dir.display());
    for entry in WalkDir::new(lang_transform_dir).into_iter().filter_entry(|p| check_path(p)){
        let path = entry?.path().to_path_buf();
        if !path.is_dir(){
            transforms.push(path);
        }
        
    }

    Ok((transforms, graphs))
}

fn check_path(dir: &DirEntry) -> bool {
    debug!("checking path: {:?}", dir);
    if dir.file_type().is_dir(){
        true
    } else  {
        dir.path().extension().unwrap_or_default() == "json"
    }
    
}