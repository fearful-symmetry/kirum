use clap::Parser;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI conlang utility for generating a language or language family based on etymological rules")]
#[clap(propagate_version = true)]
pub struct Args {

    /// Output file; defaults to stdout if unspecified
    #[clap(short, long, value_parser)]
    pub output: Option<String>,

    #[clap(subcommand)]
    pub command: Commands
}

#[derive(clap::Subcommand, Clone)]
pub enum Commands{
    /// Print a graphviz representation of the language
    Graphviz{
        /// json file of defined etymon transforms
        #[clap(short, long, value_parser, default_value_t = String::from("transforms.json"))]
        transforms: String,
        /// json file of a language graph
        #[clap(short, long, value_parser, default_value_t = String::from("graph.json"))]
        graph: String,
    },

    /// Render a lexicon from an existing set of graph files and transformations
    Render{
        /// json file of defined etymon transforms
        #[clap(short, long, value_parser, default_value_t = String::from("transforms.json"))]
        transforms: String,
        /// json file of a language graph
        #[clap(short, long, value_parser, default_value_t = String::from("graph.json"))]
        graph: String,

        #[clap(subcommand)]
        command: Format
    },

    /// Generate a graph from another source
    Generate {
        #[clap(subcommand)]
        command: Generate
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Generate{
    /// Generate a daughter language from an existing language in a graph.
    Daughter{
        /// The file path to the existing language graph.
        #[clap(short, long, value_parser)]
        graph: String,
        /// Path to transforms referenced in existing graph.
        #[clap(short, long, value_parser)]
        transforms: String,
        /// Path to global transforms; To generate daughter language, each transform with a conditional that evaluates to true will be applied
        #[clap(short, long, value_parser)]
        daughter_transforms: String,
        /// the ancestor language as specified in the "language" field of the graph definition.
        #[clap(short, long, value_parser)]
        ancestor: String,
        /// The name of the daughter language. This will become the "language" field in the daughter Lexis
        #[clap(short, long, value_parser)]
        name: String
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Format{
     /// Print one word per line
    Line,
    /// Print language in CSV format
    CSV,
    /// Print language in format specified by a handlebars template file
    Template{
        /// Path to the .hbs template file
        #[clap(short, long, value_parser)]
        template_file: String
    }
}