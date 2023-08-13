use std::{path::PathBuf, io::Write, collections::HashMap, fs::{self, File}};
use libkirum::{transforms::TransformFunc, word::{Etymology, Edge}};
use crate::entries::{RawTransform, TransformGraph, RawLexicalEntry, Derivative, WordGraph};
use anyhow::Result;

/// Helper function. Create a new project, and write it out.
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
        generate: None,
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
                derivatives: None,
                generate: None,
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

