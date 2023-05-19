use std::{path::{PathBuf, Path}, collections::HashMap};
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{LanguageTree, Lexis}, transforms::Transform, word::Etymology};
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
    // because we later need to get random words from the map to construct the etymology from the rawLex "etymology" fields,
    // the giant hashmaps of everything need to be made first
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
    
    if language_map.is_empty(){
        return Err(anyhow!("specified language tree does not contain any data. Tree files used: {:?}", graphs));
    }

    let mut tree = LanguageTree::new();

    for (lex_name, node) in &language_map{
        debug!("creating node entry {}", lex_name);
        let node_lex: Lexis = Lexis { id: lex_name.to_string(), ..node.clone().into() };
        add_single_word(&mut tree, &transform_map, &language_map, &node_lex, &node.etymology)?;
        // connect derivatives
        if let Some(derivatives) = &node.derivatives {
            debug!("Node {} has derivatives, adding", lex_name);
            let mut count = 0;
            for der in derivatives {
                let der_id = format!("{}-autoderive-{}", lex_name, count);
                let der_lex = Lexis{id: der_id, ..der.lexis.clone().into()};
                let der_transforms: Vec<Transform> = match &der.transforms{
                    Some(t) => {find_transforms(&t, &transform_map)?},
                    None => Vec::new(),
                };
                tree.connect_etymology(der_lex, node_lex.clone(), der_transforms, None);
                count+=1;
            }
        };
        
       
    }

    Ok(tree)
}

fn add_single_word(tree: &mut LanguageTree, trans_map: &HashMap<String, RawTransform>, 
    lex_map: &HashMap<String, RawLexicalEntry>, node_lex: &Lexis, lex_ety: &Option<Etymology>) -> Result<()> {
        if let Some(etymon) = lex_ety{
            // iterate through all etymons associated with the base word, construct the transforms and add the etymology for each
            for e in &etymon.etymons{
                // fetch transform list
                let word_transforms = find_transforms(&e.transforms, trans_map)?;

                let ety_lex: RawLexicalEntry = lex_map.get(&e.etymon).context(format!("etymon {} does not exist ", &e.etymon))?.clone();
                tree.connect_etymology(node_lex.clone(), Lexis { id: e.etymon.clone(), ..ety_lex.into()}, word_transforms, e.agglutination_order);
            }

        }
    Ok(())
}

pub fn find_transforms(raw: &Vec<String>, trans_tree: &HashMap<String, RawTransform>) -> Result<Vec<Transform>> {
    let mut word_transforms: Vec<Transform> = Vec::new();
    for trans in raw{
        let trans_raw = trans_tree.get(trans).context(format!("transform {} does not exist", trans))?;
        let converted_trans: Transform = trans_raw.clone().into();
        word_transforms.push(Transform { name: trans.clone(), ..converted_trans });
    }

    Ok(word_transforms)
}

pub fn handle_directory(path: String) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let lang_dir = Path::new(&path);
    let lang_graph_dir = lang_dir.join("tree");
    let lang_transform_dir = lang_dir.join("etymology");

    let mut graphs: Vec<PathBuf> = Vec::new();
    let mut transforms: Vec<PathBuf> = Vec::new();

    debug!("using tree path: {}", lang_graph_dir.display());
    for entry in WalkDir::new(lang_graph_dir).into_iter().filter_entry(check_path){
        let path = entry?.path().to_path_buf();
        if !path.is_dir(){
            graphs.push(path);
        }
        
    }

    debug!("using etymology path: {}", lang_transform_dir.display());
    for entry in WalkDir::new(lang_transform_dir).into_iter().filter_entry(check_path){
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