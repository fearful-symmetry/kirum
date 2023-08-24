use std::collections::HashMap;
use rand::seq::SliceRandom;
use crate::{lemma::Lemma, errors::{self, PhoneticParsingError}};
use serde::{Deserialize, Serialize, de::{Visitor, self, Unexpected}};

/// Carries a set of phonological and letter groupings that taken together, can generate random words
/// that match the given phonetics
#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct LexPhonology {
    /// May contain a map of any kind of phonetic value, syllables, phonemes, etc.
    /// The keys of the hashmap are referenced in the following lexis_types below.
    /// When a word is generated, a PhoneticReference from the vector is chosen at random.
    /// Keys must be all uppercase to distinguish them from letter values.
    /// For example:
    /// C = v b r t h # The available consonants
    /// V = i u o y e # The available vowels
    /// S = CVC CVV VVC # The possible syllable structures
    pub groups: HashMap<char, Vec<PhoneticReference>>,
    /// A map of `groups` keys or PhoneticReferences. A key value in the map can be referenced
    /// in the `create` field of a Lexis to generate a word.
    /// Expanding on the above example: 
    /// word = S SS SuiS
    /// prefix = S uS Su
    pub lexis_types: HashMap<String, Vec<PhoneticReference>>
}

/// A single "reference" to a phonetic value used to generate words.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct PhoneticReference(Vec<CreateValue>);

impl Serialize for PhoneticReference{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for PhoneticReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_any(PhoneticReferenceVisitor)
    }
}

struct PhoneticReferenceVisitor;

impl<'de> Visitor<'de> for PhoneticReferenceVisitor {
    type Value = PhoneticReference;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a string value")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
        match v.try_into() {
            Err(_e) => {
                Err(de::Error::invalid_value(Unexpected::Str(v), &self))
            },
            Ok(v) => {Ok(v)}
        }
    }
}

// the PhoneticReference can be formatted one of two ways:
// CCCC
// C C C C
// the latter helps for cases where we've inserted a weird character that's more than one unicode character
impl TryFrom<&str> for PhoneticReference{
    type Error = PhoneticParsingError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut phon_vec: Vec<CreateValue> = Vec::new();
        if value.matches(' ').count() > 1{
            for char in value.split_whitespace(){
                phon_vec.push(char.try_into()?)
            }
        } else {
            for char in value.chars(){ 
                phon_vec.push(char.into())
            }
        }

        Ok(PhoneticReference(phon_vec))
    }

}

impl ToString for PhoneticReference{
    fn to_string(&self) -> String {
        let mut acc = String::new();
        for part in &self.0{
            acc.push_str(&part.to_string())
        }
        acc
    }
}


#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug)]
pub enum CreateValue {
    Phoneme(String),
    Reference(char)
}

impl ToString for CreateValue{
    fn to_string(&self) -> String {
        match self {
            Self::Phoneme(p) => p.to_string(),
            Self::Reference(r) => r.to_string()
        }
    }
}

impl TryFrom<&str> for CreateValue{
    type Error = errors::PhoneticParsingError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let found_uppercase = value.chars().filter(|c| c.is_uppercase()).count();
        if found_uppercase == value.len() && value.len() == 1 {
            let raw: char = value.chars().next()
            .ok_or_else(|| PhoneticParsingError {msg:"could not find character for reference", 
            found: value.to_string()})?;
            Ok(CreateValue::Reference(raw))

        } else if found_uppercase == 0 {
            Ok(CreateValue::Phoneme(value.to_string()))
            
        } else {
            Err(PhoneticParsingError{msg: "a reference can only be one upper-case character, or an all lowercase phonetic rule", 
            found: value.to_string()})
        }
            
        
    }
}

impl From<char> for CreateValue{
    fn from(value: char) -> Self {
        if value.is_lowercase(){
            CreateValue::Phoneme(value.to_string())
        } else {
            CreateValue::Reference(value)
        }
    }
}

impl Serialize for CreateValue{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for CreateValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        deserializer.deserialize_any(CreateValueVisitor)
    }
}

struct CreateValueVisitor;

impl<'de> Visitor<'de> for CreateValueVisitor {
    type Value = CreateValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an upper or lower case character value")
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
    Ok(v.into())
    }

    // logic: if an identifier is all uppercase, treat it as a reference,
    // otherwise, it's a string phoneme
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
                match v.try_into() {
                    Err(_e) => {
                        Err(de::Error::invalid_value(Unexpected::Str(v), &self))
                    },
                    Ok(v) => {Ok(v)}
                }
    }
}

