mod entries;
mod cli;
use clap::Parser;
use anyhow::{Result, Context};
use entries::RawLexicalEntry;
use libkirum::{kirum::{LanguageTree, Transform, Lexis}, transforms::TransformFunc};
use std::{collections::HashMap};
use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let cli = cli::Args::parse();

    // let transforms = String::from("./example_files/example_transforms.json");
    // let graph = String::from("./example_files/example_graph.json");
    let mut lang_tree = read_from_files(cli.transforms, cli.graph)?;
    println!("rendering...");
    let computed = lang_tree.compute_lexicon();

    match cli.command{
        cli::Commands::Graphviz =>{
            println!("{}", computed.graphviz())
        },
        cli::Commands::Print =>{
            let rendered_dict = computed.reduce_to_dict(|_w|true);
            for word in rendered_dict{
                println!("{:?}", word);
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WordGraph {
    pub words: HashMap<String, RawLexicalEntry>,
}


fn read_from_files(transforms:String, graph:String) -> Result<LanguageTree>{
    // transforms 
    let trans_raw = std::fs::read_to_string(transforms.clone()).context(format!("error reading {}", transforms))?;
    let transforms: HashMap<String, Vec<TransformFunc>> = serde_json::from_str(&trans_raw)?;

    // map 
    let trans_raw = std::fs::read_to_string(graph.clone()).context(format!("error reading {}", graph))?;
    let raw_graph: WordGraph = serde_json::from_str(&trans_raw)?;
    

    let mut tree = LanguageTree::new();

    for (_, node) in raw_graph.words.clone(){
        let node_lex: Lexis = node.clone().into();

        if let Some(etymon) = node.etymology.clone(){
            for e in etymon.etymons{
                let trans = transforms.get(&e.transform).context(format!("transform {} does not exist", &e.transform))?;
                let ety_lex: RawLexicalEntry = raw_graph.words.get(&e.etymon).context(format!("etymon {} does not exist ", &e.etymon))?.clone();
                tree.connect_etynmology(node_lex.clone(), ety_lex.into(), Transform{name: e.transform, transforms: trans.to_vec(), agglutination_order: e.agglutination_order });
            }
        }
       
    }

    Ok(tree)
}