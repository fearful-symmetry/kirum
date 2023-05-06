mod entries;
mod cli;
use clap::Parser;
use anyhow::{Result, Context};
use entries::{RawLexicalEntry, RawTransform};
use libkirum::{kirum::{LanguageTree, Lexis, RenderedTree}, transforms::{Transform, TransformFunc}};
use std::{collections::HashMap, fs::File, io::Write};
use serde::{Deserialize, Serialize};
use csv::WriterBuilder;
use handlebars::Handlebars;

fn main() -> Result<()> {
    let cli = cli::Args::parse();

    let out_data: String = match cli.command{
        cli::Commands::Graphviz{transforms, graph} =>{
            let computed = read_and_compute(transforms, graph)?;
            computed.graphviz()
        },
        cli::Commands::Render{command, transforms, graph} =>{
            let computed = read_and_compute(transforms, graph)?;
            let rendered_dict = computed.to_vec(|_w|true);
            match command{
                cli::Format::Line =>{
                    let mut acc = String::new();
                    for word in rendered_dict {
                        acc = format!("{}\n{:?}", acc, word)
                    }
                    acc
                },
                cli::Format::CSV =>{
                    let mut wrt = WriterBuilder::new().has_headers(true).from_writer(vec![]);
                    for word in rendered_dict {
                        wrt.serialize(word)?;
                        
                    }
                   String::from_utf8(wrt.into_inner()?)?
                },
                cli::Format::Template { template_file } =>{
                    let template_string = std::fs::read_to_string(template_file.clone()).context(format!("error reading template file {}", template_file))?;
                    let reg = Handlebars::new();
                    reg.render_template(&template_string, &rendered_dict)?

                }
                
            }
        },
        cli::Commands::Generate{command} =>{
            match command{
                cli::Generate::Daughter { graph, transforms, daughter_transforms, ancestor, name:lang_name } =>{
                    let mut computed = read_and_compute(transforms, graph).context("error reading existing graph and transforms")?;
                    let trans_raw = std::fs::read_to_string(daughter_transforms.clone()).context(format!("error reading daughter transformation file {}", daughter_transforms))?;
                    let daughter_transform_map: HashMap<String, RawTransform> = serde_json::from_str(&trans_raw).context("error parsing daughter transformations")?;

                    computed.walk_create_derivatives(|lex: Lexis| 
                        if lex.language == ancestor{
                            let mut found_updated: Lexis = lex;
                            let mut  transform_acc: Vec<TransformFunc> = Vec::new();
                            for (name, trans) in &daughter_transform_map.clone(){
                                let new_trans = Transform{name: lang_name.clone(), ..trans.clone().into()};
                                let updated = new_trans.transform_option(&found_updated);
                                if let Some(upd) = updated{
                                    transform_acc = [transform_acc, trans.transforms.clone()].concat();
                                    found_updated = Lexis{language: lang_name.to_owned(), ..upd};
                                    println!("applied {}", name);
                                    //break;
                                }else{
                                    continue
                                }
                            }
                            return (Some(found_updated), Some(Transform { name: format!("Ancestor {}", lang_name), lex_match: None, transforms: transform_acc, agglutination_order: None }));
                        } else{
                            return (None, None);
                        }
                    );

                    
                    
                    // this generates correct data
                    println!("{}", computed.graphviz());
                    let rendered_dict = computed.to_vec_etymons(|word|word.language == lang_name);
                    let mut acc = String::new();
                    for (word, ety) in rendered_dict.clone() {
                        acc = format!("{}\n{:?} | etymons: {:?}", acc, word, ety)
                    }
                    // generate JSON that we can save to a file, re-ingest later
                    // next steps: write out transforms to JSON, let code read in multiple graph files
                    let graph = entries::create_json_graph(rendered_dict);
                    let graph_data = serde_json::to_string_pretty(&graph)?;
                    println!("{}", graph_data);

                    acc
                }
                
            }
        }
    };

    if let Some(out_path) = cli.output{
        let mut out_file = File::create(out_path)?;
        write!(out_file, "{}", out_data)?;
    }else {
        println!("{}", out_data);    
    }
    

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WordGraph {
    pub words: HashMap<String, RawLexicalEntry>,
}


fn read_and_compute(transforms: String, graph:String) -> Result<RenderedTree>{
    let mut lang_tree = read_from_files(transforms, graph)?;
    println!("rendering...");
    let computed = lang_tree.compute_lexicon();
    Ok(computed)
}

fn read_from_files(transforms:String, graph:String) -> Result<LanguageTree>{
    // transforms 
    let trans_raw = std::fs::read_to_string(transforms.clone()).context(format!("error reading transformation file {}", transforms))?;
    let transforms: HashMap<String, RawTransform> = serde_json::from_str(&trans_raw)?;

    // map 
    let graph_raw = std::fs::read_to_string(graph.clone()).context(format!("error reading graph file {}", graph))?;
    let raw_graph: WordGraph = serde_json::from_str(&graph_raw)?;
    

    let mut tree = LanguageTree::new();

    for (lex_name, node) in raw_graph.words.clone(){
        let node_lex: Lexis = Lexis { id: lex_name, ..node.clone().into() };

        if let Some(etymon) = node.etymology.clone(){
            for e in etymon.etymons{
                let trans = transforms.get(&e.transform).context(format!("transform {} does not exist", &e.transform))?;
                let ety_lex: RawLexicalEntry = raw_graph.words.get(&e.etymon).context(format!("etymon {} does not exist ", &e.etymon))?.clone();
                tree.connect_etymology(node_lex.clone(), Lexis { id: e.etymon, ..ety_lex.into()}, Transform{name: e.transform, transforms: trans.transforms.to_vec(), agglutination_order: e.agglutination_order, lex_match: trans.conditional.clone()});
            }
        }
       
    }

    Ok(tree)
}