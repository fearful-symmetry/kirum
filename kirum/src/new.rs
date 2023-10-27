use std::{path::PathBuf, io::Write, collections::HashMap, fs::{self, File}};
use libkirum::{transforms::TransformFunc, word::{Etymology, Edge}, lexcreate::LexPhonology};
use crate::{entries::{RawTransform, TransformGraph, RawLexicalEntry, Derivative, WordGraph}, global::Global};
use anyhow::{Result, Context, anyhow};

pub fn create_project_directory(name: &str) -> Result<()>{
    let base = PathBuf::from(name);
    let ety_path = base.join("etymology");
    let tree_path = base.join("tree");
    let phonetic_path = base.join("phonetics");
    fs::create_dir_all(ety_path)?;
    fs::create_dir_all(tree_path)?;
    fs::create_dir_all(phonetic_path)?;
    Ok(())
}

/// Helper function. Create a new project, and write it out.
pub fn create_new_project(name: &str) -> Result<()> {
    let base = PathBuf::from(name);
    let mut ety_path = base.join("etymology");
    let mut tree_path = base.join("tree");
    let mut phonetic_path = base.join("phonetics");
    create_project_directory(name).context("error creating project directory")?;

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
        historical_metadata: None,
        derivatives: None, 
        generate: None,
    });
    word_map.insert("latin_example".into(), RawLexicalEntry { 
        word: None, 
        word_type: Some("word".into()), 
        historical_metadata: None,
        language: Some("Latin".into()), 
        definition: "an instance, model, example".into(), 
        part_of_speech: Some(libkirum::word::PartOfSpeech::Noun), 
        etymology: Some(Etymology { etymons: vec![Edge{etymon: "latin_verb".into(), transforms: Some(vec!["latin-from-verb".into()]), agglutination_order: None}] }), 
        archaic: true, 
        tags: Some(vec!["example".into(), "default".into()]), 
        generate: None,
        derivatives: Some(vec![Derivative{lexis: RawLexicalEntry { 
                word: None, 
                word_type: None, 
                language: Some("Old French".into()), 
                definition: "model, example".into(), 
                part_of_speech: Some(libkirum::word::PartOfSpeech::Noun), 
                etymology: None, 
                archaic: true, 
                tags: None, 
                historical_metadata: None,
                derivatives: None,
                generate: None,
            },
            transforms: Some(vec!["of-from-latin".to_owned()]),
    }]) 
    });

    let example_tree = WordGraph{
        words: word_map
    };

    let example_phonetics = LexPhonology{
        groups: HashMap::from([
            ('C', vec!["x".try_into()?, "m".try_into()?, "p".try_into()?, "l".try_into()?,]),
            ('V', vec!["e".try_into()?, "a".try_into()?]),
            ('S', vec!["VC".try_into()?, "CCV".try_into()?])
        ]),
        lexis_types: HashMap::from([
            ("word".into(), vec!["SSS".try_into()?])
        ])
    };

    let phonetic_data = serde_json::to_string_pretty(&example_phonetics)?;
    let graph_data = serde_json::to_string_pretty(&example_tree)?;
    let trans_data = serde_json::to_string_pretty(&example_transforms)?;

    let name_path: PathBuf = name.parse()?;
    let file_name = name_path.file_name()
    .ok_or_else(|| anyhow!("could not extract final path from {}", name_path.display()))?.to_string_lossy();
    write_json(&file_name, &mut tree_path, graph_data).context(format!("error writing {}", file_name))?;
    write_json("ety", &mut ety_path, trans_data).context("error writing ety file")?;
    write_json("rules", &mut phonetic_path, phonetic_data).context("error writing rules file")?;

    let base_globals = Global{transforms: None};
    let globals_data = serde_json::to_string_pretty(&base_globals)?;
    let mut globals_file = File::create(base.join("globals.json")).context("could not create globals file")?;
    write!(globals_file, "{}", globals_data).context("error writing globals file")?;
 
    Ok(())
}

fn write_json(subpath: &str, base_path: &mut PathBuf, data: String) -> Result<()>{
    base_path.push(subpath);
    base_path.set_extension("json");
    let mut phonetics_file = File::create(base_path.clone())
    .context(format!("could not create  json file {} {}", subpath, base_path.display()))?;

    write!(phonetics_file, "{}", data)
    .context("error writing phonetics file")?;

    Ok(())
}

