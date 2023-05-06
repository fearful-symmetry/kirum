use std::collections::HashMap;

use libkirum::{word::{Word, PartOfSpeech, Etymology}, kirum::Lexis, transforms::{TransformFunc, Transform}, matching::LexisMatch};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawTransform{
    pub transforms: Vec<TransformFunc>,
    pub conditional: Option<LexisMatch>
}

impl From<RawTransform> for Transform{
    fn from(value: RawTransform) -> Self {
        Transform { name: String::new(), lex_match: value.conditional, transforms: value.transforms, agglutination_order: None }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawLexicalEntry {
    pub word: Option<Word>,
    #[serde(rename = "type")]
    pub word_type: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub definition:String,
    pub part_of_speech: Option<PartOfSpeech>,
    pub etymology: Option<Etymology>,
    #[serde(default = "default_archaic")]
    pub archaic: bool,
    pub tags: Option<Vec<String>>
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
        // TODO: don't blindly wrap in Some() statements, actually check contents
        RawLexicalEntry { word: value.word, 
            word_type: Some(value.lexis_type), 
            language: Some(value.language), 
            definition: value.definition, 
            part_of_speech: value.pos, 
            etymology: None, 
            archaic: value.archaic, 
            tags: Some(value.tags) }
    }
}

// take the output of a call to to_vec_etymons() and structure it like a graph json file structure
pub fn create_json_graph(lex: Vec<(Lexis, Etymology)>) -> HashMap<String, RawLexicalEntry>{
    let mut graph: HashMap<String, RawLexicalEntry> = HashMap::new();

    for (word, ety) in lex{
        let base: RawLexicalEntry = word.clone().into();
        let complete = RawLexicalEntry{etymology: Some(ety), ..base};
        let key = format!("daughter-gen-{}", word.clone().word.unwrap().to_string());
        graph.insert(key, complete);
    }
    graph
}