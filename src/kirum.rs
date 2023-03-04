use crate::transforms::TransformFunc;
use crate::word::{Word, PartOfSpeech};
use crate::reader::{self, RawLexicalEntry, WordGraph};
use crate::errors::LangError;

use petgraph::{Graph, visit::EdgeRef};
use std::collections::HashMap;


#[derive(Clone)]
pub struct Lexis {
    pub id: String,
    pub word: Option<Word>,
    pub language: String,
    pub pos: Option<PartOfSpeech>,
    pub lexis_type: String,
    pub definition: String,
    pub archaic: bool,
    pub tags: Vec<String>
}

impl From<RawLexicalEntry> for Lexis{
    fn from(source: RawLexicalEntry) -> Self {
        Lexis { id: String::new(), 
            word: source.word, 
            language: source.language.unwrap_or("".to_string()), 
            pos: source.part_of_speech, 
            lexis_type: source.word_type.unwrap_or("".to_string()), 
            definition: source.definition, 
            archaic: source.archaic,
            tags: source.tags.unwrap_or(Vec::new())}
    }
}

impl Default for Lexis{
    fn default() -> Self {
        Lexis { id: String::new(), 
            word: None, 
            language: String::new(), 
            pos: None, 
            lexis_type: String::new(),
             definition: String::new(), 
             archaic: false, 
             tags: Vec::new() }
    }
}

impl std::fmt::Debug for Lexis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rendered_args: String = match &self.word{
            Some(word_vec) =>{
                word_vec.to_string()
            },
            None => String::from("None")
        };
        
        rendered_args = format!("{} ({})", rendered_args, self.language);
        
        if let Some(pos) = &self.pos{
            rendered_args = format!("{}: ({:?})", rendered_args, pos);
        }
        rendered_args = format!("{} {}", rendered_args, &self.definition);

        f.write_str(&rendered_args)
    }
}

#[derive(Clone)]
pub struct Transform {
    pub name: String,
    pub transforms: Vec<TransformFunc>,
    pub agglutination_order: Option<i32>,
}

impl std::fmt::Debug for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.name))
    }
}

impl Transform{
    pub fn transform(&self, etymon: &Lexis) -> Lexis {
        let mut updated = etymon.clone();
        for transform in &self.transforms {
            updated = transform.transform(updated);
        };

        updated
    }
}

pub struct LanguageTree {
    //the Node type represents a lexical entry, the edge is a tuple of the transform, and a "holding" string that's used to "trickle down" words as they're generated
    graph: Graph<Lexis, (Transform, Option<Word>)>
}

impl LanguageTree {
    pub fn new_from_files(transform_filepath: String, graph_filepath: String) -> Result<Self, LangError> {
        // transformation rules
        let transform_list = reader::get_transforms(transform_filepath)?;
        let trans_raw = std::fs::read_to_string(graph_filepath).map_err(LangError::JSONImportError)?;
        // words
        let raw_graph: WordGraph = serde_json::from_str(&trans_raw).map_err(LangError::JSONSerdeError)?;
        let raw_map = reader::render_graph(raw_graph, transform_list)?;
        // generate
        let mut tree = LanguageTree { graph: raw_map };
        tree.compute_lexicon();
    
        Ok(tree)
    }
    /// create a new language tree from  a map of transforms 
    pub fn new(transforms: HashMap<String, Vec<TransformFunc>>, graph: WordGraph) -> Result<Self, LangError> {
        let raw_map = reader::render_graph(graph, transforms)?;
        let mut tree = LanguageTree { graph: raw_map };
        tree.compute_lexicon();

        Ok(tree)
    }
    /// reduce the language graph to a dictionary of words that match the provided function
    pub fn reduce_to_dict<F>(self, filter: F) -> Vec<Lexis>
    where
    F: Fn(&Lexis) -> bool,
    {
        let mut dict: Vec<Lexis> = Vec::new();
        for node in self.graph.node_indices(){
            if self.graph[node].word.is_some() && filter(&self.graph[node]) {
                dict.push(self.graph[node].clone());
            }
        }
        dict.sort_by_key(|k| k.word.clone().unwrap());
        dict
    }

