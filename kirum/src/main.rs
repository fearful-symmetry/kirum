mod entries;
mod cli;
mod files;
mod tmpl;
use clap::Parser;
use anyhow::{Result, Context, anyhow};
use libkirum::{kirum::{Lexis, LanguageTree}, transforms::{Transform}};
use std::{fs::File, io::Write, path::PathBuf};
use csv::WriterBuilder;
use env_logger::{Builder};
use log::LevelFilter;

#[macro_use]
extern crate log;

fn main() -> Result<()> {
    let cli = cli::Args::parse();

    let log_level: log::LevelFilter = if cli.verbose == 0 {
        LevelFilter::Info
    } else if cli.verbose ==1 {
        LevelFilter::Debug
    } else {
        LevelFilter::Trace
    };
    Builder::new().filter_level(log_level).init();

    let out_data: String = match cli.command{
        cli::Commands::Graphviz{transforms, graph, directory} =>{
            let computed = read_and_compute(transforms, graph, directory)?;
            computed.graphviz()
        },
        cli::Commands::Render{command, transforms, graph, directory} =>{
            let computed = read_and_compute(transforms, graph, directory)?;
            let rendered_dict = computed.to_vec();
            match command{
                cli::Format::Line =>{
                    let mut acc = String::new();
                    for word in rendered_dict {
                        acc = format!("{}\n{:?}", acc, word)
                    }
                    acc
                },
                // cli::Format::Csv =>{
                //     let mut wrt = WriterBuilder::new().has_headers(true).from_writer(vec![]);
                //     for word in rendered_dict {
                //         wrt.serialize(word)?;
                        
                //     }
                //    String::from_utf8(wrt.into_inner()?)?
                // },
                cli::Format::Template { template_file, rhai_files } =>{
                    tmpl::generate_from_tmpl(rendered_dict, template_file, rhai_files)?
                }
                
            }
        },
        cli::Commands::Generate{command} =>{
            match command{
                cli::Generate::Daughter { graph, transforms, daughter_etymology, ancestor, name:lang_name, directory,  output } =>{
                    let mut computed = read_and_compute(transforms, graph, directory)
                    .context("error reading existing graph and transforms")?;
                    let trans_raw = std::fs::read_to_string(daughter_etymology.clone())
                    .context(format!("error reading daughter transformation file {}", daughter_etymology))?;
                    let daughter_transform_map: files::TransformGraph = serde_json::from_str(&trans_raw)
                    .context("error parsing daughter transformations")?;

                    let processed_transforms: Vec<Transform> = daughter_transform_map.transforms.into_iter()
                    .map(|(n, t)| Transform{name: n, ..t.into()}).collect();

                    computed.generate_daughter_language(lang_name.clone(), 
                    processed_transforms, |l| l.language == ancestor, 
                    |l| Lexis {tags: [l.tags.clone(), ["autogenerated".to_string()].to_vec()].concat(), ..l.clone()});

                    let rendered_dict = computed.to_vec_etymons(|word|word.language == lang_name);

                    let acc = String::new();
                    // for (word, ety) in rendered_dict.clone() {
                    //     acc = format!("{}\n{:?} | etymons: {:?}", acc, word, ety)
                    // }
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
fn read_and_compute(transforms: Option<String>, graph: Option<String>, directory: Option<String>) -> Result<LanguageTree>{
    let (transform_files, graph_files): (Vec<PathBuf>, Vec<PathBuf>) = if transforms.is_some() && graph.is_some(){
        (vec![transforms.unwrap().into()], vec![graph.unwrap().into()])
    } else if directory.is_some(){
        files::handle_directory(directory.unwrap())?
    } else {
        return Err(anyhow!("must specify either a graph and transform file, or a directory"));
    }; 
    let mut lang_tree = files::read_from_files(transform_files, graph_files)?;
    debug!("rendering...");
    lang_tree.compute_lexicon();
    Ok(lang_tree)
}

