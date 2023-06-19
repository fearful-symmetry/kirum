use std::collections::HashMap;

use libkirum::{word::{PartOfSpeech, Etymology}, kirum::Lexis, transforms::{TransformFunc, Transform}, matching::LexisMatch, lemma::Lemma};
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawTransform{
    pub transforms: Vec<TransformFunc>,
    pub conditional: Option<LexisMatch>
}


impl From<RawTransform> for Transform{
    fn from(value: RawTransform) -> Self {
        Transform { name: String::new(), lex_match: value.conditional, transforms: value.transforms}
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransformGraph {
    pub transforms: HashMap<String, RawTransform>
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawLexicalEntry {
    pub word: Option<Lemma>,
    #[serde(rename = "type", alias="lexis_type")]
    pub word_type: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub definition:String,
    #[serde(alias = "pos")]
    pub part_of_speech: Option<PartOfSpeech>,
    pub etymology: Option<Etymology>,
    #[serde(default = "default_archaic")]
    pub archaic: bool,
    pub tags: Option<Vec<String>>,

    /// Words that will be added as a derivative of the enclosing Lexis; any value not specified will be taken from the enclosing entry.
    pub derivatives: Option<Vec<Derivative>>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Derivative{
    pub lexis: RawLexicalEntry,
    pub transforms: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WordGraph {
    pub words: HashMap<String, RawLexicalEntry>,
}

fn default_archaic() ->bool{
    false
}

impl From<RawLexicalEntry> for Lexis{
    fn from(source: RawLexicalEntry) -> Self {
        Lexis { 
            id: String::new(),
            word: source.word, 
            language: source.language.unwrap_or("".to_string()), 
            pos: source.part_of_speech, 
            lexis_type: source.word_type.unwrap_or("".to_string()), 
            definition: source.definition, 
            archaic: source.archaic,
            tags: source.tags.unwrap_or(Vec::new())}
    }
}

impl From<Lexis> for RawLexicalEntry{
    fn from(value: Lexis) -> Self {
        RawLexicalEntry { word: value.word, 
            word_type: if !value.lexis_type.is_empty() {Some(value.lexis_type)} else {None}, 
            language: if !value.language.is_empty() {Some(value.language)} else {None}, 
            definition: value.definition, 
            part_of_speech: value.pos, 
            etymology: None, 
            archaic: value.archaic, 
            tags: if !value.tags.is_empty() {Some(value.tags)} else {None},
            derivatives: None,
        }
    }
}

// take the output of a call to to_vec_etymons() and structure it like a graph json file structure
pub fn create_json_graph(lex: Vec<(Lexis, Etymology)>) -> WordGraph{
    let mut graph: HashMap<String, RawLexicalEntry> = HashMap::new();

    for (word, ety) in lex{
        let base: RawLexicalEntry = word.clone().into();
        let complete = RawLexicalEntry{etymology: Some(ety), ..base};
        let key = format!("daughter-gen-{}", word.clone().word.unwrap().string_without_sep());
        graph.insert(key, complete);
    }
    WordGraph { words: graph }
}