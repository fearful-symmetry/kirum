use libkirum::{transforms::{TransformFunc, GlobalTransform}, matching::LexisMatch};
use serde::{Serialize, Deserialize};
use serde_with::skip_serializing_none;


#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
/// Defines the contents of the global.json file
pub struct Global {
    /// Specifies global transforms
    pub transforms: Option<Vec<RawGlobalTransform>>
}


#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RawGlobalTransform {
    pub transforms: Vec<TransformFunc>,
    pub conditional: GlobalConditionals
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GlobalConditionals {
    pub etymon: Option<LexisMatch>,
    pub lexis: LexisMatch
}


impl From<RawGlobalTransform> for GlobalTransform {
    fn from(value: RawGlobalTransform) -> Self {
        GlobalTransform { 
            lex_match: value.conditional.lexis, 
            etymon_match: value.conditional.etymon, 
            transforms: value.transforms 
        }
    }
}