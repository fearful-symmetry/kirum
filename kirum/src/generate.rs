use std::{fs::File, io::Write, collections::HashMap, path::PathBuf};
use anyhow::{Result, Context, anyhow};
use libkirum::{transforms::Transform, kirum::Lexis, word::Etymology};
use crate::{files::read_and_compute, entries, cli::SeparateValues};

pub fn daughter(graph: Option<String>, 
    transforms: Option<String>, 
    daughter_ety: String, 
    ancestor: String, 
    lang_name: String, 
    directory: Option<String>, 
    output: String, 
    by_field: Option<SeparateValues>) -> Result<String> {
        // setup, read files, etc
        let mut computed = read_and_compute(transforms, graph, directory)
        .context("error reading existing graph and transforms")?;

        let trans_raw = std::fs::read_to_string(daughter_ety.clone())
        .context(format!("error reading daughter transformation file {}", daughter_ety))?;

        let daughter_transform_map: entries::TransformGraph = serde_json::from_str(&trans_raw)
        .context("error parsing daughter transformations")?;

        let processed_transforms: Vec<Transform> = daughter_transform_map.transforms.into_iter()
        .map(|(n, t)| Transform{name: n, ..t.into()}).collect();

        // actually start creating language
        debug!("Creating daughter language '{}' from '{}'", lang_name, ancestor);
        computed.generate_daughter_language(lang_name.clone(), 
        processed_transforms, |l| l.language == ancestor, 
        |l| Lexis {tags: [l.tags.clone(), ["autogenerated".to_string()].to_vec()].concat(), ..l.clone()});

        let rendered_dict = computed.to_vec_etymons(|word|word.language == lang_name);

        debug!("grouping output files by: {:?}", by_field);
        // write files
        let file_map = group_by(by_field, rendered_dict, output.clone());

        if by_field.is_some() {
            debug!("creating root directory at {}", &output);
            std::fs::create_dir_all(&output)?;
        } else {
            // in cases where there's no grouping, make sure we have the expected file
            let out_path: PathBuf = output.clone().into();
            if !out_path.is_file() {
                return Err(anyhow!("File {} does not have an extension. Did you mean to set group_by?", out_path.display()))
            }
        }

        for (fname, data) in file_map {
            let graph = entries::create_json_graph(data);

            let graph_data = serde_json::to_string_pretty(&graph)
            .context("error creating JSON from graph")?;

            let mut out_path: PathBuf = output.clone().into();
            // if the group_by field exists, treat output as the prefix path
            // then create the prefix
            if by_field.is_some(){
                out_path.push(fname);
                out_path.set_extension("json");
            }

            debug!("Creating daughter language file {}", &out_path.display());
            let mut file = File::create(&out_path)
            .context(format!("error creating file {}", out_path.display()))?;

            write!(file, "{}", graph_data)?;

        }

        Ok(String::new())
}

fn group_by(field: Option<SeparateValues>, dict: Vec<(Lexis, Etymology)>, default: String) -> HashMap<String, Vec<(Lexis, Etymology)>> {
    let mut files: HashMap<String, Vec<(Lexis, Etymology)>> = HashMap::new();

    if let Some(to_group) = field {
        match to_group {
            // if grouped by word, assume every dict entry will get its own value
            SeparateValues::Word => {
                for entry in dict {
                    files.insert(entry.0.id.clone(), vec![entry]);
                }
            },
            SeparateValues::Archaic => {
                for entry in dict {
                    let key = if entry.0.archaic {
                        "archaic"
                    } else {
                        "modern"
                    };
                    if let Some(lst) = files.get_mut(key) {
                        lst.push(entry);
                    } else {
                        files.insert(key.to_string(), vec![entry]);
                    }
                }
            },
            SeparateValues::LexisType => {
                for entry in dict {
                    if let Some(lst) = files.get_mut(&entry.0.lexis_type) {
                        lst.push(entry);
                    } else {
                        files.insert(entry.0.lexis_type.clone(), vec![entry]);
                    }
                }
            }
        }
    } else {
        files.insert(default, dict);
    }

    files
}
