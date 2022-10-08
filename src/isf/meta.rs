use std::{fmt::{Display, Formatter}, fs::{read_dir, read_to_string}, path::{Path, PathBuf}};


use isf::{Isf};
use strum::Display;

pub fn default_isf_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders")
}

pub fn parse_isf_shaders(path: impl AsRef<Path>) -> impl Iterator<Item = IsfInfo> {    
    read_dir(path)
        .unwrap()
        .into_iter()
        .filter_map(|file| {
            let path  = file.unwrap().path();

            match IsfInfo::new_from_path(&path) {
                Ok(isf) => {
                    Some(isf)
                },
                Err(err) => {
                    if matches!(err, IsfReadError::ParseError(_)) {
                        eprintln!("Error parsing isf_meta file ({path:?}): {err}");
                    }
                    None
                },
            }
        })
}



#[derive(Display, Debug)]
pub enum IsfReadError {
    IoError(std::io::Error),
    InvalidExt,
    ParseError(isf::ParseError),
}

impl From<std::io::Error> for IsfReadError {
    fn from(err: std::io::Error) -> Self {
        IsfReadError::IoError(err)
    }
}

impl From<isf::ParseError> for IsfReadError {
    fn from(err: isf::ParseError) -> Self {
        IsfReadError::ParseError(err)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsfInfo{
    pub name: String,
    pub path: PathBuf,
    pub def: Isf,
}

impl AsRef<Path> for IsfInfo {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}

impl IsfInfo {
    pub fn new_from_path(path: &Path) -> Result<Self, IsfReadError> {
        let ext = path.extension()
            .map(|ext| ext.to_str())
            .flatten()
            .ok_or(IsfReadError::InvalidExt)?;
    
        if ext == "fs" {
            let content = read_to_string(&path)?;
            let isf = isf::parse(&content)?;

            Ok(
                Self {
                    name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                    path: path.to_owned(),
                    def: isf,
                }
            )
        } else {
            Err(IsfReadError::InvalidExt)
        }
    }
}

impl Display for IsfInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}