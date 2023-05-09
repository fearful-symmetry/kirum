mod entries;
mod cli;
mod files;
use clap::Parser;
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{Lexis, RenderedTree, TreeEtymology}, transforms::{Transform}};
use std::{fs::File, io::Write, path::{PathBuf}};
use csv::WriterBuilder;
use handlebars::Handlebars;
use env_logger::{Builder};
use log::LevelFilter;

#[macro_use]
extern crate log;

fn main() -> Result<()> {
    let cli = cli::Args::parse();

    let log_level: log::LevelFilter = if cli.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    Builder::new().filter_level(log_level).init();

    let out_data: String = match cli.command{
        cli::Commands::Graphviz{transforms, graph, directory} =>{
            let computed = read_and_compute(transforms, graph, directory)?;
            computed.graphviz()
        },
        cli::Commands::Render{command, transforms, graph, directory} =>{
            let computed = read_and_compute(transforms, graph, directory)?;
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
                cli::Generate::Daughter { graph, transforms, daughter_etymology, ancestor, name:lang_name, directory,  output } =>{
                    let mut computed = read_and_compute(transforms, graph, directory).context("error reading existing graph and transforms")?;
                    let trans_raw = std::fs::read_to_string(daughter_etymology.clone()).context(format!("error reading daughter transformation file {}", daughter_etymology))?;
                    let daughter_transform_map: files::TransformGraph = serde_json::from_str(&trans_raw).context("error parsing daughter transformations")?;

                    computed.walk_create_derivatives(|lex: Lexis| 
                        if lex.language == ancestor{
                            let mut found_updated: Lexis = lex;
                            let mut  transform_acc: Vec<Transform> = Vec::new();
                            for (name, trans) in &daughter_transform_map.transforms.clone(){
                                let new_trans = Transform{name: lang_name.clone(), ..trans.clone().into()};
                                let updated = new_trans.transform_option(&found_updated);
                                if let Some(upd) = updated{
                                    let conv: Transform = trans.clone().into();
                                    transform_acc.push(Transform { name: name.to_string(), ..conv });
                                    found_updated = Lexis{language: lang_name.to_owned(), ..upd};
                                    debug!("applied {}", name);
                                    //break;
                                }else{
                                    continue
                                }
                            }
                            return (Some(found_updated), Some(TreeEtymology{transforms: transform_acc, ..Default::default()}))
                        } else{
                            return (None, None);
                        }
                    );

                    // debug statements
                    // println!("{}", computed.graphviz());
                    let rendered_dict = computed.to_vec_etymons(|word|word.language == lang_name);

                    let mut acc = String::new();
                    for (word, ety) in rendered_dict.clone() {
                        acc = format!("{}\n{:?} | etymons: {:?}", acc, word, ety)
                    }
                    // TODO: write to file
                    // generate JSON that we can save to a file, re-ingest later
                    let graph = entries::create_json_graph(rendered_dict);
                    let graph_data = serde_json::to_string_pretty(&graph)?;
                    let mut file = File::create(output.clone())?;
                    write!(file, "{}", graph_data)?;
                    info!("wrote daughter {} to {}", lang_name, output);

                    acc
                }
                
            }
        }
    };

    if let Some(out_path) = cli.output{
        let mut out_file = File::create(out_path)?;
        write!(out_file, "{}", out_data)?;
    }else {
        info!("{}", out_data);    
    }
    

    Ok(())
}



// read in the existing files and generate a graph
// deals with the logic of listed files versus a specified directory
fn read_and_compute(transforms: Option<String>, graph: Option<String>, directory: Option<String>) -> Result<RenderedTree>{
    let (transform_files, graph_files): (Vec<PathBuf>, Vec<PathBuf>) = if transforms.is_some() && graph.is_some(){
        (vec![transforms.unwrap().into()], vec![graph.unwrap().into()])
    } else if directory.is_some(){
        files::handle_directory(directory.unwrap())?
    } else {
        return Err(anyhow!("must specify either a graph and transform file, or a directory"));
    }; 
    let mut lang_tree = files::read_from_files(transform_files, graph_files)?;
    debug!("rendering...");
    let computed = lang_tree.compute_lexicon();
    Ok(computed)
}

