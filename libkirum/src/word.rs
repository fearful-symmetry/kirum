use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::{transforms, transforms::LetterRemovePosition, transforms::LetterArrayValues};
use serde_with::skip_serializing_none;


#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum PartOfSpeech {
    #[serde(rename = "noun")]
    Noun,
    #[serde(rename = "verb")]
    Verb,
    #[serde(rename = "adjective")]
    Adjective,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Etymology{
    pub etymons: Vec<Edge>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agglutination {
    order: Vec<String>
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub etymon: String,
    pub transform: Vec<String>,
    pub agglutination_order: Option<i32>
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Word {
    Letters(Vec<String>),
    String(String),
}

impl Serialize for Word{
 fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
     where
         S: serde::Serializer {
     serializer.serialize_str(&self.to_string())
 }
}

impl Word{
    pub fn chars(self) -> Vec<String> {
        match self{
            Word::Letters(letters) => letters,
            Word::String(s) =>{
                s.chars().map(String::from).collect()
            }
        }
    }

    pub fn remove_char(self, char: String, remove_type: LetterRemovePosition) ->Word{
        let mut char_arr = match self{
            Word::Letters(letters) => letters,
            Word::String(_) => self.chars()
        };
        match remove_type{
            LetterRemovePosition::All =>{
                char_arr.retain(|c| c != &char);

            },
            LetterRemovePosition::First =>{
                let pos = char_arr.iter().position(|c| c == &char);
                if let Some(to_remove) = pos{
                    char_arr.remove(to_remove);
                }
            },
            LetterRemovePosition::Last =>{
                let pos = char_arr.iter().rposition(|c| c == &char);
                if let Some(to_remove) = pos {
                    char_arr.remove(to_remove);
                }
            }
        };
        char_arr.into()
    }


    pub fn replace(self, old: String, new: String, kind: &transforms::LetterReplaceType) -> Word{
        let capture = format!("({})", old);
        let re = Regex::new(&capture).unwrap();
        match self{
            Word::Letters(letters) =>{
                let mut updated: Vec<String> = Vec::new();
                let mut replaced = false;
                for letter in letters{
                    match kind{
                        transforms::LetterReplaceType::All =>{
                            if old == letter{
                                updated.push(new.clone());
                            }else{
                                updated.push(letter);
                            }
                        },
                        transforms::LetterReplaceType::First =>{
                            if old == letter && !replaced{
                                updated.push(new.clone());
                                replaced = true;
                            }else{
                                updated.push(letter);
                            }
                        }
                    }

                }
                
                Word::Letters(updated)
            },
            Word::String(s) =>{
                match kind{
                    transforms::LetterReplaceType::All =>{
                        let updated = re.replace_all(&s, new);
                        Word::String(updated.to_string())
                    }
                    transforms::LetterReplaceType::First =>{
                        let updated = re.replace(&s, new);
                        Word::String(updated.to_string())
                    }
                }

            }
        }
    }
    pub fn add_prefix(self, prefix: Word) -> Word{
        match self{
            Word::Letters(letters) =>{
                let prefix_arr: Vec<String> = prefix.chars();
                Word::Letters([prefix_arr, letters].concat())
            },
            Word::String(s) =>{
                Word::String(format!("{}{}", prefix.to_string(), s))
            }
        }
    }
    pub fn add_postfix(self, postfix: Word) -> Word{
        match self{
            Word::Letters(letters) =>{
                let prefic_arr: Vec<String> = postfix.chars();
                Word::Letters([letters, prefic_arr].concat())
            },
            Word::String(s) =>{
                Word::String(format!("{}{}", s, postfix.to_string()))
            }
        }
    }
    
    pub fn modify_with_array(self, transform_array: &Vec<LetterArrayValues>) -> Word{
        let working_array = match self{
            Word::Letters(letters) => letters,
            Word::String(s) => {
                s.chars().map(String::from).collect()
            }
        };
        let mut new_letters: Vec<String> = Vec::new();
        for letter in transform_array{
            match letter{
                LetterArrayValues::Place(num) => {
                    let letter = match working_array.get(*num as usize){
                        Some(letter) => letter,
                        None => {
                            continue
                        }
                    };
                    new_letters.push(letter.to_string());
                }
                LetterArrayValues::Char(letter) => {
                    new_letters.push(letter.clone());
                }
            }
        };
        Word::Letters(new_letters)

    }
}

impl From<Vec<String>> for Word{
    fn from(input: Vec<String>) -> Self {
        Word::Letters(input)
    }
}

impl From<String> for Word{
    fn from(source: String) -> Self {
        Word::String(source)
    }
}

impl From<&'static str> for Word{
    fn from(source: &'static str) -> Self {
        Word::String(source.to_string())
    }
}

impl std::string::ToString for Word{
    fn to_string(&self) -> String {
        match self{
            Word::Letters(letters) => {
                letters.join("")
            },
            Word::String(s) => {
                s.to_string()
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::{word::Word, transforms::LetterReplaceType};

    #[test]
    fn test_replace_all() {
        let vec_word: Word = vec!["k".to_string(), "i".to_string(), "r".to_string(), "u".to_string(), "m".to_string()].into();
        let updated_vec = vec_word.replace("i".to_string(), "e".to_string(), &LetterReplaceType::All);

        assert_eq!(updated_vec.to_string(), "kerum".to_string());

        let str_word: Word = "kirum".into();
        let updated_str = str_word.replace("i".into(), "e".into(), &LetterReplaceType::All);

        assert_eq!(updated_str.to_string(), "kerum".to_string());
    }

    #[test]
    fn test_replace_first(){
        let vec_word: Word = vec!["k".to_string(), "i".to_string(), "r".to_string(), "u".to_string(), "u".to_string()].into();
        let updated_vec = vec_word.replace("u".to_string(), "h".to_string(), &LetterReplaceType::First);

        assert_eq!(updated_vec.to_string(), "kirhu".to_string());
    }
}