impl LexPhonology {

    /// Creates a new random word based on the applied phonetic rules
    pub fn create_word(&self, lexis_type: &str) -> Option<Lemma> {
        if let Some(found_type_list) = self.lexis_types.get(lexis_type) {
            if let Some(selected_phon) = found_type_list.choose(&mut rand::thread_rng()) {
                return self.resolve_phonetic_reference(selected_phon)
            }
        }

        None
    }

    fn resolve_phonetic_reference(&self, pref: &PhoneticReference) -> Option<Lemma> {
        let mut phonetic_acc = Lemma::default();
        for phon in &pref.0 {
            match phon {
                CreateValue::Phoneme(p) => {phonetic_acc.push_char(p)},
                CreateValue::Reference(single_ref) => {
                    if let Some(found_ref) =  self.random_phoneme(single_ref) {
                        phonetic_acc.push(found_ref)
                    } else {
                        return None
                    }
                }
            }
        }

        if phonetic_acc.is_empty(){
            None
        } else {
            Some(phonetic_acc)
        }
        
    }

    fn random_phoneme(&self, phoneme_key: &char) -> Option<Lemma> {
        if let Some(type_val) = self.groups.get(phoneme_key) {
            let picked_from = type_val.choose(&mut rand::thread_rng());
            if let Some(picked) = picked_from {
                return self.resolve_phonetic_reference(picked)
            }
        }

        None
    }

}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{lexcreate::PhoneticReference, errors::PhoneticParsingError};
    use super::{LexPhonology, CreateValue};

    #[test]
    fn test_bad_phonetic_input(){
        let bad: Result<CreateValue, PhoneticParsingError> = "Ci".try_into();
        assert!(bad.is_err())
    }

    #[test]
    fn test_spaces_bad_input(){
        let test_phon: Result<CreateValue, PhoneticParsingError> = "C wV i C r rw".try_into();
        assert!(test_phon.is_err())
    }

    #[test]
    fn test_new_no_space() {
        let test_phon: PhoneticReference = "CCCC".try_into().unwrap();
        let expected = PhoneticReference(vec!['C'.into(), 'C'.into(), 'C'.into(), 'C'.into()]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_new_spaces() {
        let test_phon: PhoneticReference = "C V i C r rw".try_into().unwrap();

        let expected = PhoneticReference(vec![
            CreateValue::Reference('C'),
            CreateValue::Reference('V'),
            CreateValue::Phoneme("i".to_string()),
            CreateValue::Reference('C'),
            CreateValue::Phoneme("r".to_string()),
            CreateValue::Phoneme("rw".to_string())
        ]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_new_no_space_mix(){
        let test_phon: PhoneticReference = "CCrC".try_into().unwrap();
        let expected = PhoneticReference(vec![
            CreateValue::Reference('C'),
            CreateValue::Reference('C'),
            CreateValue::Phoneme("r".to_string()),
            CreateValue::Reference('C')
        ]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_basic_gen() {
        let test_phon = LexPhonology{
            groups: HashMap::from([
                ('C',
                vec![
                    PhoneticReference(vec![CreateValue::Phoneme("t".to_string())]), 
                    PhoneticReference(vec![CreateValue::Phoneme("r".to_string())])
                ]),
                ('V', 
                vec![
                    PhoneticReference(vec![CreateValue::Phoneme("u".to_string())]),
                    PhoneticReference(vec![CreateValue::Phoneme("i".to_string())])
                ]),
                ('S', 
                vec![
                    PhoneticReference(vec![
                        CreateValue::Reference('C'), 
                        CreateValue::Reference('V')
                    ]), 
                    PhoneticReference(vec![
                        CreateValue::Reference('V'), 
                        CreateValue::Reference('C')
                    ])
                ])
            ]),
            lexis_types: HashMap::from([
                ("words".to_string(), 
                vec![
                    PhoneticReference(vec![CreateValue::Reference('S')]), 
                    PhoneticReference(vec![
                        CreateValue::Reference('S'), 
                        CreateValue::Reference('S') 
                    ])
                ])
            ]),
        };

        let res = test_phon.create_word("words");
        assert_eq!(true, res.is_some());
        assert!(res.clone().unwrap().len() > 0);
        println!("got: {}", res.unwrap().to_string());
    }

    
}