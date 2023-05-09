use serde::{Deserialize, Serialize};
use crate::{word::Word, matching::LexisMatch, kirum::Lexis};


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
    pub fn transform(&self, etymon: &Lexis) -> Lexis {
        self.transform_option(etymon).unwrap_or(etymon.clone())
    }

    // Transform the given lexis, or return None if the lex_match condition evaluates to false
    pub fn transform_option(&self, etymon: &Lexis) ->Option<Lexis>{
        let can_transform = if let Some(lex_match) = &self.lex_match{
            lex_match.matches(&etymon)
        } else {
            true
        };
        let mut updated = etymon.clone();
        if can_transform{
            for transform in &self.transforms {
                updated = transform.transform(&updated);
            };
            Some(updated)
        } else{
            None
        }
    }
}

 
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TransformFunc {
    #[serde(rename="letter_replace")]
    LetterReplace{letter: LetterValues, replace: LetterReplaceType},
    #[serde(rename="letter_array")]
    LetterArray{letters: Vec<LetterArrayValues>},
    #[serde(rename="postfix")]
    Postfix{value: Word},
    #[serde(rename="prefix")]
    Prefix{value: Word},
    #[serde(rename="loanword")]
    Loanword,
    #[serde(rename="letter_remove")]
    LetterRemove{letter: String, position: LetterRemovePosition}
}


impl TransformFunc{
    pub fn transform(&self, current_word: &Lexis) -> Lexis {
        if current_word.word.is_none(){
            return current_word.to_owned();
        }
        let new_word = match self {
            TransformFunc::LetterReplace{ letter, replace } => {
               current_word.word.clone().unwrap().replace(letter.old.clone(), letter.new.clone(), replace)
            },
            TransformFunc::LetterArray { letters } => {
                current_word.word.clone().unwrap().modify_with_array(letters) 
            },
            TransformFunc::Postfix { value } => {
                current_word.word.clone().unwrap().add_postfix(value.clone())
            },
            TransformFunc::Prefix { value } => {
                current_word.word.clone().unwrap().add_prefix(value.clone())

            },
            TransformFunc::Loanword => {
                current_word.word.clone().unwrap()
            },
            TransformFunc::LetterRemove {letter, position } =>{
                current_word.word.clone().unwrap().remove_char(letter.clone(), position.clone())
            }
        };
        Lexis{word: Some(new_word), ..current_word.clone()}
    }

}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LetterValues{
    pub old: String,
    pub new: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LetterRemovePosition{
    #[serde(rename="first")]
    First,
    #[serde(rename="last")]
    Last,
    #[serde(rename="all")]
    All
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LetterReplaceType {
    #[serde(rename="first")]
    First,
    #[serde(rename="all")]
    All,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum LetterArrayValues{
    Char(String),
    Place(i32)
}


#[cfg(test)]
mod tests {
    use crate::transforms::{TransformFunc, LetterValues, LetterReplaceType, LetterArrayValues, LetterRemovePosition};
    use crate::kirum::Lexis;
    use crate::word::Word;

    #[test]
    fn test_letter_replace(){
        let letter_logic = LetterValues { old: "u".to_string(), new: "a".to_string() };
        let test_transform = TransformFunc::LetterReplace { letter: letter_logic, replace:  LetterReplaceType::All};
        let old_word = Lexis{word: Some("kurum".into()), ..Default::default() };
        
        let new_word = test_transform.transform(&old_word);
        let compare: Word = "karam".into();
        assert_eq!(compare, new_word.word.unwrap());
    }

    #[test]
    fn test_letter_array(){
        let test_transform = TransformFunc::LetterArray { letters: vec![LetterArrayValues::Place(0), LetterArrayValues::Place(1),  LetterArrayValues::Char("u".to_string())] };
        let old_word =  Lexis{word: Some("krm".into()), ..Default::default() };

        let new_word = test_transform.transform(&old_word);
        assert_eq!("kru".to_string(), new_word.word.unwrap().to_string());

    }

    #[test]
    fn test_postfix(){
        let test_transform = TransformFunc::Postfix { value: "uh".into() };
        let old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        let new_word = test_transform.transform(&old_word);
        assert_eq!("kurumuh".to_string(), new_word.word.unwrap().to_string())
    }

    #[test]
    fn test_prefix(){
        let test_transform = TransformFunc::Prefix { value: "tur".into() };
        let old_word = Lexis{word: Some("kurum".into()), ..Default::default()};

        let new_word = test_transform.transform(&old_word);
        assert_eq!("turkurum".to_string(), new_word.word.unwrap().to_string());
    }

    #[test]
    fn test_letter_remove(){
        let test_transform = TransformFunc::LetterRemove { letter: "u".to_string(), position: LetterRemovePosition::All };
        let old_word = Lexis{word: Some("kurum".into()), ..Default::default()};
    
        let new_word = test_transform.transform(&old_word);
        assert_eq!("krm".to_string(), new_word.word.unwrap().to_string());
        
        let test_transform_first = TransformFunc::LetterRemove { letter: "u".to_string(), position: LetterRemovePosition::First };
        let new_word_first = test_transform_first.transform(&old_word);
        assert_eq!("krum".to_string(), new_word_first.word.unwrap().to_string());

        let test_transform_last = TransformFunc::LetterRemove { letter: "u".to_string(), position: LetterRemovePosition::Last };
        let new_word_last = test_transform_last.transform(&old_word);
        assert_eq!("kurm".to_string(), new_word_last.word.unwrap().to_string());
    }


}