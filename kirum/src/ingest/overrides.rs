use crate::entries::RawLexicalEntry;
use anyhow::{Result, anyhow};
use libkirum::word::PartOfSpeech;
use std::str::FromStr;

/// parse a list from the cli formatted as key=value into a RawLexicalEntry value.
/// currently only supports struct values that can be represented as strings.
pub fn parse(list: Vec<String>) -> Result<RawLexicalEntry> {
    let mut working = RawLexicalEntry::default();
    for val in list{
        match val.as_ref() {
            "word" => working.word = Some(val.into()),
            "type" => working.word_type = Some(val),
            "language" => working.language = Some(val),
            "pos" => working.part_of_speech = Some(PartOfSpeech::from_str(&val)?),
            "archaic" => working.archaic = bool::from_str(&val)?,
            "tag" => working.tags = Some(vec![val]),
            "generate" => working.generate = Some(val),
            _ => {
                return Err(anyhow!("unknown value {} specified for override", val));
            }
        }
    }

    Ok(working)
}