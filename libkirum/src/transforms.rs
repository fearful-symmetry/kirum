use std::fmt::Display;

use rhai::{Dynamic, Scope};
use serde::{Deserialize, Serialize};
use crate::{errors::TransformError, kirum::Lexis, lemma::Lemma, matching::LexisMatch};
use log::{debug, trace};

/// Specifies a transform at a global level. Global transforms don't have a name, but can be matched to both the target lexis, and the etymon.
#[derive(Clone, Default, Debug)]
pub struct GlobalTransform {
    /// Match statement for the word under transform
    pub lex_match: LexisMatch,
    /// Optional match statement for the lexis's etymon
    /// If a given word has multiple upstream etymons, libkirum will look for any matching etymon.
    pub etymon_match: Option<LexisMatch>,
    pub transforms: Vec<TransformFunc>
}

impl GlobalTransform {
    ///  Transform the given lexis, or return the original unaltered lexis if the specified lexii don't meet the match statements
    pub fn transform(&self,  lex: &mut Lexis, etymon: Option<&Vec<&Lexis>>) -> Result<(), TransformError> {
        // check to see if the etymon should allow us to transform
        let should_trans = if let Some(ety) = etymon  {
            if let Some(ety_match) = &self.etymon_match  {
                ety.iter().find(|&e| ety_match.matches(e)).is_some()
            } else {
                true
            }
        } else {
            true
        };
        
        trace!("checking global transforms for {}", lex.id);
        if self.lex_match.matches(lex) && should_trans{
            trace!("applying global transforms to {}", lex.id);
            for trans in &self.transforms {
                trans.transform(lex)?
            }
        };
        Ok(())
    }
}

/// Defines a series of transforms that are applied to a lexis.
#[derive(Clone, Default)]
pub struct Transform {
    pub name: String,
    pub lex_match: Option<LexisMatch>,
    pub transforms: Vec<TransformFunc>,
   //pub agglutination_order: Option<i32>,
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}

impl Transform{
    /// Transform the given lexis, or return the original unaltered lexis if the lex_match resolves to false.
    pub fn transform(&self, etymon: &mut Lexis) -> Result<(), TransformError> {
        self.transform_option(etymon)?;
        Ok(())
    }

    // Transform the given lexis, or return None if the lex_match condition evaluates to false
    pub fn transform_option(&self, etymon: &mut Lexis) -> Result<bool, TransformError> {
        let can_transform = if let Some(lex_match) = &self.lex_match{
            lex_match.matches(etymon)
        } else {
            true
        };
        //let mut updated = etymon.clone();
        if can_transform{
            for transform in &self.transforms {
                transform.transform(etymon)?; 
            };
            Ok(true)
        } else{
            Ok(false)
        }
    }
}

 
 /// Defines all the possible transforms that can be applied to a Lexis
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransformFunc {
    /// replaces one specified letter with another
    #[serde(rename="letter_replace")]
    LetterReplace{letter: LetterValues, replace: LetterPlaceType},
    /// transforms the Lemmma based on an array that can either 
    /// map a given character in a lemma, or specify a hard-coded character.
    /// For example, a LetterArray vector of [0 a 1 u 3]
    /// applied to the letter 'example' would return "eaxum".
    #[serde(rename="letter_array")]
    LetterArray{letters: Vec<LetterArrayValues>},
    /// Apply a value to the end of a word
    #[serde(rename="postfix")]
    Postfix{value: Lemma},
    /// Apply a value to the start of a word
    #[serde(rename="prefix")]
    Prefix{value: Lemma},
    /// Apply no transforms
    #[serde(rename="loanword")]
    Loanword,
    /// remove the specified letter
    #[serde(rename="letter_remove")]
    LetterRemove{letter: String, position: LetterPlaceType},
    /// double a given letter
    #[serde(rename="double")]
    Double{letter: String, position: LetterPlaceType},
    /// remove a doubled letter
    #[serde(rename="dedouble")]
    DeDouble{letter: String, position: LetterPlaceType},
    /// replace a matching substring
    #[serde(rename="match_replace")]
    MatchReplace{old: Lemma, new: Lemma},

    /// Transform a word using an rhai file.
    /// The rhai script should return a string of the updated word
    #[serde(rename="rhai_script")]
    RhaiScript{file: String}
}

