use std::collections::HashMap;

use libkirum::{word::{PartOfSpeech, Etymology}, kirum::Lexis, transforms::{TransformFunc, Transform}, matching::LexisMatch, lemma::Lemma};
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug)]
/// Defines the transform structure as created by the user in JSON.
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
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
/// Defines a single lexis entry as created by the user in JSON
pub struct RawLexicalEntry {
    /// Optional word
    pub word: Option<Lemma>,
    /// Word type. Can be anything, purely for user tagging
    #[serde(rename = "type", alias="lexis_type")]
    pub word_type: Option<String>,
    /// Language name.
    pub language: Option<String>,
    /// Word definition
    #[serde(default)]
    pub definition:String,
    /// Part of speech
    #[serde(alias = "pos")]
    pub part_of_speech: Option<PartOfSpeech>,
    /// Etymology map
    pub etymology: Option<Etymology>,
    /// Defines a word as archaic. Purely for user tagging.
    #[serde(default = "default_archaic")]
    /// Optional user tagging
    pub archaic: bool,
    pub tags: Option<Vec<String>>,
    /// A tag that tells Kirum to generate the word based on the phonetic ruleset specified by the tag
    pub generate: Option<String>,
    /// Words that will be added as a derivative of the enclosing Lexis; any value not specified will be taken from the enclosing entry.
    pub derivatives: Option<Vec<Derivative>>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
/// The "derivative" field is largely semantic sugar, and allows the user to
/// define derivative words inside a given lexis entry.
pub struct Derivative{
    pub lexis: RawLexicalEntry,
    pub transforms: Option<Vec<String>>
}

/// Defines the "base" JSON file for a word tree.
#[derive(Serialize, Deserialize, Debug, Default, PartialEq)]
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
            tags: source.tags.unwrap_or(Vec::new()),
            word_create: source.generate
        }
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
            generate: value.word_create
        }
    }
}

/// take the output of a call to to_vec_etymons() and structure it like a graph json file structure
pub fn create_json_graph<F>(lex: Vec<(Lexis, Etymology)>,mut key_gen: F) -> WordGraph
    where F: FnMut(Lexis) -> String
    {
    let mut graph: HashMap<String, RawLexicalEntry> = HashMap::new();

    for (word, ety) in lex{
        let base: RawLexicalEntry = word.clone().into();
        let found_ety = if !ety.etymons.is_empty() {Some(ety)} else {None};
        let complete = RawLexicalEntry{etymology: found_ety, ..base};
        let key = key_gen(word);
        graph.insert(key, complete);
    }
    WordGraph { words: graph }
}