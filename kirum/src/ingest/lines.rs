use std::path::Path;
use anyhow::Result;
use crate::entries::{WordGraph, RawLexicalEntry};

pub fn ingest<P: AsRef<Path>>(file: P, overrides: RawLexicalEntry) -> Result<WordGraph> {
    let raw = std::fs::read_to_string(file)?;

    let mut working = WordGraph::default();
    for line in raw.split('\n') {
        let label = format!("ingest-{}", line);
        let entry = RawLexicalEntry{definition: line.to_string(), ..overrides.clone() };
        working.words.insert(label, entry);
    };

    Ok(working)
}


#[cfg(test)]
mod tests {
    use crate::{ingest::lines::ingest, entries::RawLexicalEntry};


    #[test]
    fn test_line_ingest(){
        let path = "src/test_files/test_ingest/basic_lines.txt";
        let res = ingest(path, RawLexicalEntry::default()).unwrap();
        println!("got basic data: {:#?}", res);
        assert_eq!(res.words.len(), 5);
    }
}