impl Display for TransformFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformFunc::LetterReplace { letter, replace: _ } => {
                write!(f, "LetterReplace ({:?})", letter)
            },
            TransformFunc::Postfix { value } => {
                write!(f, "Postfix ({})", value.to_string())
            },
            TransformFunc::Prefix { value } => {
                write!(f, "Prefix ({})", value.to_string())
            },
            TransformFunc::Loanword => {
                write!(f, "Loanword")
            },
            TransformFunc::LetterRemove { letter, position:  _ } => {
                write!(f, "LetterRemove ({})", letter)
            },
            TransformFunc::Double { letter, position: _ } => {
                write!(f, "Double ({})", letter)
            },
            TransformFunc::DeDouble { letter, position: _ } => {
                write!(f, "DeDouble ({})", letter)
            },
            TransformFunc::MatchReplace { old, new } => {
                write!(f, "MatchReplace ({} > {})", old.to_string(), new.to_string())
            },
            TransformFunc::RhaiScript { file } => {
                write!(f, "RhaiScript ({})", file)
            },
            TransformFunc::LetterArray { letters } => {
                write!(f, "LetterArray ({:?})", letters)
            }
        }
    }
}


impl TransformFunc{
    pub fn transform(&self, current_word: &mut Lexis) -> Result<(), TransformError> {
        if current_word.word.is_none(){
            return Ok(())
        }
        if let Some(current) = current_word.word.as_mut() {
            match self {
                TransformFunc::LetterReplace{ letter, replace } => {
                   current.replace(&letter.old, &letter.new, replace);
                   debug!("got LetterReplace:{:?} ({:?}) for {}; updated: {}", replace, letter, current_word.id, &current.string_without_sep());
                },
                TransformFunc::LetterArray { letters } => {
                    debug!("got LetterArray ({:?}) for {}", letters, current_word.id);
                    current.modify_with_array(letters) 
                },
                TransformFunc::Postfix { value } => {
                    debug!("got Postfix for {}", current_word.id);
                    current.add_postfix(value)
                },
                TransformFunc::Prefix { value } => {
                    debug!("got Prefix for {}", current_word.id);
                    current.add_prefix(value)
    
                },
                TransformFunc::Loanword => {
                    debug!("got Loanword for {}", current_word.id);
                },
                TransformFunc::LetterRemove {letter, position } =>{
                    debug!("got LetterRemove for {}", current_word.id);
                    current.remove_char(letter, position)
                },
                TransformFunc::Double { letter, position } => {
                    debug!("got Double for {}", current_word.id);
                    current.double(letter, position)
                },
                TransformFunc::DeDouble { letter, position } => {
                    debug!("got DeDouble for {}", current_word.id);
                    current.dedouble(letter, position)
                },
                TransformFunc::MatchReplace { old, new } => {
                    current.match_replace(old, new)
                },
                TransformFunc::RhaiScript { file } => {
                    let engine = rhai::Engine::new();
                    let mut scope = Scope::new();

                    let lemma_array: Dynamic = current.clone().into();
                    let tags_array: Dynamic = current_word.tags.clone().into();
                    let metadata_object: Dynamic = current_word.historical_metadata.clone().into();

                    scope.push("language", current_word.language.clone());
                    scope.push("tags", tags_array);
                    scope.push("metadata", metadata_object);
                    scope.push("pos", current_word.pos.unwrap_or_default().to_string());
                    scope.push("lemma_array", lemma_array);
                    scope.push("lemma_string", current.clone().string_without_sep());

                    let updated: Lemma = engine.eval_file_with_scope::<Dynamic>(&mut scope, file.into())?.try_into()?;
                    *current = updated.into();
                }
            };
        };
        Ok(())
    }

}

/// Specifies the old and new letters to replace.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LetterValues{
    pub old: String,
    pub new: String,
}

/// Determines where a letter should be replaced.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LetterPlaceType {
    #[serde(rename="first")]
    First,
    #[serde(rename="all")]
    All,
    #[serde(rename="last")]
    Last,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LetterArrayValues{
    Char(String),
    Place(i32)
}


#[cfg(test)]
mod tests {
    use crate::transforms::{TransformFunc, LetterValues, LetterPlaceType, LetterArrayValues};
    use crate::kirum::Lexis;
    use crate::word::PartOfSpeech;
    use super::Transform;

    fn rhai_setup() -> Lexis {
        Lexis{
            language: "testlang".to_string(),
            word: Some("example".into()), 
            pos: Some(PartOfSpeech::Noun),
            tags: vec!["test".to_string()],
            historical_metadata: [("test".to_string(), "true".to_string())].into(),
            ..Default::default()}
    }

