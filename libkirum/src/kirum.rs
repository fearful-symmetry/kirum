use crate::transforms::Transform;
use crate::word::{Word, PartOfSpeech, Etymology, Edge};
use petgraph::Direction::Incoming;
use petgraph::dot::{Dot, Config};
use petgraph::graph::EdgeReference;
use petgraph::stable_graph::NodeIndex;
use petgraph::{Graph, visit::EdgeRef};
//use serde_with::skip_serializing_none;



#[derive(Clone, Default, PartialEq, serde::Deserialize, serde::Serialize)]
/// A Lexis represents a single entry in the language tree, be it a word, word stem, morpheme, etc.
pub struct Lexis {
    /// Optional ID for the lex. Useful when exporting out of the language tree structure
    pub id: String,
    pub word: Option<Word>,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<PartOfSpeech>,
    pub lexis_type: String,
    pub definition: String,
    pub archaic: bool,
    #[serde(skip)]
    pub tags: Vec<String>
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


#[derive(Default, Debug, Clone)]
pub struct TreeEtymology {
    pub transforms: Vec<Transform>,
    pub intermediate_word: Option<Word>,
    pub agglutination_order: Option<i32>
}

impl TreeEtymology{
    // a helper function to apply the given lexis to all transforms in the graph edge
    pub fn apply_transforms(&self, etymon: &Lexis) -> Lexis {

        let mut transformed = etymon.clone();
        for trans in self.transforms.clone(){
            transformed = trans.transform(&transformed);
        }

        transformed
    }

    // A helper function that returns a vector of all names transforms in the graph edges
    pub fn names(&self) -> Vec<String>{
       self.transforms.clone().into_iter().map(|t| t.name).collect()
    }
}


pub struct LanguageTree {
    //the Node type represents a lexical entry, the edge is a tuple of the transform, and a "holding" string that's used to "trickle down" words as they're generated
    graph: Graph<Lexis, TreeEtymology>,
}

impl Default for LanguageTree{
    fn default() -> Self {
        Self::new()
    }
}

/// LanguageTree stores the working state of a language graph.
impl LanguageTree {
    pub fn new() -> Self {
        LanguageTree {graph: Graph::<Lexis, TreeEtymology, petgraph::Directed>::new()}

    }

    /// Adds a single lexis entry to the language tree. 
    pub fn add_lexis(&mut self, lex: Lexis){
        self.graph.add_node(lex);
    }

    /// A quick and ugly helper that returns a graphviz.dot render of the graph. Useful for debugging.
    pub fn graphviz(&self) -> String{
       format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel])) 
    }

    /// creates an etymological link between two words: an upstream etymon, and a base word. If neither word exists, they will be added.
    pub fn connect_etymology(&mut self, lex: Lexis, etymon: Lexis, trans: Vec<Transform>, agglutination_order: Option<i32>){
        let mut lex_idx: Option<NodeIndex> = None;
        let mut ety_idx: Option<NodeIndex> = None;

        //no word in tree, add both of them
        if self.graph.node_count() == 0{
            lex_idx = Some(self.graph.add_node(lex.clone()));
            ety_idx = Some(self.graph.add_node(etymon.clone()));
        }

        if ety_idx.is_none() && lex_idx.is_none(){
           for nx in self.graph.node_indices(){ 
                if self.graph[nx] == lex && lex_idx.is_none(){
                    lex_idx = Some(nx);
                    continue;
                }
                if self.graph[nx] == etymon && ety_idx.is_none(){
                    ety_idx = Some(nx);
                    continue;
                }
                if ety_idx.is_some() && lex_idx.is_some(){
                    break;
                }
            }
        }

        if lex_idx.is_none(){
            lex_idx = Some(self.graph.add_node(lex));
        }
        if ety_idx.is_none(){
            ety_idx = Some(self.graph.add_node(etymon));
        }

        self.graph.add_edge(ety_idx.unwrap(), lex_idx.unwrap(), TreeEtymology { transforms: trans, intermediate_word: None, agglutination_order: agglutination_order });

    }

    /// Fill out the graph, walking the structure until all possible words have been generated.
    /// This returns a rendered tree that represents the final computed language family.
    pub fn compute_lexicon(&mut self) -> RenderedTree{
        let mut incomplete = true;
        let mut rendered = self.graph.clone();
        // for each node:
        // if the node has a hard-coded word, write transformed words to downstream edges
        // if all upstream edges are filled, write the word
        while incomplete {
            let mut changes = 0;
            for node in rendered.node_indices(){

                //we have a hard-coded word
                if rendered[node].word.is_some(){
                    // iterate over downstream edges
                    for edge in rendered.clone().edges_directed(node, petgraph::Direction::Outgoing){
                        if edge.weight().intermediate_word.is_some(){
                            continue
                        }
                        //we have an unfilled edge, generate stem
                        let mut existing = edge.weight().clone();
                        let transformed = existing.apply_transforms(&rendered[node]);
                        //println!("updated edge with word {:?}", transformed.word);
                        existing.intermediate_word = transformed.word;
                        changes+=1;
    
                        let node_target = edge.target();
                            rendered.update_edge(node, node_target, existing);
                       
                    }
                } else {
                    // we have a node with no word, see if we can fill it
                    let mut is_ready = true;
                    let mut upstreams: Vec<(i32, Word)> = Vec::new();
                    
                    for edge in rendered.clone().edges_directed(node, petgraph::Direction::Incoming){
                        if edge.weight().intermediate_word.is_none(){
                            // word still has unpopulated edges, give up
                            is_ready = false;
                            break;
                        }
                        let order = edge.weight().agglutination_order.unwrap_or(0);
                        upstreams.push((order, edge.weight().intermediate_word.clone().unwrap()));
                        
                    }
                    if is_ready{
                        changes+=1;
                        let rendered_word = join_string_vectors(&mut upstreams);
                        //println!("updated node with word: {:?}", rendered_word);
                        rendered[node].word = Some(rendered_word);
                    }
                }
    
            }
            //println!("made {} changes", changes);
            if changes == 0 {
                incomplete = false;
            }
        }

        RenderedTree { graph: rendered }
    }

}


