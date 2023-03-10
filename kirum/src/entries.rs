use libkirum::{word::{Word, PartOfSpeech, Etymology}, kirum::Lexis};
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RawLexicalEntry {
    pub word: Option<Word>,
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
        Lexis { id: String::new(), 
            word: source.word, 
            language: source.language.unwrap_or("".to_string()), 
            pos: source.part_of_speech, 
            lexis_type: source.word_type.unwrap_or("".to_string()), 
            definition: source.definition, 
            archaic: source.archaic,
            tags: source.tags.unwrap_or(Vec::new())}
    }
}