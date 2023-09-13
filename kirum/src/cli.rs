use clap::{Parser, ValueEnum};

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI conlang utility for generating a language or language family based on etymological rules")]
#[clap(propagate_version = true)]
pub struct Args {
    /// Verbose output. Specify multiple times for increased logging.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Output file; defaults to stdout if unspecified
    #[clap(short, long, value_parser)]
    pub output: Option<String>,
    #[clap(short, long, default_value_t=false)]
    /// Do not print any log output
    pub quiet: bool,

    #[clap(subcommand)]
    pub command: Commands
}

#[derive(clap::Subcommand, Clone)]
pub enum Commands{
    /// Create a new language project with the specified name
    New{
        name: String
    },
    /// Print basic statistics on the language
    Stat {
        /// path to a directory to read in all transform and graph files
        #[clap(short, long, value_parser)]
        directory: Option<String>,
    },
    /// Print a graphviz representation of the language
    Graphviz{
        /// path to a directory to read in all transform and graph files
        #[clap(short, long, value_parser)]
        directory: Option<String>,
    },

    /// Render a lexicon from an existing set of graph files and transformations
    Render{
        /// path to a directory to read in all transform and graph files.
        /// Can be specified instead of -g -d
        #[clap(short, long, value_parser)]
        directory: Option<String>,
        /// TOML file that will be used to resolve template variables in definition fields.
        /// Template variables can be written into Lexis definition fields using {{handlebars_variables}}
        #[clap(short, long, value_parser)]
        variables: Option<String>,

        #[clap(subcommand)]
        command: Format
    },

    /// Generate a language tree from another source
    Generate {
        #[clap(subcommand)]
        command: Generate
    },

    /// Create a language tree file from an external source, such as a JSON file or newline-delimited list of words.
    /// When run, `ingest` will create a file with a separate lexis entry for each specified word.
    #[clap(verbatim_doc_comment)]
    Ingest {
        /// Override a default ingest value that will be applied to all ingested words, specified in key=value form.
        /// Keys can be any value normally written into a lexis entry in a tree file passed to `render`.
        #[clap(short, long, value_parser, verbatim_doc_comment)]
        overrides: Option<Vec<String>>,
        /// Path to a directory to read in all transform and graph files. Can be used instead of -t or -g
        #[clap(short, long, value_parser, default_value="./ingested")]
        directory: String,
        #[clap(short='f', long, value_parser, default_value="ingested.json")]
        out: String,
        #[clap(subcommand)]
        command: Ingest
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Ingest {
    /// Derive a language tree from a formatted JSON file
    Json{
        /// JSON file to ingest
        file: String,
    },
    /// Derive a language tree from a newline-delimited list of words
    Lines {
        /// a newline-delimited list of words to ingest
        file: String,
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum Generate{
    /// Generate a daughter language from an existing language in a graph.
    Daughter{
        /// path to a directory to read in all transform and graph files. Can be used instead of -t or -g
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
        /// Output file to write the new language file to. If group_by is set, this is used as the directory prefix.
        #[clap(short, long, value_parser)]
        output: String,
        /// group output into different files
        #[clap(short='b', long, value_enum)]
        group_by: Option<SeparateValues>
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum SeparateValues {
    Word,
    LexisType,
    Archaic,
}

#[derive(clap::Subcommand, Clone, PartialEq, PartialOrd)]
pub enum Format{
     /// Print one word per line
    Line,
    // Print language in CSV format
    //Csv,
    /// Print language in format specified by a handlebars template file
    Template{
        /// Path to the .hbs template file
        #[clap(short, long, value_parser)]
        template_file: String,
        /// Optional rhai scripts for processing template data. See https://docs.rs/handlebars/latest/handlebars/#script-helper
        #[clap(short, long, value_parser)]
        rhai_files: Option<Vec<String>>
    },
    /// Prints a JSON object of the language
    Json
}