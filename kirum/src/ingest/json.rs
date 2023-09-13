
use std::path::Path;

use libkirum::{word::{Etymology, Edge}, lemma::Lemma};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::entries::{WordGraph, RawLexicalEntry};
use anyhow::Result;


#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Ingest {
    pub keys_are: KeyType,
    pub words: Vec<Value>
}

#[derive(Serialize, Copy, Deserialize, Clone, Debug, Default)]
pub enum KeyType {
    #[default]
    #[serde(alias="definitions")]
    Definitions,
    #[serde(alias="words")]
    Words
}

pub fn ingest<P: AsRef<Path>>(path: P, overrides: RawLexicalEntry) -> Result<WordGraph> {
    let raw = std::fs::read_to_string(path)?;
    let parsed: Ingest = serde_json::from_str(&raw)?;
    let mut working = WordGraph::default();
    for in_word in parsed.words {
        ingest_value(&overrides, parsed.keys_are, None, &mut working, in_word);
    };

    Ok(working)
}

fn ingest_value(overrides: &RawLexicalEntry,
    key_type: KeyType, 
    parent: Option<String>, 
    word_map: &mut WordGraph, val: Value){
    match val{
        Value::String(st) => {
            match parent{
                Some(p) => {
                    insert_into_map(overrides, key_type, Some(p), None, st, word_map);
                    //insert_string_with_parent(overrides, p, st, key_type, word_map);
                },
                None => {
                    insert_into_map(overrides, key_type, None, None, st, word_map);
                    //insert_single_string(overrides, st, key_type, word_map);
                }
            }
        },
        Value::Array(arr) => {
            for new_word in arr {
                ingest_value(overrides, key_type, parent.clone(), word_map, new_word);
            };
        },
        Value::Object(obj) => {
            let mut created_root = false;
            for (word_key, word_val) in obj {
                match word_val.clone() {
                    Value::String(found_string) => {
                        // skip !-directives that were read in by the parent call
                        if word_key.contains('!'){
                            continue;
                        }
                        // if we have a root map (no parent)
                        // then error out if we have an ! meta-string
                        // otherwise add the parent using a !etymology tag,
                        // or add any children in a word:!transform format, or parent:child format
                        match &parent {
                            Some(par_word) => {
                                // for a simple key:value, the value can either represent an etymology link to the parent,
                                // or another layer of depth in the word etymology 
                                match &found_string.strip_prefix('!') {
                                    Some(ety) => {
                                        insert_into_map(overrides, key_type, Some(par_word.clone()), Some(ety.to_string()), word_key.clone(), word_map);
                                    },
                                    None => {
                                        insert_into_map(overrides, key_type, Some(word_key.clone()), None, found_string.clone(), word_map);
                                    }
                                }
                                
                            },
                            None => {
                                // if there's no parent, we can only ingest a value of type parent:child
                                match &found_string.strip_prefix('!') {
                                    Some(bad) => {
                                        error!("found an etymology key '{}' but the object is root, so no etymology is possible", bad);
                                        continue;
                                    },
                                    None => {
                                        insert_into_map(overrides, key_type, Some(word_key.clone()), None, found_string.clone(), word_map);
                                    }
                                }

                            }
                        }
                    },
                    
                    Value::Object(child_obj) => {
                        // when recursing through child maps we need to "peek" into them to see if we
                        // have an !etymology tag, and insert the child value that way, rather then when we're 
                        // recursing in the child map

                        for (child_key, child_val) in child_obj{
                            if child_key == *"!etymology" {
                                if let Value::String(trans_name) = child_val {
                                    match &parent {
                                        Some(parent_word) => {
                                            insert_into_map(overrides, key_type, Some(parent_word.clone()), Some(trans_name), word_key.clone(), word_map);
                                            //insert_string_with_parent_transform(overrides, parent_word.clone(), trans_name, &word_key, key_type, word_map);
                                            created_root = true;
                                        },
                                        None => {
                                            error!("found a map with '!etymology' key and value {}, but at the root transform with no parent", trans_name);
                                            continue;
                                        }
                                    } 
                                }
                            }
                        }

                        ingest_value(overrides, key_type, Some(word_key.clone()), word_map, word_val);
                    },
                    _ => {
                        ingest_value(overrides, key_type, Some(word_key.clone()), word_map, word_val);
                    }
                }
                if !created_root {
                    ingest_value(overrides, key_type, parent.clone(), word_map, Value::String(word_key.clone()));
                }
            }

        },
        _ => {
            error!("objects can be strings, arrays, or maps; found other value");
        }
    };
}