    /// Fill out the graph, walking the structure until all possible words have been generated
    pub fn compute_lexicon(&mut self) {
        let mut incomplete = true;
    
        // for each node:
        // if the node has a hard-coded word, write transformed words to downstream edges
        // if all upstream edges are filled, write the word
        while incomplete {
            let mut changes = 0;
            for node in self.graph.node_indices(){
                //we have a hard-coded word
                if self.graph[node].word.is_some(){
                    // iterate over downstream edges
                    for edge in self.graph.clone().edges_directed(node, petgraph::Direction::Outgoing){
                        if edge.weight().1.is_some(){
                            continue
                        }
                        //we have an unfilled edge, generate stem
                        let mut existing = edge.weight().clone();
                        let transformed = existing.0.transform(&self.graph[node]);
                        println!("updated edge with word {:?}", transformed.word);
                        existing.1 = transformed.word;
                        changes+=1;
    
                        let node_target = edge.target();
                        self.graph.update_edge(node, node_target, existing);
                       
                    }
                } else {
                    // we have a node with no word, see if we can fill it
                    let mut is_ready = true;
                    let mut upstreams: Vec<(i32, Word)> = Vec::new();
                    for edge in self.graph.clone().edges_directed(node, petgraph::Direction::Incoming){
                        if edge.weight().1.is_none(){
                            // word still has unpopulated edges, give up
                            is_ready = false;
                            break;
                        }
                        let order = edge.weight().0.agglutination_order.unwrap_or(0);
                        upstreams.push((order, edge.weight().1.clone().unwrap()));
                    }
                    if is_ready{
                        changes+=1;
                        let rendered = join_string_vectors(&mut upstreams);
                        println!("updated node with word: {:?}", rendered);
                        self.graph[node].word = Some(rendered);
                    }
                }
    
            }
            println!("made {} changes", changes);
            if changes == 0 {
                incomplete = false;
            }
        }
    }

   // pub fn add_lexis(lexis: )
}

fn join_string_vectors(words: &mut Vec<(i32, Word)>) -> Word{
    words.sort_by_key(|k| k.0);
    let merged: Vec<String> = words.iter().map(|s| s.1.clone().chars()).flatten().collect();
    merged.into()
}

// pub fn get_etymon(graph: &Graph<WordDef, transforms::Transform>, node: petgraph::graph::NodeIndex) -> WordDef {
//     // base case, we have a word
//     if graph[node].word.is_some() {
//         return graph[node].clone()
//     }

//     let mut upstreams: Vec<(i32, WordDef)> = Vec::new();
//     for upstream in graph.neighbors_directed(node, petgraph::Direction::Incoming) {
//         println!("found upstream etymon {:?} for {:?}", graph[upstream], graph[node]);
//         // get parent word
//         let upstream_root = get_etymon(&graph, upstream);
//         println!("etymon {:?} has etymon {:?}", graph[upstream], upstream_root.clone());
//         // get edges for transformations
//         let edges = graph.edges_connecting(upstream, node);
//         for edge in edges{
//             println!("applying edge transform {:?} to etymon {:?}", edge, upstream_root.clone());
//             let transformed = edge.weight().transform(&upstream_root.clone());
//             let order = edge.weight().agglutination_order.unwrap_or(0);

//             println!("--transformed to: {:?}", transformed.word);
//             upstreams.push((order, transformed));
//         }

//     };
//     upstreams.sort_by_key(|k| k.0);
//     println!("combined etymons for word are: {:?}", upstreams);
//     // this is probably not what we want for "reducing" a list of multiple stems,
//     // as it will just blindly add all the letters together
//     return join_words(upstreams, graph[node].clone());
// } 