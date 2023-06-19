mod entries;
mod cli;
mod files;
mod tmpl;
mod stat;
mod new;
mod generate;
use clap::Parser;
use files::read_and_compute;
use new::create_new_project;
use anyhow::Result;
use stat::gen_stats;
use std::{fs::File, io::Write};
//use csv::WriterBuilder;
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
        cli::Commands::New { name } => {
            create_new_project(&name)?;
            format!("created new project {}", name)
        },
        cli::Commands::Graphviz{transforms, graph, directory} =>{
            let computed = read_and_compute(transforms, graph, directory)?;
            computed.graphviz()
        },
        cli::Commands::Stat { directory } => {
            let computed = read_and_compute(None, None, directory)?;
            gen_stats(computed)
        }
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
                }
                
            }
        },
        cli::Commands::Generate{command} =>{
            match command{
                cli::Generate::Daughter { graph, transforms, daughter_etymology, ancestor, 
                    name:lang_name, directory, output, group_by: separate_by_field } =>{
                    generate::daughter(graph, transforms, daughter_etymology, 
                        ancestor, lang_name, directory, output, separate_by_field)?
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



