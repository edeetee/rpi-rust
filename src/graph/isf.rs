use std::{env::current_dir, path::{Path, PathBuf}, fs::{read_dir, read_to_string}, ffi::OsStr, convert::{TryFrom, TryInto}, fmt::{Display, Formatter}};

use isf::{Isf, Input, InputType};

use super::{connection_types::NodeInputDef, def::{NodeConnectionTypes, NodeValueTypes}};

pub fn parse_isf_shaders() -> impl Iterator<Item = (IsfFile, Isf)> {
    // let files = current_dir()?;
    let shaders_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    
    read_dir(shaders_dir)
        .unwrap()
        .into_iter()
        .filter_map(|file| {
            let path  = file.unwrap().path();
            let ext = path.extension()?.to_str()?;

            if ext == "fs" {
                let content = read_to_string(&path).unwrap();
                let isf = isf::parse(&content);
                return isf.ok().map(|isf| (path.into(), isf))
            }

            None
        })
}

#[derive(Clone, PartialEq)]
pub struct IsfFile{
    pub name: String,
    pub path: PathBuf
}

impl AsRef<Path> for IsfFile {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl From<PathBuf> for IsfFile {
    fn from(path: PathBuf) -> Self {
        Self {
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            path,
        }
    }
}

impl Display for IsfFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}

impl From<&InputType> for NodeConnectionTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Image => NodeConnectionTypes::Texture2D,
            // InputType::Float(_) => Ok(NodeConnectionTypes::None),
            InputType::Point2d(_) => NodeConnectionTypes::Texture2D,
            _ => NodeConnectionTypes::None
        }
    }
}

impl From<&InputType> for NodeValueTypes {
    fn from(ty: &InputType) -> Self {
        match ty {
            InputType::Float(v) => v.default.unwrap_or_default().into(),
            InputType::Color(v) => {
                let mut slice: [f32; 4] = Default::default();
                if let Some(default) = &v.default{
                    for (from, to) in default.iter().zip(&mut slice){
                        *to = *from;
                    }
                }
                slice.into()
            },
            InputType::Point2d(v) => v.default.unwrap_or_default().into(),
            InputType::Bool(v) => v.default.unwrap_or_default().into(),
            _ => NodeValueTypes::None
        }
    }
}

impl TryFrom<&Input> for NodeInputDef {
    type Error = ();

    fn try_from(input: &Input) -> Result<Self, Self::Error> {
        let ty = (&input.ty).into();
        let value = (&input.ty).into();
        
        Ok(Self {
            name: input.name.clone(),
            ty,
            value,
        })
    }
}