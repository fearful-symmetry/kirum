use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::errors;

/// The possible Part Of Speech values for a Lexis
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum PartOfSpeech {
    #[serde(rename(deserialize= "noun", serialize="noun"))]
    Noun,
    #[serde(rename(deserialize="verb", serialize="verb"))]
    Verb,
    #[serde(rename(deserialize= "adjective", serialize="adjective"))]
    Adjective,
}

impl FromStr for PartOfSpeech {
    type Err = errors::POSFromError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_ref() {
            "noun" => Ok(Self::Noun),
            "verb" => Ok(Self::Verb),
            "adjective" => Ok(Self::Adjective),
            "adj" => Ok(Self::Adjective),
            _ => Err(errors::POSFromError { found: s.to_string() })
        }
    }
}


impl std::string::ToString for PartOfSpeech{
    fn to_string(&self) -> String {
        match self{
            Self::Adjective => "adjective".to_string(),
            Self::Noun => "noun".to_string(),
            Self::Verb => "verb".to_string()
        }
    }
}

/// The etymology of a given lexis.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Etymology{
    pub etymons: Vec<Edge>,
}

/// The edge of the tree graph, containing a reference to the upstream word, and other metadata.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Edge {
    pub etymon: String,
    pub transforms: Option<Vec<String>>,
    pub agglutination_order: Option<i32>
}