/// RenderedTree is the final generated language family tree, as generated by a LanguageTree object.
pub struct RenderedTree {
    graph: Graph<Lexis, TreeEtymology>,
}

impl RenderedTree{
    /// reduce the language graph to a vector of words that match the provided function.
    pub fn to_vec<F>(self, filter: F) -> Vec<Lexis>
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

    /// reduce the language graph to a vector of words that match the provided function. Returns a vector of tuples for each matching word and any associated etymological data.
    pub fn to_vec_etymons<F>(self, filter: F) -> Vec<(Lexis, Etymology)> 
    where 
    F: Fn(&Lexis) -> bool,
    {
        let mut word_vec: Vec<(Lexis, Etymology)> = Vec::new();
        for node in self.graph.node_indices(){
           if self.graph[node].word.is_some(){
                if filter(&self.graph[node]){
                    let mut etymon_list: Vec<Edge> = Vec::new();
                    for etymon in self.graph.neighbors_directed(node, Incoming){
                        let ety_link: Vec<EdgeReference<TreeEtymology>> = self.graph.edges_connecting(etymon, node).collect();
                        let mut transform_name: Vec<String> = Vec::new();
                        let mut agg_order: Option<i32> = None;
                        if let Some(trans_link) = ety_link.get(0){
                            let trans_data =  trans_link.weight();
                            transform_name =  trans_data.names();
                            agg_order = trans_data.agglutination_order;
                        }
                        etymon_list.push(Edge{etymon: self.graph[etymon].id.clone(), transform: transform_name, agglutination_order: agg_order});
                    }
                    word_vec.push((self.graph[node].clone(), Etymology{etymons: etymon_list}));
                }
           }
        }

        word_vec
    }

    pub fn graphviz(&self) -> String{
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))  
    }
    /// Walk through each word in the tree, applying the walk_function closure. The closure takes the a Lexis value, and returns a tuple of two optional Lexis and Transform values.
    /// If the closure returns `Some()` for the Lexis value, the enclosed Lexis will be added as a derivative word to the tree.
    pub fn walk_create_derivatives(&mut self, mut walk_function: impl FnMut(Lexis)->(Option<Lexis>, Option<TreeEtymology>)){
        for node in self.graph.node_indices(){
            let (new, trans) = walk_function(self.graph[node].clone());
            if let Some(der_word) = new{
                let new_node = self.graph.add_node(der_word);
                self.graph.add_edge(node, new_node, trans.unwrap_or_default());
            }
        }
    }
}

fn join_string_vectors(words: &mut [(i32, Word)]) -> Word{
    words.sort_by_key(|k| k.0);
    let merged: Vec<String> = words.iter().flat_map(|s| s.1.clone().chars()).collect();
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