/// do some logic to create an etymology value, then insert the new word into the WordTree
fn insert_into_map(overrides: &RawLexicalEntry, in_type: KeyType, parent: Option<String>, parent_transform: Option<String>, input_word: String, graph: &mut WordGraph) {
    let mut parent_ety: Option<Etymology> = None;
    // don't insert a transform unless we have a parent Lex
    if let Some(parent_val) = parent {
        let parent_id = format!("ingest-{}", parent_val);
        let mut new_edge = Edge{etymon: parent_id, ..Default::default()};
        if let Some(trans_id) = parent_transform{
            new_edge.transforms = Some(vec![trans_id])
        }
        parent_ety = Some(Etymology { etymons: vec![new_edge] });
    }
    let id = format!("ingest-{}", input_word);
    match in_type{
        KeyType::Definitions => {
            graph.words.insert(id, RawLexicalEntry{definition: input_word, etymology: parent_ety, ..overrides.clone()});
        },
        KeyType::Words => {
            let new_lemma: Lemma = input_word.into();
            graph.words.insert(id, RawLexicalEntry{word: Some(new_lemma), etymology: parent_ety, ..overrides.clone()});
        }
    }

}




#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use libkirum::word::{Etymology, Edge};
    use crate::entries::{RawLexicalEntry, WordGraph};
    use super::{ingest, ingest_value, KeyType};

    #[test]
    fn test_with_override() {
        let gen_statement = Some("example_generate".to_string());
        let test_over = RawLexicalEntry{generate: gen_statement.clone(), ..Default::default()};
        let mut new = WordGraph::default();
        let raw = r#"
        {
            "attack": ["attacking", "attacked"]
        }"#;
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        ingest_value(&test_over, KeyType::Definitions, None,  &mut new, parsed);

        println!("got: {:#?}", new);

        for (_, entry) in new.words{
            assert_eq!(entry.generate, gen_statement);
        }
    }

    #[test]
    fn basic_ingest_test() {

        let good_input =  WordGraph {
            words: HashMap::from([(
                "ingest-failure".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "failure".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-fail".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                    }),
                ("ingest-grab".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "grab".to_string(),
                    part_of_speech: None,
                    etymology: None,
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-fail".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "fail".to_string(),
                    part_of_speech: None,
                    etymology: None,
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-twistable".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "twistable".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-twist".to_string(),
                                    transforms: Some(
                                        vec![
                                            "capability".to_string(),
                                        ],
                                    ),
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-failing".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "failing".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-fail".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-unretwistable".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "unretwistable".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-retwistable".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-untwistable".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "untwistable".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-twistable".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-twist".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "twist".to_string(),
                    part_of_speech: None,
                    etymology: None,
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-retwistable".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "retwistable".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-twistable".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-attack".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "attack".to_string(),
                    part_of_speech: None,
                    etymology: None,
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-attacked".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "attacked".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-attack".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
                ("ingest-attacking".to_string(), RawLexicalEntry {
                    word: None,
                    word_type: None,
                    language: None,
                    definition: "attacking".to_string(),
                    part_of_speech: None,
                    etymology: Some(
                        Etymology {
                            etymons: vec![
                                Edge {
                                    etymon: "ingest-attack".to_string(),
                                    transforms: None,
                                    agglutination_order: None,
                                },
                            ],
                        },
                    ),
                    archaic: false,
                    tags: None,
                    generate: None,
                    derivatives: None,
                }),
            ]),
        };

        let path = "src/test_files/test_ingest/basic.json";
        let res = ingest(path, RawLexicalEntry::default()).unwrap();
        println!("got basic data: {:#?}", res);
        assert_eq!(res, good_input);
    }

}