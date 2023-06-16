use std::{ffi::OsStr, path::PathBuf};

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct File {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub version: u32,
}

impl File {
    pub fn fmt_version(&self) -> String {
        format!("v{:03}", self.version)
    }

    pub fn from_path(path: PathBuf) -> Result<Self, String> {
        let extension = String::from(
            path.extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or(""),
        );
        let name = String::from(
            path.file_stem()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap_or(""),
        );
        let mut version_string = name.clone();
        let version_offset = name.len() - 5;
        let name = version_string.drain(..version_offset).collect();

        if !(&version_string.chars().nth(0).unwrap_or('0') == &'_'
            && &version_string.chars().nth(1).unwrap_or('0') == &'v')
        {
            return Err(String::from("Not a valid filename."));
        }
        version_string.remove(0);
        version_string.remove(0);
        let version: u32 = version_string.parse().unwrap_or(1);
        Ok(Self {
            name: name,
            path: path,
            version: version,
            extension: extension,
        })
    }
}
