use serde::{Deserialize, Serialize};
use crate::transforms::TransformFunc;
use crate::kirum::Lexis;
use crate::word::{Word, PartOfSpeech};


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Value{
    #[serde(rename="not")]
    Not(ValueMatch),
    #[serde(rename="match")]
    Match(ValueMatch)
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

    fn ne(&self, other: &String) -> bool {
        ! self.eq(other)
    }
}

impl PartialEq<Word> for ValueMatch{
    fn eq(&self, other: &Word) -> bool {
        *self == other.to_string()
    }
    fn ne(&self, other: &Word) -> bool {
        ! self.eq(other)
    }
}

impl PartialEq<PartOfSpeech> for ValueMatch{
    fn eq(&self, other: &PartOfSpeech) -> bool {
        *self == other.to_string()
    }
    fn ne(&self, other: &PartOfSpeech) -> bool {
        ! self.eq(other)
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

fn value_matches<T>(val: &Option<Value>, to_match: &T) -> bool
    where
    ValueMatch: PartialEq<T>
    {
    if let Some(v) = val {
        if v.is_true(to_match){
            true
        } else{
            false
        }
    } else{
        true
    }
}

impl PartialEq<Lexis> for LexisMatch{
    fn eq(&self, other: &Lexis) -> bool {
        if let Some(word) = &other.word {value_matches(&self.word, word);} else{true;} &
        value_matches(&self.language, &other.language) &
        if let Some(pos) = other.pos{value_matches(&self.pos, &pos)} else{true} &
        value_matches(&self.lexis_type, &other.lexis_type) &
        if let Some(a) = self.archaic{a == other.archaic} else{true} & 
        value_matches(&self.tags, &other.tags)
    }
}

impl Default for LexisMatch {
    fn default() -> Self {
        LexisMatch { id: None, word: None, language: None, pos: None, lexis_type: None, archaic: None, tags: None }
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
pub struct Match{
    pub lexis: LexisMatch,
    pub etymon: EtymonMatch
}

impl Match {
    pub fn matches(self, lex: Lexis, etymons: Vec<Lexis>) -> bool{
            self.lexis == lex && self.etymon == etymons
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalMatch{
    pub matches: Match,
    pub transform: Vec<TransformFunc>,
    pub when: WhenMatch
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum WhenMatch{
    /// Before will match a lexis before it has been transformed by any other non-global transforms
    #[serde(alias="before")]
    Before,
    /// After will match a lexis after a word has been genterated for that lexis
    #[serde(alias="after")]
    After
}

// pub fn global_matches_from_file(filepath: String) -> Result<Vec<GlobalMatch>, LangError>{
//     let match_raw = std::fs::read_to_string(filepath).map_err(LangError::JSONImportError)?;
//     let matches: Vec<GlobalMatch> = from_str(&match_raw).map_err(LangError::JSONSerdeError)?;

//     Ok(matches)
// }


#[cfg(test)]
mod tests {

    use serde_json::from_str;

    use crate::errors::LangError;
    use crate::kirum::Lexis;
    use crate::matching::{Value, ValueMatch, EtymonMatch, LexisMatch, EqualValue};

    use super::GlobalMatch;
   // use crate::matching::global_matches_from_file;

    pub fn global_matches_from_file(filepath: String) -> Result<Vec<GlobalMatch>, LangError>{
        let match_raw = std::fs::read_to_string(filepath).map_err(LangError::JSONImportError)?;
        let matches: Vec<GlobalMatch> = from_str(&match_raw).map_err(LangError::JSONSerdeError)?;
    
        Ok(matches)
    }

    #[test]
    fn test_file_ingest()->Result<(), LangError>{

        let matches = global_matches_from_file("src/example_files/example_global.json".to_string())?;
        let main_match = matches[0].clone();

        let match_lexis = main_match.matches.lexis;
        assert_eq!(match_lexis.lexis_type, Some(Value::Match(ValueMatch::Equals(EqualValue::String("stem".to_string())))));
        assert_eq!(match_lexis.tags, Some(Value::Match(ValueMatch::OneOf(vec!["genitive".to_string()]))));
        assert_eq!(match_lexis.pos, Some(Value::Not(ValueMatch::Equals(EqualValue::String("noun".to_string())))));

        let match_etymon = main_match.matches.etymon;
        assert_eq!(match_etymon, EtymonMatch::All(LexisMatch{id: None, word: None, language: None, pos: None, lexis_type: Some(Value::Match(ValueMatch::Equals(EqualValue::String("root".to_string())))), archaic: None, tags: None}));


        for word_match in matches{
            println!("Got match: {:?}", word_match);
        };

        Ok(())
    }

    #[test]
    fn test_lexis_match()-> Result<(), LangError>{
        let test_lexis = Lexis{
        word: Some("kirum".into()), 
        lexis_type: "".to_string(),
        language: "Old Babylonian".to_string(),
        pos: None,
        definition: "".to_string(),
        archaic: false,
        tags: vec!["tag1".to_string(), "tag2".to_string()]
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