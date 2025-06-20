use serde::{de::Error, Deserialize, Serialize};
use std::{fs, path::PathBuf};
use directories::ProjectDirs;
use crate::errors::ClientError;

/// Representation of a single HiveCore profile
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub host: String,
    pub port_infer: u16,
    pub port_manage: u16,
    pub client_token: String,
    pub admin_token: String,
}

/// Wrapper for the profiles file
#[derive(Debug, Serialize, Deserialize)]
struct ProfilesFile {
    pub profiles: Vec<Profile>,
}

/// Returns the path to the profiles.toml file, creating directories if needed
fn profiles_path() -> Result<PathBuf, ClientError> {
    let proj = ProjectDirs::from("si", "famnit", "hivecore-tui")
        .ok_or_else(|| ClientError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unable to determine config directory",
        )))?;
    let dir = proj.config_dir();
    fs::create_dir_all(dir)?;
    Ok(dir.join("profiles.toml"))
}

/// Load all profiles from disk
pub fn load_profiles() -> Result<Vec<Profile>, ClientError> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = fs::read_to_string(path)?;
    let file: ProfilesFile = toml::from_str(&contents)?;
    Ok(file.profiles)
}

/// Save all profiles to disk
pub fn save_profiles(profiles: &[Profile]) -> Result<(), ClientError> {
    let path = profiles_path()?;
    let file = ProfilesFile { profiles: profiles.to_vec() };
    let toml = toml::to_string_pretty(&file)
        .map_err(toml::de::Error::custom)?;
    fs::write(path, toml)?;
    Ok(())
}
