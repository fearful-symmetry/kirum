use std::collections::HashMap;

use libkirum::kirum::LanguageTree;
use tabled::{Tabled, Table, settings::{object::FirstRow, Disable, panel::Header}};

#[derive(Default, Tabled)]
struct Stats {
    nouns: i64,
    verbs: i64,
    adjectives: i64,
    total: usize
}

pub fn gen_stats(tree: LanguageTree) -> String {
    let mut languages: HashMap<String, i64> = HashMap::new();
    let mut types: HashMap<String, i64> = HashMap::new();
    let mut stats = Stats{total: tree.len(), ..Stats::default()};
    for lex in tree.into_iter() {
        if let Some(pos) = lex.pos {
            match pos {
                libkirum::word::PartOfSpeech::Adjective => stats.adjectives+=1,
                libkirum::word::PartOfSpeech::Verb => stats.verbs+=1,
                libkirum::word::PartOfSpeech::Noun => stats.nouns+=1
            }
        }
        let lang_name = match lex.language.as_str() {
            "" => "None Set",
            st => st,
        };
        let new_lang_count = languages.get(lang_name).unwrap_or(&0)+1;
        languages.insert(lang_name.to_string(), new_lang_count);
        
        let new_type_count = types.get(&lex.lexis_type).unwrap_or(&0)+1;
        types.insert(lex.lexis_type, new_type_count);
    }


    let stats_vec = vec![stats];
    let stat_str = Table::new(stats_vec).to_string();
    let lang_str = Table::new(languages)
    .with(Disable::row(FirstRow)).with(Header::new("Languages")).to_string();
    let type_str = Table::new(types)
    .with(Disable::row(FirstRow)).with(Header::new("Types")).to_string();
    format!("\n{}\n{}\n{}\n", stat_str, lang_str, type_str)
}