    #[test]
    fn test_rhai_script_metadata_tags() {
        let mut word = rhai_setup();
        let transform = Transform{
            name: "test".to_string(),
            lex_match: None,
            transforms: vec![
                TransformFunc::RhaiScript { file: "testfiles/basic.rhai".to_string() }
            ]
        };
        transform.transform(&mut word).unwrap();
        assert_eq!(word.word.unwrap().string_without_sep(), "example-test&map:true".to_string())
    }

    #[test]
    fn test_rhai_return_array() {
        let mut word = rhai_setup();
        let transform = Transform{
            name: "test".to_string(),
            lex_match: None,
            transforms: vec![
                TransformFunc::RhaiScript { file: "testfiles/return_array.rhai".to_string() }
            ]
        };
        transform.transform(&mut word).unwrap();
        assert_eq!(word.word.unwrap().string_without_sep(), "+e+x+a+m+p+l+e".to_string())
    }

    #[test]
    fn test_rhai_pos() {
        let mut word = rhai_setup();
        let transform = Transform{
            name: "test".to_string(),
            lex_match: None,
            transforms: vec![
                TransformFunc::RhaiScript { file: "testfiles/pos.rhai".to_string() }
            ]
        };
        transform.transform(&mut word).unwrap();
        assert_eq!(word.word.unwrap().string_without_sep(), "example-noun".to_string())
    }

    #[test]
    fn test_rhai_language() {
        let mut word = rhai_setup();
        let transform = Transform{
            name: "test".to_string(),
            lex_match: None,
            transforms: vec![
                TransformFunc::RhaiScript { file: "testfiles/language.rhai".to_string() }
            ]
        };
        transform.transform(&mut word).unwrap();
        assert_eq!(word.word.unwrap().string_without_sep(), "example-testlang".to_string())
    }

    #[test]
    fn test_rhai_complex_unicode_lemma() {
        let mut word = Lexis{
            language: "testlang".to_string(),
            word: Some(vec!["hʷ", "a", "n"].into()), 
            pos: Some(PartOfSpeech::Noun),
            tags: vec!["test".to_string()],
            historical_metadata: [("test".to_string(), "true".to_string())].into(),
            ..Default::default()};

        let transform = Transform{
                name: "test".to_string(),
                lex_match: None,
                transforms: vec![
                    TransformFunc::RhaiScript { file: "testfiles/unicode_handle.rhai".to_string() }
            ]
        };

        transform.transform(&mut word).unwrap();
        assert_eq!(word.word.unwrap().string_without_sep(), "hanʷ".to_string())
    }

    #[test]
    fn test_replace_all_multiple_matches() {
        let mut word = Lexis{word: Some("kirum".into()), ..Default::default()};
        let transform = Transform{
            name: "test".to_string(),
            lex_match: None,
            transforms: vec![
                TransformFunc::LetterReplace { letter: LetterValues { old: "k".to_string(), new: "o".to_string() }, replace: LetterPlaceType::All },
                TransformFunc::LetterReplace { letter: LetterValues { old: "m".to_string(), new: "n".to_string() }, replace: LetterPlaceType::All },
            ]
        };

        transform.transform(&mut word).unwrap();

        assert_eq!(word.word.unwrap().string_without_sep(), "oirun".to_string())
    }

    #[test]
    fn test_letter_replace(){
        let letter_logic = LetterValues { old: "u".to_string(), new: "a".to_string() };
        let test_transform = TransformFunc::LetterReplace { letter: letter_logic, replace:  LetterPlaceType::All};
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default() };
        
        test_transform.transform(&mut old_word).unwrap();
        //let compare: Word = "karam".into();
        assert_eq!("karam".to_string(), old_word.word.unwrap().string_without_sep());
    }

    #[test]
    fn test_letter_array(){
        let test_transform = TransformFunc::LetterArray { letters: vec![LetterArrayValues::Place(0), LetterArrayValues::Place(1),  LetterArrayValues::Char("u".to_string())] };
        let mut old_word =  Lexis{word: Some("krm".into()), ..Default::default() };

        test_transform.transform(&mut old_word).unwrap();
        assert_eq!("kru".to_string(), old_word.word.unwrap().string_without_sep());

    }

    #[test]
    fn test_postfix(){
        let test_transform = TransformFunc::Postfix { value: "uh".into() };
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        test_transform.transform(&mut old_word).unwrap();
        assert_eq!("kurumuh".to_string(), old_word.word.unwrap().string_without_sep())
    }

    #[test]
    fn test_prefix(){
        let test_transform = TransformFunc::Prefix { value: "tur".into() };
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        test_transform.transform(&mut old_word).unwrap();
        assert_eq!("turkurum".to_string(), old_word.word.unwrap().string_without_sep());
    }



}