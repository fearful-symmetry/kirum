use clap::Parser;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI conlang utility for generating a language or language family based on etymological rules")]
#[clap(propagate_version = true)]
pub struct Args {
    // verbose output
    #[arg(short, long)]
    pub verbose: bool,
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
        /// JSON file of defined etymon transforms
        #[clap(short, long, value_parser)]
        transforms: Option<String>,
        /// json file of a language graph
        #[clap(short, long, value_parser)]
        graph: Option<String>,

        /// path to a directory to read in all transform and graph files
        #[clap(short, long, value_parser)]
        directory: Option<String>,
    },

    /// Render a lexicon from an existing set of graph files and transformations
    Render{
        /// JSON file of defined etymon transforms
        #[clap(short, long, value_parser)]
        transforms: Option<String>,
        /// JSON file of a language graph
        #[clap(short, long, value_parser)]
        graph: Option<String>,

        /// path to a directory to read in all transform and graph files
        #[clap(short, long, value_parser)]
        directory: Option<String>,

        #[clap(subcommand)]
        command: Format
    },

    /// Generate a language tree from another source
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
        graph: Option<String>,
        /// Path to transforms referenced in existing graph.
        #[clap(short, long, value_parser)]
        transforms: Option<String>,
        // path to a directory to read in all transform and graph files. Can be used instead of -t or -g
        #[clap(short, long, value_parser)]
        directory: Option<String>,
        /// Path to global transforms used for the daughter language; To generate daughter language, each transform with a conditional that evaluates to true will be applied
        #[clap(short='e', long, value_parser)]
        daughter_etymology: String,
        /// the ancestor language as specified in the "language" field of the graph definition.
        #[clap(short, long, value_parser)]
        ancestor: String,
        /// The name of the daughter language. This will become the "language" field in the daughter Lexis
        #[clap(short, long, value_parser)]
        name: String,
        /// Output file to write the new language file to
        #[clap(short, long, value_parser)]
        output: String
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Format{
     /// Print one word per line
    Line,
    /// Print language in CSV format
    Csv,
    /// Print language in format specified by a handlebars template file
    Template{
        /// Path to the .hbs template file
        #[clap(short, long, value_parser)]
        template_file: String,
        /// Optional rhai scripts for processing template data. See https://docs.rs/handlebars/latest/handlebars/#script-helper
        #[clap(short, long, value_parser)]
        rhai_files: Option<Vec<String>>
    }
}