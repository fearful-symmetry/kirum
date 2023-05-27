use regex::Regex;
use serde::{Deserialize, Serialize};
use crate::{transforms, transforms::{LetterArrayValues, LetterPlaceType}};
use serde_with::skip_serializing_none;
use log::error;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum PartOfSpeech {
    #[serde(rename(deserialize= "noun", serialize="noun"))]
    Noun,
    #[serde(rename(deserialize="verb", serialize="verb"))]
    Verb,
    #[serde(rename(deserialize= "adjective", serialize="adjective"))]
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
    pub transforms: Vec<String>,
    pub agglutination_order: Option<i32>
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Word {
    Letters(Vec<String>),
    String(String),
}

impl Default for Word{
    fn default() -> Self {
        Word::String("".to_string())
    }
}

impl Serialize for Word{
 fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
     where
         S: serde::Serializer {
     serializer.serialize_str(&self.to_string())
 }
}

impl IntoIterator for Word{
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Letters(l) => l.into_iter(),
            Self::String(s) => {
                let svec: Vec<String> = s.chars().map(String::from).collect();
                svec.into_iter()
            }
        }
    }
}

impl FromIterator<std::string::String> for Word {
    fn from_iter<T: IntoIterator<Item = std::string::String>>(iter: T) -> Self {
        let svec: Vec<String> = iter.into_iter().collect();
        svec.into()
    }
}

impl From<Vec<String>> for Word{
    fn from(input: Vec<String>) -> Self {
        Word::Letters(input)
    }
}

