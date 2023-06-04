use std::{path::{PathBuf, Path}, fs::{self, File}, io::Write, collections::HashMap};
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{LanguageTree, Lexis}, transforms::{Transform, TransformFunc}, word::{Etymology, Edge}};
use walkdir::{WalkDir, DirEntry};
use crate::entries::{RawTransform, RawLexicalEntry, TransformGraph, WordGraph, Derivative};


pub fn create_new_project(name: &str) -> Result<()> {
    let base = PathBuf::from(name);
    let mut ety_path = base.join("etymology");
    let mut tree_path = base.join("tree");
    fs::create_dir_all(&ety_path)?;
    fs::create_dir_all(&tree_path)?;

    let mut transform_map: HashMap<String, RawTransform> = HashMap::new();
    transform_map.insert("of-from-latin".into(), RawTransform { 
        transforms: vec![TransformFunc::MatchReplace { old: "exe".into(), new: "esse".into() },
        TransformFunc::MatchReplace { old: "um".into(), new: "e".into() }
        ], 
        conditional: None 
        }
    );
    transform_map.insert("latin-from-verb".into(), RawTransform { 
        transforms: vec![TransformFunc::MatchReplace { old: "ere".into(), new: "plum".into() },
        TransformFunc::Prefix { value: "ex".into() }
        ],
        conditional: None 
        }
    );
    let example_transforms = TransformGraph{transforms: transform_map};

    let mut word_map: HashMap<String, RawLexicalEntry> = HashMap::new();
    word_map.insert("latin_verb".into(), RawLexicalEntry { 
        word: Some("emere".into()), 
        word_type: Some("word".into()), 
        language: Some("Latin".into()), 
        definition: "To buy, remove".into(), 
        part_of_speech: Some(libkirum::word::PartOfSpeech::Verb), 
        etymology: None, 
        archaic: true, 
        tags: None, 
        derivatives: None, 
    });
    word_map.insert("latin_example".into(), RawLexicalEntry { 
        word: None, 
        word_type: Some("word".into()), 
        language: Some("Latin".into()), 
        definition: "an instance, model, example".into(), 
        part_of_speech: Some(libkirum::word::PartOfSpeech::Noun), 
        etymology: Some(Etymology { etymons: vec![Edge{etymon: "latin_verb".into(), transforms: Some(vec!["latin-from-verb".into()]), agglutination_order: None}] }), 
        archaic: true, 
        tags: Some(vec!["example".into(), "default".into()]), 
        derivatives: Some(vec![Derivative{lexis: RawLexicalEntry { 
                word: None, 
                word_type: None, 
                language: Some("Old French".into()), 
                definition: "model, example".into(), 
                part_of_speech: Some(libkirum::word::PartOfSpeech::Noun), 
                etymology: None, 
                archaic: true, 
                tags: None, 
                derivatives: None
            },
            transforms: Some(vec!["of-from-latin".to_owned()]),
    }]) 
    });

    let example_tree = WordGraph{
        words: word_map
    };

    let graph_data = serde_json::to_string_pretty(&example_tree)?;
    let trans_data = serde_json::to_string_pretty(&example_transforms)?;

    tree_path.push(name);
    tree_path.set_extension("json");
    let mut tree_file = File::create(tree_path)?;
    write!(tree_file, "{}", graph_data)?;

    ety_path.push("ety");
    ety_path.set_extension("json");
    let mut ety_file = File::create(ety_path)?;
    write!(ety_file, "{}", trans_data)?;

    Ok(())
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
        // read in derivative words, convert them to "normal" words in the graph
        for (lex_name, node) in &raw_graph.words{
            if let Some(derivatives) = &node.derivatives {
                debug!("Node {} has derivatives, adding", lex_name);
                for (count, der) in derivatives.iter().enumerate() {
                    let der_id = format!("{}-autoderive-{}", lex_name, count);
                    let der_lex_raw = RawLexicalEntry{
                        etymology: Some(Etymology { 
                            etymons: vec![Edge{etymon: lex_name.to_string(), 
                            transforms: der.transforms.clone(),
                            agglutination_order: None}] }),
                        ..der.lexis.clone()
                    };
                    language_map.insert(der_id, der_lex_raw);
                }
            }

        }
        for (key, lex) in raw_graph.words {
            let found = language_map.insert(key.clone(), lex);
            if found.is_some() {
                return Err(anyhow!("Error: Key '{}' found multiple times", key));
            }
        }
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
        // if let Some(derivatives) = &node.derivatives {
        //     debug!("Node {} has derivatives, adding", lex_name);
        //     for (count, der) in derivatives.iter().enumerate() {
        //         let der_id = format!("{}-autoderive-{}", lex_name, count);
        //         let der_lex = Lexis{id: der_id, ..der.lexis.clone().into()};
        //         let der_transforms: Vec<Transform> = match &der.transforms{
        //             Some(t) => {find_transforms(t, &transform_map)?},
        //             None => Vec::new(),
        //         };
        //         tree.connect_etymology(der_lex, node_lex.clone(), der_transforms, None);
        //     }
        // };
        
       
    }

    Ok(tree)
}

fn add_single_word(tree: &mut LanguageTree, trans_map: &HashMap<String, RawTransform>, 
    lex_map: &HashMap<String, RawLexicalEntry>, node_lex: &Lexis, lex_ety: &Option<Etymology>) -> Result<()> {
        if let Some(etymon) = lex_ety{
            // iterate through all etymons associated with the base word, construct the transforms and add the etymology for each
            for e in &etymon.etymons{
                // fetch transform list
                let word_transforms = match &e.transforms {
                    Some(tf) =>  find_transforms(tf, trans_map)?,
                    None => vec![Transform{name: "loanword".into(), lex_match: None, transforms: vec![TransformFunc::Loanword]}]
                };
                let ety_lex: RawLexicalEntry = lex_map.get(&e.etymon).context(format!("etymon {} does not exist ", &e.etymon))?.clone();
                debug!("adding lex {} with etymon {}", node_lex.id, e.etymon);
                tree.connect_etymology(node_lex.clone(), Lexis { id: e.etymon.clone(), ..ety_lex.into()}, word_transforms, e.agglutination_order);
            }
        } else {
            debug!("Adding lex {} without etymology", node_lex.id);
            // connect_etymology checks for duplicates, add_lexis does not
            if !tree.contains(node_lex) {
                tree.add_lexis(node_lex.clone())
            }
            
        }
    Ok(())
}

/// Searches the Hashmap for the transform objects specified in trans_tree, or return defaults
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


#[cfg(test)]
mod tests {
    use crate::read_and_compute;
    use anyhow::Result;

    #[test]
    fn test_ingest_with_derivatives() -> Result<()> {
        let directory = Some(String::from("src/test_files/test_der"));
        let computed = read_and_compute(None, None, directory)?;
        let rendered_dict = computed.to_vec();

        assert_eq!(4, rendered_dict.len());
        Ok(())
    }

    #[test]
    fn test_repeated_keys()  {
        let directory = Some(String::from("src/test_files/repeated_keys"));
        let res = read_and_compute(None, None, directory);

        assert_eq!(true, res.is_err());

    }
}