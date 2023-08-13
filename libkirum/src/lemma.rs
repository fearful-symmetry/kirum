use serde::{Serialize, Deserialize, de::Visitor};
use unicode_segmentation::UnicodeSegmentation;
use crate::transforms::{LetterPlaceType, LetterArrayValues};
use regex::Regex;
use log::error;

const WORD_SEP: char = '\u{200B}';

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lemma {
    value: String,
}

impl std::fmt::Debug for Lemma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.string_without_sep()))
    }
}


impl Serialize for Lemma {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&self.string_without_sep())
    }
}

impl<'de> Deserialize<'de> for Lemma {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_any(LemmaVisitor)
    }
}

struct LemmaVisitor;

impl<'de> Visitor<'de> for LemmaVisitor {
    type Value = Lemma;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string or array of strings")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        let lem: Lemma = v.to_owned().into();
        Ok(lem)
    }

    fn visit_seq<A>(self, mut seq:  A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>, {
                let mut acc: Vec<String> = Vec::new();
                while let Some(value) = seq.next_element()? {
                    acc.push(value);
                };
                let parsed: Lemma = acc.into();
                Ok(parsed)
    }

}

impl IntoIterator for Lemma {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let separated: Vec<String> = self.into();
        separated.into_iter()
    }
}

impl FromIterator<std::string::String> for Lemma {
    fn from_iter<T: IntoIterator<Item = std::string::String>>(iter: T) -> Self {
        let from_iter: Vec<String> = iter.into_iter().collect();
        from_iter.into()
    }
}

impl From<Vec<String>> for Lemma {
    fn from(value: Vec<String>) -> Self {
        let mut build = String::new();
        for part in value.into_iter() {
            if part == WORD_SEP.to_string() || part == "" {
                continue
            }
            build = format!("{}{}", build, part);
            build.push(WORD_SEP)
        }
        Lemma {value: build}
    }
}

impl From<Vec<&str>> for Lemma {
    fn from(value: Vec<&str>) -> Self {
        let string_vec: Vec<String> = value.into_iter().map(|c|c.to_owned()).collect();
        string_vec.into() 
    }
}

impl From<String> for Lemma {
    fn from(value: String) -> Self {
        let strings: Vec<String> = value.graphemes(true).map(|c| c.to_string()).collect();
        strings.into()
    }
}

impl From<&'static str> for Lemma {
    fn from(value: &'static str) -> Self {
        value.to_string().into()
    }
}

impl std::string::ToString for Lemma {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}

impl From<Lemma> for Vec<String> {
    fn from(value: Lemma) -> Self {
        value.value.split(WORD_SEP).map(|c|c.to_owned()).filter(|c| c != "").collect()
    }
}


impl Lemma {

    pub fn len(&self) -> usize {
        self.clone().into_iter().count()
    }

    pub fn push(&mut self, pushed: Lemma) {
        if self.len() >0 {
            let mut vectored: Vec<String> = self.clone().into();
            let mut update_vec: Vec<String> = pushed.into();
            vectored.append(&mut update_vec);
            let updated: Lemma = vectored.into();
            self.value = updated.value
        } else {
            self.value = pushed.value
        }

    }

    pub fn push_char(&mut self, pushed: &str) {
        // a bit horrible, but the easiest way to insure we're inserting the separators properly
        if self.len() > 0 {
            let mut vectored: Vec<String> = self.clone().into();
            vectored.push(pushed.to_string());
            let updated: Lemma = vectored.into();
            self.value = updated.value
        } else {
            self.value = pushed.to_string();
        }

    }

    /// Return a string without the Lemma-specific character delimiters
    pub fn string_without_sep(&self) -> String {
        let rep = WORD_SEP.to_string();
        self.value.replace(&rep, "")
    }

    /// Turn the Lemma into a vector of characters
    pub fn chars(self) -> Vec<String> {
        self.into_iter().collect()
    }

    /// Removes the given character from the Lemma
    pub fn remove_char(&mut self, char: &str, remove_type: &LetterPlaceType) {
        self.replace_str(char, "", remove_type);
        self.dedouble_sep();
    }

    /// Replace the specified character
    pub fn replace(&mut self, old: &str, new: &str, kind: &LetterPlaceType) {
        self.replace_str(old, new, kind)
    }

