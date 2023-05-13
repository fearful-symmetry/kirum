use std::collections::HashMap;

use crate::transforms::Transform;
use crate::word::{Word, PartOfSpeech, Etymology, Edge};
use petgraph::Direction::Incoming;
use petgraph::dot::{Dot, Config};
use petgraph::graph::EdgeReference;
use petgraph::stable_graph::NodeIndex;
use petgraph::{Graph, visit::EdgeRef};
use log::debug;

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

#[derive(Clone)]
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

    /// the same as connect_etymology, but takes a string ID for the upstream etymon. If no etymon matching the ID could be found, the method returns false
    pub fn connect_etymology_id(&mut self, lex: Lexis, etymon_id: String, trans: Vec<Transform>, agglutination_order: Option<i32>) -> bool{
        // TODO: this should take both lexii as an ID
        let upstream_lex = self.graph.node_indices().find(|l| self.graph[*l].id == etymon_id);
        match upstream_lex {
            Some(etymon) => {
                self.connect_etymology(lex, self.graph[etymon].clone(), trans, agglutination_order);
                true
            }
            None => false
        }
    }

    // Older version of this algo
    /// Fill out the graph, walking the structure until all possible words have been generated.
    /// This returns a rendered tree that represents the final computed language family.
    // pub fn compute_lexicon(&mut self) -> RenderedTree{
    //     let mut incomplete = true;
    //     let mut rendered = self.graph.clone();
    //     // for each node:
    //     // if the node has a hard-coded word, write transformed words to downstream edges
    //     // if all upstream edges are filled, write the word
    //     while incomplete {
    //         let mut changes = 0;
    //         for node in rendered.node_indices(){

    //             //we have a hard-coded word
    //             if rendered[node].word.is_some(){
    //                 // iterate over downstream edges
    //                 for edge in rendered.clone().edges_directed(node, petgraph::Direction::Outgoing){
    //                     if edge.weight().intermediate_word.is_some(){
    //                         continue
    //                     }
    //                     //we have an unfilled edge, generate stem
    //                     let mut existing = edge.weight().clone();
    //                     let transformed = existing.apply_transforms(&rendered[node]);
    //                     //println!("updated edge with word {:?}", transformed.word);
    //                     existing.intermediate_word = transformed.word;
    //                     changes+=1;
    
    //                     let node_target = edge.target();
    //                         rendered.update_edge(node, node_target, existing);
                       
    //                 }
    //             } else {
    //                 // we have a node with no word, see if we can fill it
    //                 let mut is_ready = true;
    //                 let mut upstreams: Vec<(i32, Word)> = Vec::new();
                    
    //                 for edge in rendered.clone().edges_directed(node, petgraph::Direction::Incoming){
    //                     if edge.weight().intermediate_word.is_none(){
    //                         // word still has unpopulated edges, give up
    //                         is_ready = false;
    //                         break;
    //                     }
    //                     let order = edge.weight().agglutination_order.unwrap_or(0);
    //                     upstreams.push((order, edge.weight().intermediate_word.clone().unwrap()));
                        
    //                 }
    //                 if is_ready{
    //                     changes+=1;
    //                     let rendered_word = join_string_vectors(&mut upstreams);
    //                     //println!("updated node with word: {:?}", rendered_word);
    //                     rendered[node].word = Some(rendered_word);
    //                 }
    //             }
    
    //         }
    //         //println!("made {} changes", changes);
    //         if changes == 0 {
    //             incomplete = false;
    //         }
    //     }

    //     RenderedTree { graph: rendered }
    // }

    /// Fill out the graph, walking the structure until all possible lexii have been generated or updated
    /// This method is idempotent, and can be run any time to calculate unpopulated out incorrect lexii in the language tree.
    pub fn compute_lexicon(&mut self) {
        let mut incomplete = true;
        let mut updated: HashMap<NodeIndex, bool> = HashMap::new();
        while incomplete{
            let mut changes = 0;

            for node in self.graph.node_indices(){

                let mut is_ready = true;
                let mut upstreams: Vec<(i32, Word)> = Vec::new();
                
                if !updated.contains_key(&node){
                    let mut etymons_in_lex = 0;
                    for edge in self.graph.clone().edges_directed(node, petgraph::Direction::Incoming){
                        etymons_in_lex += 1;
                        if edge.weight().intermediate_word.is_none(){
                            // word still has unpopulated edges, give up
                            is_ready = false;
                            break;
                            
                        }
                        // add our populated edge to the list, be prepared to use it
                        let order = edge.weight().agglutination_order.unwrap_or(0);
                        upstreams.push((order, edge.weight().intermediate_word.clone().unwrap()));
                    }

                    // word has all populated upstream edges
                    if etymons_in_lex > 0 && is_ready{
                        changes+=1;
                        let rendered_word = join_string_vectors(&mut upstreams);
                        debug!("updated node {} with word: {:?}", self.graph[node].id, rendered_word);
                        self.graph[node].word = Some(rendered_word);
                        updated.insert(node, true);
                    }
                    // we have a lexis with no upstream edges, but contains a word. mark as updated.
                    if self.graph[node].word.is_some() && etymons_in_lex == 0 {
                        debug!("updated node {} with no upstreams: {:?}", self.graph[node].id, self.graph[node].word);
                        changes+=1;
                        updated.insert(node, true);
                    }
                }


                // if a word is updated, "trickle down" to outgoing edges
                if updated.contains_key(&node){
                    for edge in self.graph.clone().edges_directed(node, petgraph::Direction::Outgoing){
                        // do we need this check?
                        if edge.weight().intermediate_word.is_some(){
                            continue
                        }
                        //we have an unfilled edge, generate stem
                        let mut existing = edge.weight().clone();
                        let transformed = existing.apply_transforms(&self.graph[node]);
                        debug!("updated edge with word {:?}", transformed.word);
                        existing.intermediate_word = transformed.word;
                        changes+=1;
    
                        let node_target = edge.target();
                        self.graph.update_edge(node, node_target, existing);
                       
                    }
                }
            }

            // we iterated the graph without making any changes, consider it done
            if changes == 0 {
                incomplete = false;
            }
        }
    }

    /// Walk through each word in the tree, applying the walk_function closure. The closure takes a Lexis value, and returns a tuple of two optional Lexis and Transform values.
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

    /// A more sophisticated version of walk_create_derivatives, this will generate a daughter language
    /// by applying the given transformations to any Lexis that matches the apply_to closure
    /// After a word is generated, 
    pub fn generate_daughter_language<F, P>(&mut self, daughter_name: String, 
        daughter_transforms: Vec<Transform>, 
        mut apply_to: F, 
        mut postprocess: P) 
    where
    F: FnMut(&Lexis) -> bool,
    P: FnMut(&Lexis) -> Lexis,
    {
        for node in self.graph.node_indices() {
            if apply_to(&self.graph[node]) {
                let mut applied_transforms: Vec<Transform> = Vec::new();
                let mut found_updated: Lexis = self.graph[node].clone();
                for trans in &daughter_transforms {
                    let updated = trans.transform_option(&found_updated);
                    if let Some(upd) = updated {
                        applied_transforms.push(trans.clone());
                        found_updated = upd;
                    }
                }
                
                found_updated.language = daughter_name.clone();
                found_updated = postprocess(&found_updated);
                
                let new_node = self.graph.add_node(found_updated);
                self.graph.add_edge(node, new_node, TreeEtymology { transforms: applied_transforms, ..Default::default() });
            }
        }
        // self.walk_create_derivatives(|potential_etymon|
        // if apply_to(&potential_etymon){
        //     let mut found_updated: Lexis = potential_etymon;
        //     let mut  transform_acc: Vec<Transform> = Vec::new();
            
        //     for trans in &daughter_transforms{
        //         let updated = trans.transform_option(&found_updated);
        //         if let Some(upd) = updated {
        //             let new_word = if let Some(post) = postprocess {
        //                 post(&upd)
        //             } else {
        //                 upd
        //             };
        //         }
        //     };
        //     (Some(found_updated),  Some(TreeEtymology{transforms: transform_acc, ..Default::default()}))
        // } else {
        //     (None, None)
        // });
    }

    

    /// reduce the language graph to a vector of words
    pub fn to_vec(&self) -> Vec<Lexis>{
        let mut dict: Vec<Lexis> = Vec::new();
        for node in self.graph.node_indices(){
            if self.graph[node].word.is_some() {
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
   

}




/// RenderedTree is the final generated language family tree, as generated by a LanguageTree object.
// #[derive(Clone)]
// pub struct RenderedTree {
//     graph: Graph<Lexis, TreeEtymology>,
// }

// impl RenderedTree{
//     /// reduce the language graph to a vector of words that match the provided function.
//     pub fn to_vec<F>(self, filter: F) -> Vec<Lexis>
//     where
//     F: Fn(&Lexis) -> bool,
//     {
//         let mut dict: Vec<Lexis> = Vec::new();
//         for node in self.graph.node_indices(){
//             if self.graph[node].word.is_some() && filter(&self.graph[node]) {
//                 dict.push(self.graph[node].clone());
//             }
//         }
//         dict.sort_by_key(|k| k.word.clone().unwrap());
//         dict

//     }

//     /// reduce the language graph to a vector of words that match the provided function. Returns a vector of tuples for each matching word and any associated etymological data.
//     pub fn to_vec_etymons<F>(self, filter: F) -> Vec<(Lexis, Etymology)> 
//     where 
//     F: Fn(&Lexis) -> bool,
//     {
//         let mut word_vec: Vec<(Lexis, Etymology)> = Vec::new();
//         for node in self.graph.node_indices(){
//            if self.graph[node].word.is_some(){
//                 if filter(&self.graph[node]){
//                     let mut etymon_list: Vec<Edge> = Vec::new();
//                     for etymon in self.graph.neighbors_directed(node, Incoming){
//                         let ety_link: Vec<EdgeReference<TreeEtymology>> = self.graph.edges_connecting(etymon, node).collect();
//                         let mut transform_name: Vec<String> = Vec::new();
//                         let mut agg_order: Option<i32> = None;
//                         if let Some(trans_link) = ety_link.get(0){
//                             let trans_data =  trans_link.weight();
//                             transform_name =  trans_data.names();
//                             agg_order = trans_data.agglutination_order;
//                         }
//                         etymon_list.push(Edge{etymon: self.graph[etymon].id.clone(), transform: transform_name, agglutination_order: agg_order});
//                     }
//                     word_vec.push((self.graph[node].clone(), Etymology{etymons: etymon_list}));
//                 }
//            }
//         }

//         word_vec
//     }

//     pub fn graphviz(&self) -> String{
//         format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))  
//     }
//     /// Walk through each word in the tree, applying the walk_function closure. The closure takes the a Lexis value, and returns a tuple of two optional Lexis and Transform values.
//     /// If the closure returns `Some()` for the Lexis value, the enclosed Lexis will be added as a derivative word to the tree.
//     pub fn walk_create_derivatives(&mut self, mut walk_function: impl FnMut(Lexis)->(Option<Lexis>, Option<TreeEtymology>)){
//         for node in self.graph.node_indices(){
//             let (new, trans) = walk_function(self.graph[node].clone());
//             if let Some(der_word) = new{
//                 let new_node = self.graph.add_node(der_word);
//                 self.graph.add_edge(node, new_node, trans.unwrap_or_default());
//             }
//         }
//     }
// }

fn join_string_vectors(words: &mut [(i32, Word)]) -> Word{
    words.sort_by_key(|k| k.0);
    let merged: Vec<String> = words.iter().flat_map(|s| s.1.clone().chars()).collect();
    merged.into()
}

#[cfg(test)]
mod tests {

    use crate::{kirum::{LanguageTree, Lexis}, transforms::{Transform, LetterArrayValues, TransformFunc, self, LetterValues}, matching::{LexisMatch, Value}};

    fn create_basic_words() -> LanguageTree {
        let parent = Lexis{id: "parent".to_string(), word: Some("wrh".into()), language: "gauntlet".to_string(), lexis_type: "root".to_string(), ..Default::default()};
        let derivative_one = Lexis{id: "derivative_one".to_string(), word: None, lexis_type: "word".to_string(), ..parent.clone()};
        let derivative_two = Lexis{id: "derivative_two".to_string(), word: None, lexis_type: "word".to_string(), ..parent.clone()};

        let transform_one = Transform{name: "first_transform".to_string(), 
        lex_match: None, 
        transforms: vec![TransformFunc::LetterArray { letters: vec![LetterArrayValues::Place(0), LetterArrayValues::Char("a".into()), LetterArrayValues::Place(1), LetterArrayValues::Place(2)] }]
        };

        let transform_two = Transform{name: "second_transform".to_string(),
        lex_match: None,
        transforms: vec![TransformFunc::Prefix { value: "au".into() }],
    };

        // a basic three-word graph, two words auto-generated
        let mut tree = LanguageTree::new();
        tree.connect_etymology(derivative_one.clone(), parent, vec![transform_one], None);
        tree.connect_etymology(derivative_two, derivative_one, vec![transform_two], None);

        tree
    }

    #[test]
    fn test_basic_tree(){
        let mut tree = create_basic_words();

        tree.compute_lexicon();

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().to_string()).collect();

        assert_eq!(out_words.contains(&"wrh".to_string()), true);
        assert_eq!(out_words.contains(&"warh".to_string()), true);
        assert_eq!(out_words.contains(&"auwarh".to_string()), true);
        
    }

    #[test]
    fn test_agglutination(){
        let mut tree = create_basic_words();
        let parent_part = Lexis{id: "parent_part".to_string(), word: Some("maark".into()), language: "gauntlet".to_string(), lexis_type: "word".to_string(), ..Default::default()};
        let combined_word = Lexis{id: "combined_words".to_string(), word: None, ..parent_part.clone()};

        let agg_transform = vec![Transform{name: "agg_transform".to_string(), lex_match: None, transforms: vec![TransformFunc::Loanword]}];

        tree.connect_etymology(combined_word.clone(), parent_part, agg_transform.clone(), Some(0));
        tree.connect_etymology_id(combined_word, "derivative_one".to_string(), agg_transform , Some(1));

        tree.compute_lexicon();
        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().to_string()).collect();

        assert_eq!(out_words.contains(&"maarkwarh".to_string()), true);
       // tree.connect_etymology(lex, etymon, trans, agglutination_order)
    }

    #[test]
    fn test_lexis_overwrite() {
        let proto_word = Lexis{id: "proto_word".to_string(), word: Some("vrh".into()), language: "proto-gauntlet".to_string(), lexis_type: "stem".to_string(), ..Default::default()};
        let root = Lexis{id: "parent".to_string(), word: Some("wrh".into()), language: "gauntlet".to_string(), lexis_type: "root".to_string(), ..Default::default()};
        
        let proto_transform = Transform{name: "proto-transform".to_string(), 
        lex_match: None, 
        transforms: vec![TransformFunc::LetterReplace { letter: LetterValues{old: "w".to_string(), new: "v".to_string()}, replace: transforms::LetterReplaceType::All }]};
        
        let mut tree = create_basic_words();
        tree.connect_etymology(root, proto_word, vec![proto_transform], None);

        tree.compute_lexicon();
        let out = tree.to_vec();
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().to_string()).collect();

        println!("{}", tree.graphviz());

        assert_eq!(out_words.contains(&"vrh".to_string()), true);
        assert_eq!(out_words.contains(&"varh".to_string()), true);
        assert_eq!(out_words.contains(&"auvarh".to_string()), true);
    }

    #[test]
    fn test_idempotent(){
        let mut tree = create_basic_words();

        tree.compute_lexicon();
        // run again
        tree.compute_lexicon();

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().to_string()).collect();

        assert_eq!(out_words.contains(&"wrh".to_string()), true);
        assert_eq!(out_words.contains(&"warh".to_string()), true);
        assert_eq!(out_words.contains(&"auwarh".to_string()), true);

    }

    #[test]
    fn test_daughter_basic(){
        let mut tree = create_basic_words();

        let daughter_transforms = vec![Transform{
            name: "test_transform_1".to_string(),
            lex_match: Some(LexisMatch{
                id: None,
                word: None,
                language: None,
                pos: None,
                lexis_type: Some(Value::Match(crate::matching::ValueMatch::Equals(crate::matching::EqualValue::String("word".to_string())))),
                archaic: None,
                tags: None
            }),
            transforms: vec![
                TransformFunc::LetterReplace { letter: LetterValues { old: "w".to_string(), new: "k".to_string() }, replace: transforms::LetterReplaceType::All },
                TransformFunc::LetterRemove { letter: "u".to_string(), position: transforms::LetterRemovePosition::All },
            ]
        }];

        tree.compute_lexicon();

        tree.generate_daughter_language("High Gauntlet".to_string(), daughter_transforms, |lex|lex.language == "gauntlet".to_string(), |lex| Lexis {tags: vec!["tested".to_string()], ..lex.clone() });

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().filter(|lex|lex.tags.len() > 0).map(|l| l.word.unwrap_or_default().to_string()).collect();

        assert_eq!(out_words.contains(&"karh".to_string()), true);
        assert_eq!(out_words.contains(&"akarh".to_string()), true);
    }
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