impl From<Vec<&str>> for Word{
    fn from(value: Vec<&str>) -> Self {
        let string_vec: Vec<String> = value.into_iter().map(|c|c.to_owned()).collect();
        string_vec.into() 
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

impl Word{
    pub fn chars(self) -> Vec<String> {
        match self{
            Word::Letters(letters) => letters,
            Word::String(s) =>{
                s.chars().map(String::from).collect()
            }
        }
    }

    pub fn remove_char(self, char: &str, remove_type: &LetterPlaceType) ->Word{
        let mut char_arr = match self{
            Word::Letters(letters) => letters,
            Word::String(_) => self.chars()
        };
        match remove_type{
            LetterPlaceType::All =>{
                char_arr.retain(|c| c != char);

            },
            LetterPlaceType::First =>{
                let pos = char_arr.iter().position(|c| c == char);
                if let Some(to_remove) = pos{
                    char_arr.remove(to_remove);
                }
            },
            LetterPlaceType::Last =>{
                let pos = char_arr.iter().rposition(|c| c == char);
                if let Some(to_remove) = pos {
                    char_arr.remove(to_remove);
                }
            }
        };
        char_arr.into()
    }


    pub fn replace(self, old: &str, new: &str, kind: &transforms::LetterPlaceType) -> Word{
        match kind {
            transforms::LetterPlaceType::All => {
                self.into_iter().map(|l| if l == old {new.to_owned()} else {l}).collect()
            }
            transforms::LetterPlaceType::First => {
                let mut found = false;
                self.into_iter().map(|l| if l == old && !found {found=true; new.to_owned()} else {l}).collect()
            },
            transforms::LetterPlaceType::Last => {
                let mut found = false;
                let mut reved: Vec<String> = self.into_iter().rev().map(|l| if l == old && !found {found=true; new.to_owned()} else {l}).collect();
                reved.reverse();
                reved.into()
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

    pub fn dedouble(self, letter: &str, position: &LetterPlaceType) -> Word {
        let mut acc: Vec<String> = Vec::new();

        let mut found = false;
        let mut cur = String::new();
        match position {
            LetterPlaceType::All => {
                for char in self.into_iter(){
                    if char == cur && char == letter {
                        continue
                    }
                    acc.push(char.clone());
                    cur = char;
                }
            },
            LetterPlaceType::First => {
                for char in self.into_iter(){
                    if char == cur && !found && char == letter {
                        found = true;
                        continue
                    }
                    acc.push(char.clone());
                    cur = char;
                }
            },
            LetterPlaceType::Last => {
                for char in self.into_iter().rev(){
                    if char == cur && !found && char == letter {
                        found = true;
                        continue
                    }
                    acc.insert(0, char.clone());
                    cur = char;
                }
            }
        }
        acc.into()
    }

    pub fn double(self, letter: &str, position: &LetterPlaceType) -> Word {
        match position {
            LetterPlaceType::All => {
                self.into_iter().map(|c| if c == letter{format!("{}{}",c,c)}else {c}).collect()
            },
            LetterPlaceType::First => {
                let found = self.clone().into_iter().position(|c|c == letter);
                double_vec(self.chars(), letter, found, false)
            },
            LetterPlaceType::Last => {
                let mut found = self.chars();
                found.reverse();
                let found_pos = found.clone().into_iter().position(|c|c == letter);
                double_vec(found, letter, found_pos, true)
            }
        }
    }

    // TODO: This is a lossy operation, as we have to convert Word to a string,
    // which means we lose any user data about what a character is. If a Word letter
    // is multiple unicode characters, we'll loose that next time we need to iterate over characters.
    // Instead of an enum, a word should probably be a String + metadata about individual characters
    pub fn match_replace(self, old: &str, new: &str) -> Word {
        let re = match Regex::new(old) {
            Ok(m) => m,
            Err(err) => {
                error!("could not parse match {}, returning: {}", old, err );
                return self
            }
        };
        let word_string = self.to_string();
        let updated = re.replace(&word_string, new);
        updated.into_owned().into()
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

fn double_vec(current: Vec<String>, letter: &str, found_pos: Option<usize>, reverse: bool) -> Word {
    let mut updated: Vec<String> = current;
    if let Some(pos) = found_pos { updated.insert(pos, letter.to_owned()) }
    if reverse{
        updated.reverse();
    }
    updated.into()
}

#[cfg(test)]
mod tests {
    use crate::{word::Word, transforms::LetterPlaceType};

    #[test]
    fn test_replace_all() {
        let vec_word: Word = vec!["k", "i", "r", "u", "m"].into();
        let updated_vec = vec_word.replace("i", "e", &LetterPlaceType::All);

        assert_eq!(updated_vec.to_string(), "kerum".to_string());

        let str_word: Word = "kirum".into();
        let updated_str = str_word.replace("i".into(), "e".into(), &LetterPlaceType::All);

        assert_eq!(updated_str.to_string(), "kerum".to_string());
    }

    #[test]
    fn test_replace_first(){
        let vec_word: Word = vec!["k", "i", "r", "u", "u"].into();
        let updated_vec = vec_word.replace("u", "h", &LetterPlaceType::First);

        assert_eq!(updated_vec.to_string(), "kirhu".to_string());
    }

    #[test]
    fn test_replace_last(){
        let vec_word: Word = vec!["u", "i", "r", "u"].into();
        let updated_vec = vec_word.replace("u", "h", &LetterPlaceType::Last);

        assert_eq!(updated_vec.to_string(), "uirh".to_string());
    }

    #[test]
    fn test_double_all() {
        let string_word: Word = String::from("test").into();
        let updated_word = string_word.double("t", &LetterPlaceType::All);

        assert_eq!(updated_word.to_string(), String::from("ttestt"));
    }

    #[test]
    fn test_double_first(){
        let string_word: Word = String::from("test").into();
        let updated_word = string_word.double("t", &LetterPlaceType::First);

        assert_eq!(updated_word.to_string(), String::from("ttest"));
    }

    #[test]
    fn test_double_last(){
        let string_word: Word = String::from("test").into();
        let updated_word = string_word.double("t", &LetterPlaceType::Last);

        assert_eq!(updated_word.to_string(), String::from("testt"));
    }

    #[test]
    fn test_dedouble_all() {
        let string_word: Word = String::from("ttestt").into();
        let updated_word = string_word.dedouble("t", &LetterPlaceType::All);

        assert_eq!(updated_word.to_string(), String::from("test"));
    }

    #[test]
    fn test_dedouble_first() {
        let string_word: Word = String::from("ttestt").into();
        let updated_word = string_word.dedouble("t", &LetterPlaceType::First);

        assert_eq!(updated_word.to_string(), String::from("testt"));
    }

    #[test]
    fn test_dedouble_last() {
        let string_word: Word = String::from("ttestt").into();
        let updated_word = string_word.dedouble("t", &LetterPlaceType::Last);

        assert_eq!(updated_word.to_string(), String::from("ttest"));
    }

    #[test]
    fn test_match_replace() {
        let string_word: Word = String::from("kirum").into();
        let updated_word = string_word.match_replace("rum", "teh");

        assert_eq!(updated_word.to_string(), String::from("kiteh"));
    }
}