    /// Adds the prefix to the given Lemma
    pub fn add_prefix(&mut self, prefix: &Lemma) {
        self.value = format!("{}{}", prefix.value, self.value)
    }

    /// Adds the postfix to the given Lemma
    pub fn add_postfix(&mut self, postfix: &Lemma) {
        self.value = format!("{}{}", self.value, postfix.value)
    }

    // TODO: refactor, this is horrible, clones should not be needed
    pub fn dedouble(&mut self, letter: &str, position: &LetterPlaceType) {
        let mut acc: Vec<String> = Vec::new();
        let mut found = false;
        let mut cur = String::new();
        match position {
            LetterPlaceType::All => {
                for char in self.clone().into_iter() {
                    if char == cur && char == letter {
                        continue
                    }
                    acc.push(char.clone());
                    cur = char;
                }
            },
            LetterPlaceType::First => {
                for char in self.clone().into_iter(){
                    if char == cur && !found && char == letter {
                        found = true;
                        continue
                    }
                    acc.push(char.clone());
                    cur = char;
                }
            },
            LetterPlaceType::Last => {
                for char in self.clone().into_iter().rev(){
                    if char == cur && !found && char == letter {
                        found = true;
                        continue
                    }
                    acc.insert(0, char.clone());
                    cur = char;
                }
            }
        }

        let new_lemma: Lemma = acc.into();
        self.value = new_lemma.value;
    }

    // TODO: refactor, this is horrible, clones should not be needed
    pub fn double(&mut self, letter: &str, position: &LetterPlaceType) {
        match position {
            LetterPlaceType::All => {
                let updated: Lemma = self.clone().into_iter().map(|c| if c == letter{format!("{}{}",c,c)}else {c}).collect();
                self.value = updated.value;
            },
            LetterPlaceType::First => {
                let found = self.clone().into_iter().position(|c|c == letter);
                let updated = double_vec(self.clone().chars(), letter, found, false);
                self.value = updated.value;
            },
            LetterPlaceType::Last => {
                let mut found = self.clone().chars();
                found.reverse();
                let found_pos = found.clone().into_iter().position(|c|c == letter);
                let updated = double_vec(found, letter, found_pos, true);
                self.value = updated.value;
            }
        }
    }

    /// match_replace replaces the target substring with the given new string.
    /// It assumes that all strings are in proper "lemmatized" type, as
    /// the underlying regex call with fail if one substring is using different unicode delimiters.
    pub fn match_replace(&mut self, old: &Lemma, new: &Lemma) {
        let re = match Regex::new(&old.value) {
            Ok(m) => m,
            Err(err) => {
                error!("could not parse match {}, returning: {}", old.value, err );
                return
            }
        };
        //let word_string = self.to_string();
        let updated = re.replace(&self.value, new.value.clone());
        self.value = updated.into_owned();
        self.dedouble_sep();
    }

    pub fn modify_with_array(&mut self, transform_array: &Vec<LetterArrayValues>) {
        let working = self.clone().chars();

        let mut new_letters = String::new();

        for letter in transform_array {
            match letter {
                LetterArrayValues::Char(letter) => {
                    new_letters.push_str(letter);
                    new_letters.push(WORD_SEP);
                },
                LetterArrayValues::Place(pos) => {
                    let letter = match working.get(*pos as usize){
                        Some(letter) => letter,
                        None => {
                            continue
                        }
                    };
                    new_letters.push_str(letter);
                    new_letters.push(WORD_SEP);
                }
            }
        }
        self.value = new_letters;

    }

    fn replace_str(&mut self, old: &str, new: &str, kind: &LetterPlaceType) {
        match kind {
            LetterPlaceType::All => {
                let upd = self.value.replace(old, new);
                self.value = upd;
            },
            LetterPlaceType::First => {
                let upd = self.value.replacen(old, new, 1);
                self.value = upd;
            },
            LetterPlaceType::Last => {
                let revd: Lemma = self.clone().into_iter().rev().collect();
                let rev_replace = revd.value.replacen(old, new, 1);
                let completed_rev: Lemma = rev_replace.into();
                let completed: Lemma = completed_rev.into_iter().rev().collect();
                self.value = completed.value;
            }
        }
    }

    fn dedouble_sep(&mut self) {
        let mut acc = String::new();
        let mut cur = "";
        for char in self.value.graphemes(true) {
            if char == cur && char == WORD_SEP.to_string() {
                continue
            }
            acc.push_str(char);
            cur = char;
        }
        self.value = acc;
    }
}

