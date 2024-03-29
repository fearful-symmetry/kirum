use std::collections::HashMap;

use crate::errors::TransformError;
use crate::lemma::Lemma;
use crate::lexcreate;
use crate::transforms::{Transform, GlobalTransform};
use crate::word::{PartOfSpeech, Etymology, Edge};
use petgraph::Direction::{Incoming, Outgoing, self};
use petgraph::dot::{Dot, Config};
use petgraph::graph::EdgeReference;
use petgraph::stable_graph::NodeIndex;
use petgraph::Graph;
use log::{trace, debug};

#[derive(Clone, Default,  serde::Deserialize, serde::Serialize)]
/// A Lexis represents a headword in Kirum's lexicon, be it a word, word stem, morpheme, etc.
pub struct Lexis {
    /// Optional ID for the lex, used by connect_etymology_id
    pub id: String,
    /// The Word associated with this Lexis entry. If set to none, libkirum will attempt to derive the word during compute_lexicon().
    pub word: Option<Lemma>,
    /// The language of the Lexis
    pub language: String,
    /// Part Of Speech
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pos: Option<PartOfSpeech>,
    /// Optional value that can be used for different morpheme types. Stem, root, word, etc.
    pub lexis_type: String,
    /// Dictionary definition
    pub definition: String,
    /// Marks the lexis as archaic. Currently not used by any internal methods.
    pub archaic: bool,
    /// Optional user-supplied tags
    //#[serde(skip)]
    pub tags: Vec<String>,
    /// Optional user-supplied metadata. Unlike tags, historical_metadata will trickle down to any derivative words.
    /// This shared metadata can be used to track common qualities of words, for filtering, templating, etc
    pub historical_metadata: HashMap<String, String>,
    /// Optional field that can be used to randomly generate a word value if none exists, separate from any etymology.
    /// If the given word has no etymology, this value takes prescience.
    /// The string value is used to generate a word based on the underlying phonology rules supplied to the TreeEtymology structure.
    pub word_create: Option<String>
}

