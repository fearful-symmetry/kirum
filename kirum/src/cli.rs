use clap::Parser;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI conlang utility for generating a language or language family based on etymological rules")]
#[clap(propagate_version = true)]
pub struct Args {
    /// json file of defined etymon transforms
    #[clap(short, long, value_parser, default_value_t = String::from("transforms.json"))]
    pub transforms: String,
    /// json file of a language graph
    #[clap(short, long, value_parser, default_value_t = String::from("graph.json"))]
    pub graph: String,
    /// Output file; defaults to stdout if unspecified
    #[clap(short, long, value_parser)]
    pub output: Option<String>,

    #[clap(subcommand)]
    pub command: Commands
}

#[derive(clap::Subcommand, Clone)]
pub enum Commands{
    /// Print a graphviz representation of the language
    Graphviz,
    /// Print the rendered dictionary to stdout
    Render{
        #[clap(subcommand)]
        command: Format
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Format{
    Line,
    CSV,
    Template{
        #[clap(short, long, value_parser, default_value_t = String::from("template.tmpl"))]
        template_file: String
    }
}