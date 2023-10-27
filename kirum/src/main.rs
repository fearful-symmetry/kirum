mod entries;
mod cli;
mod files;
mod tmpl;
mod stat;
mod new;
mod generate;
mod ingest;
mod import;
mod global;

use clap::Parser;
use entries::create_json_graph;
use files::{read_and_compute, apply_def_vars};
use new::create_new_project;
use anyhow::{Result, Context};
use stat::gen_stats;
use std::{fs::File, io::Write};
//use csv::WriterBuilder;
use env_logger::Builder;
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
    if !cli.quiet {
        Builder::new().filter_level(log_level).init();    
    }
    

    let out_data: String = match cli.command.clone(){
        cli::Commands::New { name } => {
            create_new_project(&name)?;
            format!("created new project {}", name)
        },
        cli::Commands::Graphviz{directory} =>{
            let computed = read_and_compute(directory)?;
            computed.graphviz()
        },
        cli::Commands::Stat { directory } => {
            let computed = read_and_compute(directory)?;
            gen_stats(computed)
        },
        cli::Commands::Ingest {command, directory, out, overrides} => {
            import::ingest_from_cli(overrides, directory, out, command)?;
            String::from("")
        },
        cli::Commands::Render{command, directory, variables} =>{
            let computed = read_and_compute(directory)?;
            debug!("computed {} raw entries", computed.len());
            let mut rendered_dict = computed.to_vec();
            apply_def_vars(variables, &mut rendered_dict)?;
            debug!("rendered lexicon of {} lemmas", rendered_dict.len());
            match command{
                cli::Format::Line =>{
                    let mut acc = String::new();
                    for word in rendered_dict {
                        acc = format!("{}\n{:?}", acc, word)
                    }
                    acc
                },
                // CSV is disabled because of serializing issues
                // cli::Format::Csv =>{
                //     let mut wrt = WriterBuilder::new().has_headers(true).from_writer(vec![]);
                //     for word in rendered_dict {
                //         wrt.serialize(word)?;
                        
                //     }
                //    String::from_utf8(wrt.into_inner()?)?
                // },
                cli::Format::Template { template_file, rhai_files } =>{
                    tmpl::generate_from_tmpl(rendered_dict, template_file, rhai_files)?
                },
                cli::Format::Json => {
                    let words = computed.to_vec_etymons(|_|true);
                    let word_data = create_json_graph(words, |l| l.id, false)
                    .context("could not create map from language data")?;
                    serde_json::to_string_pretty(&word_data)?
                }
                
            }
        },
        cli::Commands::Generate{command} =>{
            match command{
                cli::Generate::Daughter { daughter_etymology, ancestor, 
                    name:lang_name, directory, output, group_by: separate_by_field } =>{
                    generate::daughter(daughter_etymology, 
                        ancestor, lang_name, directory, output, separate_by_field)?
                }
                
            }
        }
    };

    if let Some(out_path) = cli.output{
        let mut out_file = File::create(out_path)?;
        write!(out_file, "{}", out_data)?;
    }else if !out_data.is_empty() {
        println!("{}", out_data);
    }

    Ok(())
}