// this custom implementation exists because we don't want history metadata to count towards equality
// as the metadata field might shift while the graph is still being built.
impl PartialEq for Lexis {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id &&
        self.word == other.word &&
        self.language == other.language &&
        self.pos == other.pos &&
        self.lexis_type == other.lexis_type && 
        self.definition == other.definition && 
        self.archaic == other.archaic &&
        self.tags == other.tags && 
        self.word_create == other.word_create

    }
    fn ne(&self, other: &Self) -> bool {
        ! self.eq(other)
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


/// TreeEtymology represents the graph edge of the language tree, and
/// determines the relationship of one word to another.
#[derive(Default, Debug, Clone)]
pub struct TreeEtymology {
    /// A list of Transforms that define the etymology between one word and another.
    pub transforms: Vec<Transform>,
    intermediate_word: Option<Lemma>,
    /// Determines what order this morpheme is agglutinated in to create derived lexii.
    /// For example, if a lexis has two upstream etymons, Word A with agglutination_order=1 
    /// and Word B with agglutination_order=2, the lexis will by generated by agglutinating A+B
    pub agglutination_order: Option<i32>,
}

impl TreeEtymology{
    /// a helper function to apply the given lexis to all transforms in the graph edge
    fn apply_transforms(&self, etymon: &mut Lexis) -> Result<(), TransformError>{

        //let mut transformed = etymon.clone();
        for trans in self.transforms.clone(){
            trans.transform(etymon)?;
        };
        Ok(())
    }

    /// A helper function that returns a vector of all names transforms in the graph edges
    pub fn names(&self) -> Vec<String>{
       self.transforms.clone().into_iter().map(|t| t.name).collect()
    }
}

/// Represents an entire language family tree as tracked by libkirum.
#[derive(Clone)]
pub struct LanguageTree {
    //the Node type represents a lexical entry, the edge is a tuple of the transform, and a "holding" string that's used to "trickle down" words as they're generated
    graph: Graph<Lexis, TreeEtymology>,

    /// A set of phonology rules that can be used generate new words without etymology.
    /// Using these rules, Kirum will randomly stitch together phonemes to create a new lexis.
    pub word_creator_phonology: lexcreate::LexPhonology,

    /// An optional set of global transforms.
    /// If specified, every word in the tree will be matched to the global transform list, 
    /// and the transform will be applied _after_ any other matching transform
    pub global_transforms: Option<Vec<GlobalTransform>>
}

impl Default for LanguageTree{
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for LanguageTree {
    type Item = Lexis;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let mut lexii: Vec<Lexis> = Vec::new();
        for node in self.graph.node_indices() {
            lexii.push(self.graph[node].clone());
        }
        lexii.into_iter()
    }
}


impl LanguageTree {
    pub fn new() -> Self {
        LanguageTree {graph: Graph::<Lexis, TreeEtymology, petgraph::Directed>::new(), 
            word_creator_phonology: lexcreate::LexPhonology { groups: HashMap::new(), lexis_types: HashMap::new() },
            global_transforms: None,
        }

    }

    /// Adds a single lexis entry to the language tree. 
    pub fn add_lexis(&mut self, lex: Lexis){
        self.graph.add_node(lex);
    }
    /// Returns true if the language contains a given word
    pub fn contains(&self, lex: &Lexis) -> bool {
        for nx in self.graph.node_indices(){ 
            if &self.graph[nx] == lex {
                return true
            }
        }

        false
    }

    /// returns the total number of words
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// returns true if the language tree is empty
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0

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


        if ety_idx.is_none(){
            ety_idx = Some(self.graph.add_node(etymon));
        }
 
        if lex_idx.is_none(){
            lex_idx = Some(self.graph.add_node(lex));
        }

        self.graph.add_edge(ety_idx.unwrap(), lex_idx.unwrap(), TreeEtymology { transforms: trans, intermediate_word: None, agglutination_order });

    }

    /// the same as connect_etymology, but takes a string ID for the upstream etymon. If no etymon matching the ID could be found, the method returns false
    pub fn connect_etymology_id(&mut self, lex: Lexis, etymon_id: String, trans: Vec<Transform>, agglutination_order: Option<i32>) -> bool{
        let upstream_lex = self.graph.node_indices().find(|l| self.graph[*l].id == etymon_id);
        match upstream_lex {
            Some(etymon) => {
                self.connect_etymology(lex, self.graph[etymon].clone(), trans, agglutination_order);
                true
            }
            None => false
        }
    }


    /// Fill out the graph, walking the structure until all possible lexii have been generated or updated.
    /// This method is idempotent, and can be run any time to calculate unpopulated or incorrect lexii in the language tree.
    pub fn compute_lexicon(&mut self) -> Result<(), TransformError> {
        let mut incomplete = true;
        let mut updated: HashMap<NodeIndex, bool> = HashMap::new();
        while incomplete{
            let mut changes = 0;

            for node in self.graph.node_indices(){

                let mut is_ready = true;
                let mut upstreams: Vec<(i32, Lemma)> = Vec::new();
                
                if !updated.contains_key(&node){

                    // try word generation from supplied phonetic rules first, before transforms
                    if self.graph[node].word_create.is_some() && self.graph[node].word.is_none() {
                        trace!("word_create has value, no word found, creating one...");
                        let word_type = self.graph[node].word_create.clone().unwrap();
                        let new_gen = self.word_creator_phonology.create_word(&word_type);
                        if let Some(found_new) = new_gen {
                            let debug_iter: Vec<String> = found_new.clone().into_iter().collect();
                            trace!("created new word ({:?}) from phonology rules for ID {}", debug_iter, self.graph[node].id);
                            self.graph[node].word = Some(found_new);
                            //continue;
                        }
                    }

                    let mut etymons_in_lex = 0;
                    for edge in self.graph.edges_directed(node, petgraph::Direction::Incoming){
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

                    // word has all populated upstream edges, add to tree proper
                    if etymons_in_lex > 0 && is_ready{
                        changes+=1;
                        let rendered_word = join_string_vectors(&mut upstreams);

                        trace!("updated node {} with word: {:?}", self.graph[node].id, rendered_word);
                        self.graph[node].word = Some(rendered_word);
                        updated.insert(node, true);


                        // merge upstream historical metadata
                        self.combine_maps_for_lex_idx(&node);
                        // check global transforms
                        if let Some(gt) = &self.global_transforms  {
                            let mut updating = self.graph[node].clone();
                            let etys: Vec<&Lexis> = self.graph.neighbors_directed(node, Direction::Incoming).map(|e| &self.graph[e]).collect();
                            for trans in gt {
                                // collect the upstream etymons
                                trans.transform(&mut updating, Some(&etys))?;
                                trace!("updated word {:?} with global transform ", self.graph[node].id);
                            }
                            self.graph[node] = updating;
                        }
                    }
                    // we have a lexis with no upstream edges, but contains a word. mark as updated.
                    if self.graph[node].word.is_some() && etymons_in_lex == 0 {
                        trace!("updated node '{}' with no upstreams: {:?}", self.graph[node].id, self.graph[node].word);
                        changes+=1;
                        updated.insert(node, true);
                    }
                }


                // if a word is updated, "trickle down" to outgoing edges
                if updated.contains_key(&node){
                    let mut edges = self.graph.neighbors_directed(node, Outgoing).detach();
                    while let Some(edge) = edges.next_edge(&self.graph) {
                         // do we need this check?
                        if self.graph[edge].intermediate_word.is_some(){
                            continue
                        }
                        let mut temp_ref = self.graph[node].clone();
                        self.graph[edge].apply_transforms(&mut temp_ref)?;
                        //self.graph[node] = temp_ref;
                        trace!("updated edge with word {:?}", temp_ref.word);

                        self.graph[edge].intermediate_word = temp_ref.word;
                        changes+=1;
                    }

                }
            }

            // we iterated the graph without making any changes, consider it done
            if changes == 0 {
                incomplete = false;
            }
        };
        Ok(())
    }

    fn combine_maps_for_lex_idx(&mut self,  id: &NodeIndex) {
        let etys: Vec<Lexis> = self.graph.neighbors_directed(*id, Direction::Incoming).map(|e| self.graph[e].clone()).collect();
        for ety in etys {
            if !ety.historical_metadata.is_empty(){
                self.graph[*id].historical_metadata.extend(ety.historical_metadata.iter().map(|(k, v)| (k.clone(), v.clone())));
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

    /// A more sophisticated version of walk_create_derivatives(), this will generate a daughter language
    /// by applying the given transformations to any Lexis that matches the apply_to closure. If a given lexis matches apply_to
    /// but does not match any transforms, it will be added to the daughter language without any transformations.
    /// After a word is generated, the postprocess closure will be called on the lexis before it's added to the language tree.
    pub fn generate_daughter_language<F, P>(&mut self, daughter_name: String, 
        daughter_transforms: Vec<Transform>, 
        mut apply_to: F, 
        mut postprocess: P) -> Result<(), TransformError>
    where
    F: FnMut(&Lexis) -> bool,
    P: FnMut(&Lexis) -> Lexis,
    {
        for node in self.graph.node_indices() {
            if apply_to(&self.graph[node]) {
                debug!("Created daughter word from {}", &self.graph[node].id);
                let mut applied_transforms: Vec<Transform> = Vec::new();
                let mut found_updated: Lexis = self.graph[node].clone();
                for trans in &daughter_transforms {
                    let updated = trans.transform_option(&mut found_updated)?;
                    if updated {
                        applied_transforms.push(trans.clone());
                        //found_updated = upd;
                    }
                }
                
                found_updated.language = daughter_name.clone();
                found_updated = postprocess(&found_updated);
                
                let new_node = self.graph.add_node(found_updated);
                self.graph.add_edge(node, new_node, TreeEtymology { transforms: applied_transforms, ..Default::default() });
                
            }
        };
        Ok(())
    }

    

    /// Reduce the language graph to a vector of words.
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

    /// Get a Lemma entry by the ID value
    pub fn get_by_id(&self, id: &str) -> Option<Lexis> {
        for node in self.graph.node_indices(){ 
            if self.graph[node].id == id {
                return Some(self.graph[node].clone())
            }
        }
        None
    }


    /// Reduce the language graph to a vector of words that match the provided function. 
    /// Returns a vector of tuples for each matching word and any associated etymological data.
    pub fn to_vec_etymons<F>(&self, filter: F) -> Vec<(Lexis, Etymology)> 
    where 
    F: Fn(&Lexis) -> bool,
    {
        let mut word_vec: Vec<(Lexis, Etymology)> = Vec::new();
        for node in self.graph.node_indices(){
            if self.graph[node].word.is_some() && filter(&self.graph[node]){
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
                        etymon_list.push(Edge{etymon: self.graph[etymon].id.clone(), transforms: Some(transform_name), agglutination_order: agg_order});
                    }
                    word_vec.push((self.graph[node].clone(), Etymology{etymons: etymon_list}));
            }
        }

        word_vec
    }
   

}


fn join_string_vectors(words: &mut [(i32, Lemma)]) -> Lemma{
    words.sort_by_key(|k| k.0);
    let merged: Vec<String> = words.iter().flat_map(|s| s.1.clone().chars()).collect();
    merged.into()
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use log::LevelFilter;
    use crate::{kirum::{LanguageTree, Lexis}, transforms::{Transform, LetterArrayValues, TransformFunc, self, LetterValues, GlobalTransform}, matching::{LexisMatch, Value, ValueMatch, EqualValue}, lexcreate::LexPhonology, lemma::Lemma};
    use env_logger::Builder;


    fn create_basic_words() -> LanguageTree {
        let parent = Lexis{id: "parent".to_string(), word: Some("wrh".into()), language: "gauntlet".to_string(), 
        historical_metadata: HashMap::from([("test".to_string(), "t".to_string())]), lexis_type: "root".to_string(), ..Default::default()};
        let derivative_one = Lexis{id: "derivative_one".to_string(), word: None, 
        historical_metadata: HashMap::from([("derivative".to_string(), "one".to_string())]), lexis_type: "word".to_string(), ..parent.clone()};
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

    fn create_basic_with_globals() -> LanguageTree {
        let mut test_tree = create_basic_words();
        let transforms = vec![GlobalTransform{
            lex_match: LexisMatch { id: None, word: None, 
                language: Some(Value::Match(ValueMatch::Equals(EqualValue::String("New Gauntlet".to_string())))), ..Default::default() },
            etymon_match: Some(LexisMatch {
                    language: Some(Value::Match(ValueMatch::Equals(EqualValue::String("gauntlet".to_string())))), ..Default::default()}),
            transforms: vec![TransformFunc::Prefix { value: "ka".into() }]
        }];
        test_tree.global_transforms = Some(transforms);

        test_tree

    }

    #[test]
    fn basic_global_transforms() {
        Builder::new().filter_level(LevelFilter::Info).init();
        let mut test_tree = create_basic_with_globals();

        let derivative_lang = Lexis{id: "derivative_lang".to_string(), 
            word: None, lexis_type: "word".to_string(), language: "New Gauntlet".to_string(), ..Default::default()};

        test_tree.connect_etymology_id(derivative_lang, "derivative_two".to_string(),
         vec![Transform{name: "test".to_string(), lex_match: None, transforms: vec![TransformFunc::Prefix { value: Lemma::from("sur") }]}], None);

        test_tree.compute_lexicon().unwrap();
        let test_word = test_tree.to_vec_etymons(|f| f.language == "New Gauntlet".to_string());
        assert_eq!(test_word[0].0.word.clone().unwrap(), Lemma::from("kasurauwarh"))
    }

    #[test]
    fn test_metadata_derives(){
        let mut test_tree = create_basic_with_globals();
        test_tree.compute_lexicon().unwrap();

        let final_dict = test_tree.to_vec();
        for word in final_dict {
            assert_eq!((Some(&"t".to_string())), word.historical_metadata.get("test"))
        }
    }

    #[test]
    fn metadata_multiple_object() {
        let mut test_tree = create_basic_with_globals();
        test_tree.compute_lexicon().unwrap();

        let final_dict = test_tree.to_vec();
        for word in final_dict {
            match word.id.as_str() {
               "parent" => {
                    assert_eq!(HashMap::from([("test".to_string(), "t".to_string())]), word.historical_metadata)
                },
                "derivative_one"=> {
                    assert_eq!(HashMap::from([("test".to_string(), "t".to_string()), ("derivative".to_string(), "one".to_string())]), word.historical_metadata)
                }, 
                "derivative_two" => {
                    assert_eq!(HashMap::from([("test".to_string(), "t".to_string()), ("derivative".to_string(), "one".to_string())]), word.historical_metadata)
                }
                _ => {assert!(false, "bad map value in test")}
            }
        }
    }

    #[test]
    fn metadata_out_of_order() {
        let parent = Lexis{id: "parent".to_string(), word: Some("wrh".into()), language: "gauntlet".to_string(), 
        historical_metadata: HashMap::from([("test".to_string(), "t".to_string())]), lexis_type: "root".to_string(), ..Default::default()};
        let derivative_one = Lexis{id: "derivative_one".to_string(), word: None, 
        historical_metadata: HashMap::from([("derivative".to_string(), "one".to_string())]), lexis_type: "word".to_string(), ..parent.clone()};
        let derivative_two = Lexis{id: "derivative_two".to_string(), word: None, lexis_type: "word".to_string(), ..parent.clone()};

        let transform_one = Transform{name: "first_transform".to_string(), 
        lex_match: None, 
        transforms: vec![TransformFunc::LetterArray { letters: vec![LetterArrayValues::Place(0), LetterArrayValues::Char("a".into()), LetterArrayValues::Place(1), LetterArrayValues::Place(2)] }]
        };

        let transform_two = Transform{name: "second_transform".to_string(),
        lex_match: None,
        transforms: vec![TransformFunc::Prefix { value: "au".into() }],
        };

        let mut tree = LanguageTree::new();

        tree.add_lexis(derivative_one.clone());
        tree.connect_etymology_id(derivative_two, derivative_one.id.clone(), vec![transform_two], None);
        tree.connect_etymology(derivative_one, parent, vec![transform_one], None);


        tree.compute_lexicon().unwrap();

        let final_dict = tree.to_vec();
        for word in final_dict {
            assert_eq!((Some(&"t".to_string())), word.historical_metadata.get("test"))
        }
    }

    #[test]
    fn global_with_local_transform() {
        let mut test_tree = create_basic_with_globals();

        let derivative_lang = Lexis{id: "derivative_lang".to_string(), 
            word: None, lexis_type: "word".to_string(), language: "New Gauntlet".to_string(), ..Default::default()};

        test_tree.connect_etymology_id(derivative_lang, "derivative_two".to_string(),
         vec![Transform{name: "test".to_string(), lex_match: None, transforms: vec![TransformFunc::Loanword]}], None);

        test_tree.compute_lexicon().unwrap();
        let test_word = test_tree.to_vec_etymons(|f| f.language == "New Gauntlet".to_string());
        assert_eq!(test_word[0].0.word.clone().unwrap(), Lemma::from("kaauwarh"))
    }

    #[test]
    fn global_with_downstream_transform(){
        let mut test_tree = create_basic_with_globals();
        let derivative_lang = Lexis{id: "derivative_lang".to_string(), word: None, 
            lexis_type: "word".to_string(), language: "New Gauntlet".to_string(), ..Default::default()};

        let derivative_new_word = Lexis{
            id: "derivative_word".to_string(),
            word: None,
            lexis_type: "word".to_string(),
            language: "New Gauntlet".to_string(),
            ..Default::default()
        };

        test_tree.connect_etymology_id(derivative_lang, "derivative_two".to_string(),
        vec![Transform{name: "test".to_string(), lex_match: None, transforms: vec![TransformFunc::Loanword]}], None);

        test_tree.connect_etymology_id(derivative_new_word, "derivative_lang".to_string(), 
        vec![Transform{name: "test_downstream".to_string(), lex_match: None, transforms: vec![TransformFunc::Postfix { value: "`sh".into() }]}], 
        None);

        test_tree.compute_lexicon().unwrap();
        let test_words = test_tree.to_vec_etymons(|f| f.language == "New Gauntlet".to_string());
        assert_eq!(test_words.iter().find(|e| e.0.word == Some(Lemma::from("kaauwarh"))).is_some(), true);
        

        assert_eq!(test_words.iter().find(|e| e.0.word == Some(Lemma::from("kaauwarh`sh"))).is_some(), true);
       
    }

    #[test]
    fn test_word_create() {
        //let log_level = LevelFilter::Trace;
        //Builder::new().filter_level(log_level).init();
        let test_phon_rules = LexPhonology{
            groups: HashMap::from([
                ('C',
                vec![
                    "h".try_into().unwrap(), 
                    "r".try_into().unwrap(),
                    "x".try_into().unwrap(),
                    "k".try_into().unwrap()
                ]),
                ('V', 
                vec![
                    "u".try_into().unwrap(),
                    "i".try_into().unwrap()
                ]),
            ]),
            lexis_types: HashMap::from([
                ("root".to_string(), 
                vec![
                    "CCC".try_into().unwrap()
                ])
            ]),
        };
        let parent = Lexis{id: "parent".to_string(), word:None, 
        language: "gauntlet".to_string(), lexis_type: "root".to_string(), word_create: Some("root".to_string()), ..Default::default()};
        let derivative_one = Lexis{id: "derivative_one".to_string(), word: None, lexis_type: "word".to_string(), word_create: None, ..parent.clone()};
        let derivative_two = Lexis{id: "derivative_two".to_string(), word: None, lexis_type: "word".to_string(), word_create: None, ..parent.clone()};

        let transform_one = Transform{name: "first_transform".to_string(), 
        lex_match: None, 
        transforms: vec![TransformFunc::LetterArray { 
            letters: vec![LetterArrayValues::Place(0),
            LetterArrayValues::Char("a".into()),
            LetterArrayValues::Place(1), 
            LetterArrayValues::Place(2)] }]
        };

        let transform_two = Transform{name: "second_transform".to_string(),
        lex_match: None,
        transforms: vec![TransformFunc::Prefix { value: "au".into() }],
        };
        let mut tree = LanguageTree::new();
        tree.connect_etymology(derivative_one.clone(), parent, vec![transform_one], None);
        tree.connect_etymology(derivative_two, derivative_one, vec![transform_two], None);
        tree.word_creator_phonology = test_phon_rules;
        tree.compute_lexicon().unwrap();

        let gen_parent = tree.get_by_id("parent").unwrap().word.unwrap().chars();
        let der_two = tree.get_by_id("derivative_two");
        println!("All words: {:?}", tree.to_vec());
        println!("updated string is: {}", der_two.clone().unwrap().word.unwrap().string_without_sep());
        

        let reconstructed = format!("au{}a{}{}", gen_parent[0], gen_parent[1], gen_parent[2]);
        assert_eq!(der_two.unwrap().word.unwrap().string_without_sep(), reconstructed);
    }

    #[test]
    fn test_basic_tree(){
        let mut tree = create_basic_words();

        tree.compute_lexicon().unwrap();

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

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

        tree.compute_lexicon().unwrap();
        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

        assert_eq!(out_words.contains(&"maarkwarh".to_string()), true);
       // tree.connect_etymology(lex, etymon, trans, agglutination_order)
    }

    #[test]
    fn test_lexis_overwrite() {
        let proto_word = Lexis{id: "proto_word".to_string(), word: Some("vrh".into()), language: "proto-gauntlet".to_string(), lexis_type: "stem".to_string(), ..Default::default()};
        let root = Lexis{id: "parent".to_string(), word: Some("wrh".into()), language: "gauntlet".to_string(), lexis_type: "root".to_string(), ..Default::default()};
        
        let proto_transform = Transform{name: "proto-transform".to_string(), 
        lex_match: None, 
        transforms: vec![TransformFunc::LetterReplace { letter: LetterValues{old: "w".to_string(), new: "v".to_string()}, replace: transforms::LetterPlaceType::All }]};
        
        let mut tree = create_basic_words();
        tree.connect_etymology(root, proto_word, vec![proto_transform], None);

        tree.compute_lexicon().unwrap();
        let out = tree.to_vec();
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

        println!("{}", tree.graphviz());

        assert_eq!(out_words.contains(&"vrh".to_string()), true);
        assert_eq!(out_words.contains(&"varh".to_string()), true);
        assert_eq!(out_words.contains(&"auvarh".to_string()), true);
    }

    #[test]
    fn test_idempotent(){
        let mut tree = create_basic_words();

        tree.compute_lexicon().unwrap();
        // run again
        tree.compute_lexicon().unwrap();

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

        assert_eq!(out_words.contains(&"wrh".to_string()), true);
        assert_eq!(out_words.contains(&"warh".to_string()), true);
        assert_eq!(out_words.contains(&"auwarh".to_string()), true);

    }

    #[test]
    fn test_words_without_etymology() {
        let parent = Lexis{id: "isolate_one".to_string(), word: Some("tree".into()), language: "gauntlet".to_string(), lexis_type: "word".to_string(), ..Default::default()};
        let lex_one = Lexis{id: "isolate_two".to_string(), word: Some("frost".into()), lexis_type: "word".to_string(), ..parent.clone()};
        let lex_two = Lexis{id: "isolate_three".to_string(), word: Some("rain".into()), lexis_type: "word".to_string(), ..parent.clone()};
    
        let mut tree = LanguageTree::new();
        tree.add_lexis(parent);
        tree.add_lexis(lex_one);
        tree.add_lexis(lex_two);
        tree.compute_lexicon().unwrap();

        let out = tree.to_vec();

        let out_words: Vec<String> = out.into_iter().map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

        assert_eq!(out_words.contains(&"tree".to_string()), true);
        assert_eq!(out_words.contains(&"frost".to_string()), true);
        assert_eq!(out_words.contains(&"rain".to_string()), true);
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
                TransformFunc::LetterReplace { letter: LetterValues { old: "w".to_string(), new: "k".to_string() }, replace: transforms::LetterPlaceType::All },
                TransformFunc::LetterRemove { letter: "u".to_string(), position: transforms::LetterPlaceType::All },
            ]
        }];

        tree.compute_lexicon().unwrap();

        tree.generate_daughter_language("High Gauntlet".to_string(), 
        daughter_transforms, |lex|lex.language == "gauntlet".to_string(), |lex| Lexis {tags: vec!["tested".to_string()], ..lex.clone() }).unwrap();

        let out = tree.to_vec();
        println!("got words: {:?}", out);
        let out_words: Vec<String> = out.into_iter().filter(|lex|lex.tags.len() > 0).map(|l| l.word.unwrap_or_default().string_without_sep()).collect();

        assert_eq!(out_words.contains(&"karh".to_string()), true);
        assert_eq!(out_words.contains(&"akarh".to_string()), true);
    }
}
