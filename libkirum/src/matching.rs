use serde::{Deserialize, Serialize};
use crate::kirum::Lexis;
use crate::lemma::Lemma;
use crate::word::PartOfSpeech;


/// A match value that can be used to evaluate if a given Lexis field matches a predicate.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value{
    #[serde(rename="not")]
    Not(ValueMatch),
    #[serde(rename="match")]
    Match(ValueMatch)
}

impl From<String> for Value{
    fn from(value: String) -> Self {
        Value::Match(ValueMatch::Equals(EqualValue::String(value)))
    }
}

impl Value{
    pub fn is_true<T>(&self, val: &T)-> bool
    where
    ValueMatch: PartialEq<T>
     {
        match self{
            Self::Match(v) =>{
                *v == *val
            }, 
            Self::Not(v) =>{
                ! (*v == *val)
            }
        }
    }
}

/// Defines an equality in a match statement
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum EqualValue{
    String(String),
    Vector(Vec<String>)
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ValueMatch{
    #[serde(rename="equals")]
    Equals(EqualValue),
    #[serde(rename="oneof")]
    OneOf(Vec<String>)
}

impl PartialEq<String> for ValueMatch{
    fn eq(&self, other: &String) -> bool {
        match self {
            Self::Equals(v) => {
                match v {
                    EqualValue::Vector(_) => false,
                    EqualValue::String(s) => { s == other}
                }
            },
            Self::OneOf(a) => {
                a.contains(other)
            }
        }
    }

}

// impl PartialEq<Word> for ValueMatch{
//     fn eq(&self, other: &Word) -> bool {
//         match other {
//             Word::Letters(l) => {*self == *l},
//             Word::String(s) => {*self == *s}
//         }
//     }
// }


impl PartialEq<Lemma> for ValueMatch{
    fn eq(&self, other: &Lemma) -> bool {
        *self == other.string_without_sep()
    }
}

impl PartialEq<PartOfSpeech> for ValueMatch{
    fn eq(&self, other: &PartOfSpeech) -> bool {
        *self == other.to_string()
    }
}

impl PartialEq<Vec<std::string::String>> for ValueMatch{
    fn eq(&self, other: &Vec<std::string::String>) -> bool {
        match self {
            Self::OneOf(lst) => {
               lst.iter().any(|i| other.contains(i))
            },
            Self::Equals(lst) => {
                match lst {
                    EqualValue::Vector(v) => v.iter().all(|i| other.contains(i)),
                    EqualValue::String(_) => false,
                }
                
            }
        }
    }
}

/// A matching object that can be used to evaluate if the selected predicates match a supplied Lexis
#[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
pub struct LexisMatch{
    pub id: Option<Value>,
    pub word: Option<Value>,
    pub language: Option<Value>,
    pub pos: Option<Value>,
    #[serde(alias="type")]
    pub lexis_type: Option<Value>,
    pub archaic: Option<bool>,
    pub tags: Option<Value>
}

impl LexisMatch {
    /// determine if the Match object matches the supplied Lexis
    pub fn matches(&self, lex: &Lexis) -> bool{
            self == lex
    }
}

fn value_matches<T>(val: &Option<Value>, to_match: &T) -> bool
    where
    ValueMatch: PartialEq<T>
    {
    if let Some(v) = val {
        v.is_true(to_match)
    } else{
        true
    }
}

impl PartialEq<Lexis> for LexisMatch{
    fn eq(&self, other: &Lexis) -> bool {
        value_matches(&self.tags, &other.tags) &
        if let Some(word) = &other.word{value_matches(&self.word, word)} else{true} &
        value_matches(&self.language, &other.language) &
        if let Some(pos) = other.pos{value_matches(&self.pos, &pos)} else{true} &
        value_matches(&self.lexis_type, &other.lexis_type) &
        if let Some(a) = self.archaic{a == other.archaic} else{true}
        
    }
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EtymonMatch{
    #[serde(rename="all")]
    All(LexisMatch),
    #[serde(rename="one")]
    One(LexisMatch)
}

impl PartialEq<Vec<Lexis>> for EtymonMatch {
    fn eq(&self, other: &Vec<Lexis>) -> bool {
        match self {
            EtymonMatch::All(lm) => {
                other.iter().all(|i| *lm == *i)
            }
            EtymonMatch::One(lm) => {
                other.iter().any(|i| *lm == *i)
            }
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WhenMatch{
    /// Before will match a lexis before it has been transformed by any other non-global transforms
    #[serde(alias="before")]
    Before,
    /// After will match a lexis after a word has been generated for that lexis
    #[serde(alias="after")]
    After
}


#[cfg(test)]
mod tests {

    use crate::errors::LangError;
    use crate::kirum::Lexis;
    use crate::matching::{Value, ValueMatch, LexisMatch, EqualValue};


    #[test]
    fn test_lexis_match()-> Result<(), LangError>{
        let test_lexis = Lexis{
        id: String::new(),
        word: Some("kirum".into()), 
        lexis_type: "".to_string(),
        language: "Old Babylonian".to_string(),
        pos: None,
        definition: "".to_string(),
        archaic: false,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        word_create: None
        }; 

        let test_match = LexisMatch{
            id: None,
            word: None,
            language: Some(Value::Match(ValueMatch::Equals(EqualValue::String("Old Babylonian".to_string())))),
            pos: None,
            archaic: Some(false),
            lexis_type: None,
            tags: Some(Value::Match(ValueMatch::OneOf(vec!["tag1".to_string(), "tag3".to_string()])))
        };
        assert_eq!(test_match == test_lexis, true);
        Ok(())
    }
    #[test]
    fn test_lexis_tags()-> Result<(), LangError> {
        let test_lexis = Lexis{tags: vec!["tag1".to_string(), "tag2".to_string()], ..Default::default()};
        let tags_all = LexisMatch{
            tags: Some(Value::Match(ValueMatch::Equals(EqualValue::Vector(vec!["tag1".to_string(), "tag2".to_string()])))),
            ..Default::default()
        };
        assert_eq!(tags_all == test_lexis, true);

        let tags_not_all = LexisMatch{
            tags: Some(Value::Not(ValueMatch::Equals(EqualValue::Vector(vec!["tag3".to_string(), "tag4".to_string()])))),
            ..Default::default()
        };
        assert_eq!(tags_not_all == test_lexis, true);

        let tags_not_oneof = LexisMatch{
            tags: Some(Value::Not(ValueMatch::OneOf(vec!["tag3".to_string(), "tag4".to_string()]))),
            ..Default::default()
        };
        assert_eq!(tags_not_oneof == test_lexis, true);
        Ok(())
    }
}