use crate::entries::RawLexicalEntry;
use anyhow::{Result, anyhow};
use libkirum::word::PartOfSpeech;
use std::str::FromStr;

/// parse a list from the cli formatted as key=value into a RawLexicalEntry value.
/// currently only supports struct values that can be represented as strings.
pub fn parse(list: Vec<String>) -> Result<RawLexicalEntry> {
    let mut working = RawLexicalEntry::default();
    for val in list{
        let raw_values: Vec<&str> = val.split('=').collect();
        if raw_values.len() < 2 {
            return Err(anyhow!("could not parse {}, expecting key=value", val));
        }
        let stripped_val = raw_values[1].to_string();
        match raw_values[0] {
            "word" => working.word = Some(stripped_val.into()),
            "type" => working.word_type = Some(stripped_val),
            "language" => working.language = Some(stripped_val),
            "pos" => working.part_of_speech = Some(PartOfSpeech::from_str(&stripped_val)?),
            "archaic" => working.archaic = bool::from_str(&stripped_val)?,
            "tag" => working.tags = Some(vec![stripped_val]),
            "generate" => working.generate = Some(stripped_val),
            _ => {
                return Err(anyhow!("unknown value {} specified for override", raw_values[0]));
            }
        }
    }
    Ok(working)
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn test_override() {
        let list = vec!["generate=test_gen".to_string(), "pos=noun".to_string()];
        let parsed = parse(list).unwrap();
        assert_eq!(parsed.generate, Some(String::from("test_gen")));
        assert_eq!(parsed.part_of_speech, Some(libkirum::word::PartOfSpeech::Noun));
    }
}