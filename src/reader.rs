use std::collections::HashMap;
use petgraph::Graph;
use serde::{Deserialize, Serialize};
use crate::transforms::TransformFunc;
use crate::errors::LangError;
use crate::word::{ Word, PartOfSpeech, Etymology};
use crate::kirum::{Transform, Lexis};


#[derive(Serialize, Deserialize, Clone)]
pub struct RawLexicalEntry {
    pub word: Option<Word>,
    pub word_type: Option<String>,
    pub language: Option<String>,
    #[serde(default)]
    pub definition:String,
    pub part_of_speech: Option<PartOfSpeech>,
    pub etymology: Option<Etymology>,
    #[serde(default = "default_archaic")]
    pub archaic: bool,
    pub tags: Option<Vec<String>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WordGraph {
    pub words: HashMap<String, RawLexicalEntry>,
}

fn default_archaic() ->bool{
    false
}

impl std::fmt::Debug for RawLexicalEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rendered_args: String = match &self.word{
            Some(word_vec) =>{
                word_vec.to_string()
            },
            None => String::from("None")
        };
        if let Some(lang) = &self.language{
            rendered_args = format!("{} ({})", rendered_args, lang);
        }
        if let Some(pos) = &self.part_of_speech{
            rendered_args = format!("{}: ({:?})", rendered_args, pos);
        }
        
        rendered_args = format!("{} {}", rendered_args, self.definition);
        
        f.write_str(&rendered_args)
    }
}

// take the raw input values we got, render then as a read graph
pub fn render_graph(raw_graph: WordGraph, known_transforms: HashMap<String, Vec<TransformFunc>>) -> Result<Graph<Lexis, (Transform, Option<Word>)>, LangError>{
    

    let mut word_graph = Graph::<Lexis, (Transform, Option<Word>)>::new();
    let mut added: HashMap<String, petgraph::graph::NodeIndex> = HashMap::new();

    for (node_name, node) in &raw_graph.words{ 
        let word_idx = word_graph.add_node(Lexis{id: node_name.to_string(), ..node.clone().into()});
        added.insert(node_name.to_owned(), word_idx); 
    }

    for (node_name, node) in raw_graph.words{
        let etymons = match node.etymology {
            Some(ety) => ety,
            None => continue,
        };
        for etymon in etymons.etymons{
            let written_transform = match known_transforms.get(&etymon.transform){
                Some(trans) => trans.clone(),
                None => {
                    return Err(LangError::ValidationError(format!("could not find transform with ID {}", etymon.transform)));
                } 
            };
            let etymon_idx = match added.get(&etymon.etymon){
                Some(ety) => ety.clone(),
                None => {
                    return Err(LangError::ValidationError(format!("could not find etymon with ID {}", etymon.etymon)));
                }
            };
            let derivative_idx = match added.get(&node_name){
                Some(der) => der.clone(),
                None => {
                    return Err(LangError::ValidationError(format!("could not find derivative with ID {}", node_name)));
                }
            };
            let trans = (Transform{transforms: written_transform, agglutination_order: etymon.agglutination_order, name: etymon.transform}, None);
            
            word_graph.add_edge(etymon_idx, derivative_idx, trans);
        }

    }

    Ok(word_graph)
}

pub fn get_transforms(filepath: String) -> Result<HashMap<String, Vec<TransformFunc>>, LangError>{
    let trans_raw = std::fs::read_to_string(filepath).map_err(LangError::JSONImportError)?;
    let transforms: HashMap<String, Vec<TransformFunc>> = serde_json::from_str(&trans_raw).map_err(LangError::JSONSerdeError)?;

    Ok(transforms)
}