use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::{transforms, transforms::LetterRemovePosition, transforms::LetterArrayValues};


#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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
    agglutination: Option<Agglutination>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agglutination {
    order: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    pub etymon: String,
    pub transform: String,
    pub agglutination_order: Option<i32>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Word {
    Letters(Vec<String>),
    String(String),
}

impl Word{
    pub fn chars(self) -> Vec<String> {
        match self{
            Word::Letters(letters) => letters,
            Word::String(s) =>{
                s.chars().map(|c| String::from(c)).collect()
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


    pub fn replace(self, old: String, new: String, _kind: &transforms::LetterReplaceType) -> Word{
        let capture = format!("({})", old);
        let re = Regex::new(&capture).unwrap();
        match self{
            Word::Letters(letters) =>{
                let mut updated: Vec<String> = Vec::new();
                for letter in letters{
                    let new_letter = re.replace_all(&letter, new.clone());
                    updated.push(new_letter.to_string());
                }
                
                Word::Letters(updated)
            },
            Word::String(s) =>{
                let updated = re.replace_all(&s, new);
                Word::String(updated.to_string())
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
                s.chars().map(|c| String::from(c)).collect()
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
    fn test_replace() {
        let vec_word: Word = vec!["k".to_string(), "i".to_string(), "r".to_string(), "u".to_string(), "m".to_string()].into();
        let updated_vec = vec_word.replace("i".to_string(), "e".to_string(), &LetterReplaceType::All);

        assert_eq!(updated_vec.to_string(), "kerum".to_string());

        let str_word: Word = "kirum".into();
        let updated_str = str_word.replace("i".into(), "e".into(), &LetterReplaceType::All);

        assert_eq!(updated_str.to_string(), "kerum".to_string());
    }
}