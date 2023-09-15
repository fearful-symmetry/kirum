use std::{path::{PathBuf, Path},  collections::HashMap, fs::File, io::Write};
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{LanguageTree, Lexis}, transforms::{Transform, TransformFunc}, word::{Etymology, Edge}, lexcreate::LexPhonology};
use serde::Serialize;
use walkdir::{WalkDir, DirEntry};
use crate::entries::{RawTransform, RawLexicalEntry, TransformGraph, WordGraph};
use handlebars::Handlebars;

/// contains path data for everything needed for a project
pub struct Project {
    pub graphs: Vec<PathBuf>,
    pub transforms: Vec<PathBuf>,
    pub phonetic_rules: Option<Vec<PathBuf>>
}

/// renders any templating code that was written into word definitions
pub fn apply_def_vars(var_file: Option<String>, dict: &mut Vec<Lexis>) -> Result<()> {
    if let Some(vars) = var_file {
        debug!("Applying variables from {}", vars);
        let vars_toml = std::fs::read_to_string(vars)?;
                
        let vars: HashMap<String, String> = toml::from_str(&vars_toml)?;
    
        for word in dict {
            let mut handlebars = Handlebars::new();
            handlebars.register_template_string("def", &word.definition)?;
            let updated = handlebars.render("def", &vars)?;
            word.definition = updated;
        }
    }
    Ok(())
}

/// read a list of tree and transform files, return the raw Language Tree Object
pub fn read_from_files(proj: Project) -> Result<LanguageTree>{
    //first merge all the files into one giant hashmap for the transforms and graph
    // because we later need to get random words from the map to construct the etymology from the rawLex "etymology" fields,
    // the giant hashmaps of everything need to be made first
    let transform_map = read_transform_files(&proj.transforms)?;

    let language_map = read_tree_files(&proj.graphs)?;
    
    if language_map.is_empty(){
        return Err(anyhow!("specified language tree does not contain any data. Tree files used: {:?}", proj.graphs));
    }

    let mut tree = LanguageTree::new();
    if let Some(phonetic_files) = proj.phonetic_rules{
        tree.word_creator_phonology = create_phonetics(phonetic_files)?;
    }



    for (lex_name, node) in &language_map{
        debug!("creating node entry {}", lex_name);
        let node_lex: Lexis = Lexis { id: lex_name.to_string(), ..node.clone().into() };
        add_single_word(&mut tree, &transform_map, &language_map, &node_lex, &node.etymology)?; 
    }

    Ok(tree)
}

