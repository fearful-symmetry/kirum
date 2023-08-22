use std::collections::HashMap;
use rand::seq::SliceRandom;
use crate::lemma::Lemma;
use serde::{Deserialize, Serialize, de::Visitor};


/// Carries the phonological rules for a word generator.
#[derive(Clone, PartialEq, Serialize, Deserialize, Default, Debug)]
pub struct LexPhonology {
    /// May contain any kind of phonetic value, syllables, phonemes, etc
    /// the keys of the hashmap are referenced in the following lexis_types below.
    /// When a word is generated, a PhoneticReference from the vector is chosen at random.
    /// Keys must be all uppercase to distinguish them from letter values.
    /// For example:
    /// C = v b r t h # The available consonants
    /// V = i u o y e # The available vowels
    /// S = CVC CVV VVC # The possible syllable structures
    pub groups: HashMap<String, Vec<PhoneticReference>>,
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
        Ok(v.into())
    }
}

// the PhoneticReference can be formatted one of two ways:
// CCCC
// C C C C
// the latter helps for cases where we've inserted a weird character that's more than one unicode character
impl From<&str> for PhoneticReference{
    fn from(value: &str) -> Self {
        let mut phon_vec: Vec<CreateValue> = Vec::new();
        if value.matches(' ').count() > 1{
            for char in value.split_whitespace(){
                phon_vec.push(char.into())
            }
        } else {
            for char in value.chars(){
                phon_vec.push(char.into())
            }
        }

        PhoneticReference(phon_vec)
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
    Reference(String)
}

impl ToString for CreateValue{
    fn to_string(&self) -> String {
        match self {
            Self::Phoneme(p) => p.to_string(),
            Self::Reference(r) => r.to_string()
        }
    }
}

impl From<&str> for CreateValue{
    fn from(value: &str) -> Self {
        let found_lowercase = value.chars().find(|c| c.is_lowercase());
        if found_lowercase.is_some() {
            CreateValue::Phoneme(value.to_string())
        } else {
            CreateValue::Reference(value.to_string())
        }
    }
}

impl From<char> for CreateValue{
    fn from(value: char) -> Self {
        if value.is_lowercase(){
            CreateValue::Phoneme(value.to_string())
        } else {
            CreateValue::Reference(value.to_string())
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

    // logic: if an identifier is all uppercase, treat it as a reference,
    // otherwise, it's a string phoneme
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
                Ok(v.into())
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

    fn random_phoneme(&self, phoneme_key: &str) -> Option<Lemma> {
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

    use crate::lexcreate::PhoneticReference;

    use super::{LexPhonology, CreateValue};

    #[test]
    fn test_new_no_space(){
        let test_phon: PhoneticReference = "CCCC".into();
        let expected = PhoneticReference(vec!["C".into(), "C".into(), "C".into(), "C".into()]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_new_spaces() {
        let test_phon: PhoneticReference = "C V i C r rw".into();

        let expected = PhoneticReference(vec![
            CreateValue::Reference("C".to_string()),
            CreateValue::Reference("V".to_string()),
            CreateValue::Phoneme("i".to_string()),
            CreateValue::Reference("C".to_string()),
            CreateValue::Phoneme("r".to_string()),
            CreateValue::Phoneme("rw".to_string())
        ]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_new_no_space_mix(){
        let test_phon: PhoneticReference = "CCrC".into();
        let expected = PhoneticReference(vec![
            CreateValue::Reference("C".to_string()),
            CreateValue::Reference("C".to_string()),
            CreateValue::Phoneme("r".to_string()),
            CreateValue::Reference("C".to_string())
        ]);
        assert_eq!(test_phon, expected)
    }

    #[test]
    fn test_basic_gen() {
        let test_phon = LexPhonology{
            groups: HashMap::from([
                ("C".to_string(),
                vec![
                    PhoneticReference(vec![CreateValue::Phoneme("t".to_string())]), 
                    PhoneticReference(vec![CreateValue::Phoneme("r".to_string())])
                ]),
                ("V".to_string(), 
                vec![
                    PhoneticReference(vec![CreateValue::Phoneme("u".to_string())]),
                    PhoneticReference(vec![CreateValue::Phoneme("i".to_string())])
                ]),
                ("S".to_string(), 
                vec![
                    PhoneticReference(vec![
                        CreateValue::Reference("C".to_string()), 
                        CreateValue::Reference("V".to_string())
                    ]), 
                    PhoneticReference(vec![
                        CreateValue::Reference("V".to_string()), 
                        CreateValue::Reference("C".to_string())
                    ])
                ])
            ]),
            lexis_types: HashMap::from([
                ("words".to_string(), 
                vec![
                    PhoneticReference(vec![CreateValue::Reference("S".to_string())]), 
                    PhoneticReference(vec![
                        CreateValue::Reference("S".to_string()), 
                        CreateValue::Reference("S".to_string()) 
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