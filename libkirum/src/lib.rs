
pub mod word;
pub mod transforms;
pub mod errors;
pub mod kirum;
pub mod matching;
pub mod lemma;
pub mod lexcreate;

// #[cfg(test)]
// mod tests {
//     use crate::kirum;
//     use crate::errors;

//     #[test]
//     fn test_init() -> Result<(), errors::LangError> {
//         let graph = kirum::LanguageTree::new_from_files("src/example_files/example_transforms.json".to_string(), "src/example_files/example_graph.json".to_string(), None)?;
//         let dict = graph.reduce_to_dict(|w| w.archaic == false);

//         for word in dict{
//             println!("{:?}", word);
//         }

//         Ok(())
//     }
// }
