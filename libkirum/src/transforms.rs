use serde::{Deserialize, Serialize};
use crate::{matching::LexisMatch, kirum::Lexis, lemma::Lemma};
use log::debug;

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
    pub fn transform(&self, etymon: &mut Lexis) {
        self.transform_option(etymon);
    }

    // Transform the given lexis, or return None if the lex_match condition evaluates to false
    pub fn transform_option(&self, etymon: &mut Lexis) -> bool {
        let can_transform = if let Some(lex_match) = &self.lex_match{
            lex_match.matches(etymon)
        } else {
            true
        };
        //let mut updated = etymon.clone();
        if can_transform{
            for transform in &self.transforms {
                transform.transform(etymon); 
            };
            true
        } else{
            false
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
    MatchReplace{old: Lemma, new: Lemma}
}


impl TransformFunc{
    pub fn transform(&self, current_word: &mut Lexis) {
        if current_word.word.is_none(){
            return
        }
        if let Some(current) = current_word.word.as_mut() {
            match self {
                TransformFunc::LetterReplace{ letter, replace } => {
                    debug!("got LetterReplace for {}", current_word.id);
                   current.replace(&letter.old, &letter.new, replace)
                },
                TransformFunc::LetterArray { letters } => {
                    debug!("got LetterArray for {}", current_word.id);
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
                }
            };
        }

        //Lexis{word: Some(new_word), ..current_word.clone()}
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

    #[test]
    fn test_letter_replace(){
        let letter_logic = LetterValues { old: "u".to_string(), new: "a".to_string() };
        let test_transform = TransformFunc::LetterReplace { letter: letter_logic, replace:  LetterPlaceType::All};
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default() };
        
        test_transform.transform(&mut old_word);
        //let compare: Word = "karam".into();
        assert_eq!("karam".to_string(), old_word.word.unwrap().string_without_sep());
    }

    #[test]
    fn test_letter_array(){
        let test_transform = TransformFunc::LetterArray { letters: vec![LetterArrayValues::Place(0), LetterArrayValues::Place(1),  LetterArrayValues::Char("u".to_string())] };
        let mut old_word =  Lexis{word: Some("krm".into()), ..Default::default() };

        test_transform.transform(&mut old_word);
        assert_eq!("kru".to_string(), old_word.word.unwrap().string_without_sep());

    }

    #[test]
    fn test_postfix(){
        let test_transform = TransformFunc::Postfix { value: "uh".into() };
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        test_transform.transform(&mut old_word);
        assert_eq!("kurumuh".to_string(), old_word.word.unwrap().string_without_sep())
    }

    #[test]
    fn test_prefix(){
        let test_transform = TransformFunc::Prefix { value: "tur".into() };
        let mut old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        test_transform.transform(&mut old_word);
        assert_eq!("turkurum".to_string(), old_word.word.unwrap().string_without_sep());
    }



}