// if found_pos exists, double the character at that position
fn double_vec(current: Vec<String>, letter: &str, found_pos: Option<usize>, reverse: bool) -> Lemma {
    let mut updated: Vec<String> = current;
    if let Some(pos) = found_pos { updated.insert(pos, letter.to_owned()) }
    if reverse{
        updated.reverse();
    }
    updated.into()
}

#[cfg(test)]
mod tests {
    use crate::{lemma::Lemma, transforms::{LetterPlaceType, LetterArrayValues}};

    #[test]
    fn test_char_array() {
        let mut vec_word: Lemma = vec!["k", "i", "r", "u", "m"].into();
        let test_array = vec![LetterArrayValues::Place(0),
        LetterArrayValues::Char("t".to_string()),
        LetterArrayValues::Char("q".to_string()),
        LetterArrayValues::Place(1)];
        vec_word.modify_with_array(&test_array);

        let golden: Lemma = vec!["k", "t", "q", "i"].into();

        assert_eq!(vec_word.value, golden.value);
    }

    #[test]
    fn test_remove() {
        let mut vec_word: Lemma = vec!["k", "i", "r", "u", "m"].into();
        vec_word.remove_char("i", &LetterPlaceType::All);

        // Do this so we can compare the word, and the placement of the separator
        let golden_word: Lemma = vec!["k", "r", "u", "m"].into();

        assert_eq!(vec_word.value, golden_word.value);
    }

    #[test]
    fn test_replace_all() {
        let mut vec_word: Lemma = vec!["k", "i", "r", "u", "m"].into();
        vec_word.replace("i", "e", &LetterPlaceType::All);

        assert_eq!(vec_word.string_without_sep(), "kerum".to_string());

        let mut str_word: Lemma = "kirum".into();
        str_word.replace("i".into(), "e".into(), &LetterPlaceType::All);

        assert_eq!(str_word.string_without_sep(), "kerum".to_string());
    }

    #[test]
    fn test_replace_first(){
        let mut vec_word: Lemma = vec!["k", "i", "r", "u", "u"].into();
        vec_word.replace("u", "h", &LetterPlaceType::First);

        assert_eq!(vec_word.string_without_sep(), "kirhu".to_string());
    }

    #[test]
    fn test_replace_last(){
        let mut vec_word: Lemma = vec!["u", "i", "r", "u"].into();
        vec_word.replace("u", "h", &LetterPlaceType::Last);

        assert_eq!(vec_word.string_without_sep(), "uirh".to_string());
    }

    #[test]
    fn test_double_all() {
        let mut string_word: Lemma = String::from("test").into();
        string_word.double("t", &LetterPlaceType::All);

        assert_eq!(string_word.string_without_sep(), String::from("ttestt"));
    }

    #[test]
    fn test_double_first(){
        let mut string_word: Lemma = String::from("test").into();
        string_word.double("t", &LetterPlaceType::First);

        assert_eq!(string_word.string_without_sep(), String::from("ttest"));
    }

    #[test]
    fn test_double_last(){
        let mut string_word: Lemma = String::from("test").into();
        string_word.double("t", &LetterPlaceType::Last);

        assert_eq!(string_word.string_without_sep(), String::from("testt"));
    }

    #[test]
    fn test_dedouble_all() {
        let mut string_word: Lemma = String::from("ttestt").into();
        string_word.dedouble("t", &LetterPlaceType::All);

        assert_eq!(string_word.string_without_sep(), String::from("test"));
    }

    #[test]
    fn test_dedouble_first() {
        let mut string_word: Lemma = String::from("ttestt").into();
        string_word.dedouble("t", &LetterPlaceType::First);

        assert_eq!(string_word.string_without_sep(), String::from("testt"));
    }

    #[test]
    fn test_dedouble_last() {
        let mut string_word: Lemma = String::from("ttestt").into();
        string_word.dedouble("t", &LetterPlaceType::Last);

        assert_eq!(string_word.string_without_sep(), String::from("ttest"));
    }

    #[test]
    fn test_match_replace() {
        let mut string_word: Lemma = String::from("kirum").into();
        string_word.match_replace(&"rum".into(), &"teh".into());

        assert_eq!(string_word.string_without_sep(), String::from("kiteh"));
    }
}