pub fn read_tree_files(files: &Vec<PathBuf>) -> Result<HashMap<String, RawLexicalEntry>> {
    let mut language_map: HashMap<String, RawLexicalEntry> = HashMap::new();
    for lang_file in files{
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

    Ok(language_map)
}

pub fn read_transform_files(files: &Vec<PathBuf>) -> Result<HashMap<String, RawTransform>> {
    let mut transform_map: HashMap<String, RawTransform> = HashMap::new();
    for trans_file in files {
        let trans_raw = std::fs::read_to_string(trans_file.clone()).context(format!("error reading etymology file {}", trans_file.display()))?;
        let transforms: TransformGraph = serde_json::from_str(&trans_raw).context(format!("error parsing etymology file {}", trans_file.display()))?;
        debug!("read in transform file: {}", trans_file.display());
        transform_map.extend(transforms.transforms);
    };

    Ok(transform_map)
}

/// Add a single word entry to the tree, including any derivative words
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

/// Generate phonology structures from files
pub fn create_phonetics(paths: Vec<PathBuf>) -> Result<LexPhonology>{
    let mut phonetic_set = LexPhonology::default();
    for path in paths{
        let raw = std::fs::read_to_string(&path)?;
        let parsed: LexPhonology = serde_json::from_str(&raw)?;
        phonetic_set.groups.extend(parsed.groups);
        phonetic_set.lexis_types.extend(parsed.lexis_types);
    }

    Ok(phonetic_set)
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

/// Traverse a directory, returning a list of transforms and graph files
pub fn handle_directory(path: &str) -> Result<Project> {
    let lang_dir = Path::new(path);
    let lang_graph_dir = lang_dir.join("tree");
    let lang_transform_dir = lang_dir.join("etymology");
    let phonetics_path = lang_dir.join("phonetics");

    debug!("using tree path: {}", lang_graph_dir.display());
    let  graphs: Vec<PathBuf> = read_subdir_create_list(lang_graph_dir)?;

    debug!("using etymology path: {}", lang_transform_dir.display());
    let  transforms: Vec<PathBuf> = read_subdir_create_list(lang_transform_dir)?;

    debug!("using phonetics path: {}", phonetics_path.display());
    
    let phonetic_rules: Option<Vec<PathBuf>> = if phonetics_path.exists(){
        Some(read_subdir_create_list(phonetics_path)?)
    } else {
        None
    };
    

    Ok(Project { graphs, 
        transforms, 
        phonetic_rules})
}

fn read_subdir_create_list(path: PathBuf) -> Result<Vec<PathBuf>>{
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_entry(check_path){
        let found_path = entry?.path().to_path_buf();
        if !found_path.is_dir(){
            paths.push(found_path);
        }
    }

    Ok(paths)
}

/// check if the path is a valid file we want to read
fn check_path(dir: &DirEntry) -> bool {
    debug!("checking path: {:?}", dir);
    if dir.file_type().is_dir(){
        true
    } else  {
        dir.path().extension().unwrap_or_default() == "json"
    }
    
}

/// read in the existing files and generate a graph
/// deals with the logic of listed files versus a specified directory
pub fn read_and_compute(directory: Option<String>) -> Result<LanguageTree>{
    let new_project: Project = if directory.is_some(){
        handle_directory(&directory.unwrap())?
    } else {
        return Err(anyhow!("must specify either a graph and transform file, or a directory"));
    }; 
    info!("Reading in existing language files...");
    let mut lang_tree = read_from_files(new_project)?;
    info!("rendering tree...");
    lang_tree.compute_lexicon();
    Ok(lang_tree)
}

/// add a tree file to the existing directory
pub fn add_tree_file<P: AsRef<Path>>(path: P, name: &str, data: WordGraph) -> Result<()> {
    let write_to = Path::new(path.as_ref()).join("tree").join(name);
    add_file(&write_to, data).context(format!("error adding file {} to project", write_to.display()))?;
    Ok(())
}

pub fn add_ety_file<P: AsRef<Path>>(path: P, name: &str, data: TransformGraph) -> Result<()> {
    let write_to = Path::new(path.as_ref()).join("etymology").join(name);
    add_file(&write_to, data).context(format!("error adding file {} to project", write_to.display()))?;
    Ok(())
}

/// add a file to the project at the specified path
pub fn add_file<P, S>(path: P, data: S) -> Result<()>
where
    P: AsRef<Path> + std::fmt::Debug,
    S: Sized + Serialize
    {
        let mut file = File::create(&path)
        .context(format!("error creating file {:?}", path))?;
    
        let graph_data = serde_json::to_string_pretty(&data)
                .context("error creating JSON from graph")?;
    
        write!(file, "{}", graph_data)?;
        Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use anyhow::Result;
    use libkirum::{kirum::Lexis, lexcreate::LexPhonology};

    use crate::files::read_and_compute;

    use super::apply_def_vars;

    #[test]
    fn test_phonetic_ingest() -> Result<()>{
        let raw = std::fs::read_to_string("src/test_files/test_phonetics.json")?;
        let parsed: LexPhonology = serde_json::from_str(&raw)?;

        let example = LexPhonology{
            groups: HashMap::from([
                ('C', vec!["r".try_into()?, "k".try_into()?, "c".try_into()?, "ch".try_into()?, "b".try_into()?]),
                ('V', vec!["i".try_into()?, "u".try_into()?, "o".try_into()?]),
                ('S', vec!["CV".try_into()?, "CVC".try_into()?, "VC".try_into()?])
            ]),
            lexis_types: HashMap::from([
                ("word".into(), vec!["S".try_into()?, "SuS".try_into()?, "iSSS".try_into()?, "SSSS".try_into()?])
            ])
        };

        assert_eq!(example, parsed);

        Ok(())
    }

    #[test]
    fn test_phonetic_create() -> Result<()> {
        let dir = Some(String::from("src/test_files/with_phonetic_rules"));
        let computed = read_and_compute(dir)?;
        let rendered: Vec<Lexis> = computed.to_vec().into_iter().filter(|w| w.word.is_some()).collect();
        println!("Got: {:?}", rendered);
        assert_eq!(3, rendered.len());

        Ok(())
    }

    #[test]
    fn test_ingest_with_derivatives() -> Result<()> {
        let directory = Some(String::from("src/test_files/test_der"));
        let computed = read_and_compute(directory)?;
        let rendered_dict = computed.to_vec();

        assert_eq!(4, rendered_dict.len());
        Ok(())
    }

    #[test]
    fn test_def_templates() -> Result<()> {
        let vars = Some(String::from("src/test_files/test_tmpl_vars.toml"));
        let example_lex = Lexis{definition: String::from("a word in {{ln}}"), ..Default::default()};
        let mut dict = vec![example_lex];

        apply_def_vars(vars, &mut dict)?;

        assert_eq!("a word in test_lang".to_string(), dict[0].definition);

        Ok(())
    }

    #[test]
    fn test_repeated_keys()  {
        let directory = Some(String::from("src/test_files/repeated_keys"));
        let res = read_and_compute(directory);

        assert_eq!(true, res.is_err());
    }
}