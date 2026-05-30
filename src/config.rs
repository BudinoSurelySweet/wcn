use crate::{args::ClapArgs, profile};
use clap::{CommandFactory, FromArgMatches};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ProfileType {
    Executable,
    StaticLib,
    SharedLib,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    pub name: String,
    pub version: Option<String>,
    #[serde(default = "default_config_file_path")]
    pub config_file: PathBuf,
    #[serde(default = "default_build_directory")]
    pub build_dir: PathBuf,
    pub default_profile: Option<String>,
}

// TODO: Remove `build_dir` and put it inside `ProjectSettings`. Every profile will have it's own
// folder inside the build directory.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileSettings {
    #[serde(default = "default_profile_type")]
    #[serde(rename = "type")]
    pub profile_type: ProfileType,
    pub compiler: Option<String>,
    #[serde(default = "default_source_directory")]
    pub source_dir: PathBuf,
    #[serde(default = "default_include_directory")]
    pub include_dir: PathBuf,
    pub lib_dir: Option<PathBuf>,
    pub flags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub project: ProjectSettings,
    pub profiles: HashMap<String, ProfileSettings>,
}

pub fn default_build_directory() -> PathBuf {
    PathBuf::from("build")
}

pub fn default_config_file_path() -> PathBuf {
    PathBuf::from("build.toml")
}

pub fn default_profile_type() -> ProfileType {
    ProfileType::Executable
}

pub fn default_source_directory() -> PathBuf {
    PathBuf::from("src")
}

pub fn default_include_directory() -> PathBuf {
    PathBuf::from("include")
}

// TODO: Try to make this function a `const fn` to optimize performance
fn generate_default_config(project_name: &str) -> Settings {
    let dev_profile = "dev".to_string();
    let release_profile = "release".to_string();

    let project = ProjectSettings {
        name: project_name.to_string(),
        version: Some("0.0.0".to_string()),
        config_file: default_config_file_path(),
        build_dir: default_build_directory(),
        default_profile: Some(dev_profile.clone()),
    };

    let mut profiles = HashMap::new();

    profiles.insert(
        release_profile,
        ProfileSettings {
            profile_type: ProfileType::Executable,
            compiler: Some("gcc".to_string()),
            source_dir: PathBuf::from("src"),
            include_dir: PathBuf::from("include"),
            lib_dir: Some(PathBuf::from("lib")),
            flags: Some(vec![
                "-O3".to_string(),      // Maximum optimization for performance
                "-DNDEBUG".to_string(), // Disable standard C assert() macros
            ]),
        },
    );

    profiles.insert(
        dev_profile,
        ProfileSettings {
            profile_type: default_profile_type(),
            compiler: Some("gcc".to_string()),
            source_dir: default_source_directory(),
            include_dir: default_include_directory(),
            lib_dir: Some(PathBuf::from("lib")),
            flags: Some(vec![
                "-g".to_string(),      // Include debug symbols for gdb/lldb
                "-O0".to_string(),     // Disable optimizations for easier debugging
                "-Wall".to_string(),   // Enable all major warnings
                "-Wextra".to_string(), // Enable extra important warnings
            ]),
        },
    );

    Settings { project, profiles }
}

// Write configuration (`settings` or default) to the config file
pub fn setup_config_file(project_name: &str, settings: Option<Settings>) -> Result<(), String> {
    let config = match settings {
        Some(settings) => settings,
        None => generate_default_config(project_name),
    };

    let config = toml::to_string_pretty(&config)
        .map_err(|e| format!("Error during serialization of the config file: {}", e))?;

    let file = default_config_file_path();

    fs::write(format!("{}/{}", project_name, file.display()), config)
        .map_err(|e| format!("Cannot write in file: {}", e))?;

    Ok(())
}

pub fn get_profile(profile_name: Option<&String>) -> Result<ProfileSettings, String> {
    let config = get_config_content()?;
    let default_profile_name = config.project.default_profile;

    let name = if let Some(name) = profile_name {
        name
    } else if let Some(name) = &default_profile_name {
        name
    } else {
        return Err("No profile specified. Please provide a \
            profile as a command-line argument or set a \
            default_profile in the configuration file."
            .into());
    };

    if let Some(settings) = config.profiles.get(name) {
        profile!(name);

        Ok(settings.clone())
    } else {
        Err(format!("There is no profile with name \"{}\"", name))
    }
}

pub fn get_all_profiles(config: &Settings) -> Vec<String> {
    let mut available_profiles: Vec<String> = Vec::new();

    // Fetch all profiles inside the configuration
    for p in &config.profiles {
        available_profiles.push(p.0.clone());
    }

    available_profiles
}

pub fn get_config_content() -> Result<Settings, String> {
    let path = default_config_file_path();
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read the configuration file: {}", e))?;

    let config: Settings = toml::from_str(&content)
        .map_err(|e| format!("Syntax error inside the .toml file: {}", e))?;

    Ok(config)
}

pub fn update_clap_with_config_content(config: Settings) -> Result<ClapArgs, String> {
    let mut cmd = ClapArgs::command();
    let available_profiles = get_all_profiles(&config);

    // Prettify the vectory into a list
    let available_profiles = if !available_profiles.is_empty() {
        // Add `(default)` to the default profile
        let available_profiles: Vec<String> = available_profiles
            .iter()
            .map(|profile| {
                if let Some(default_profile) = &config.project.default_profile
                    && *profile == *default_profile
                {
                    format!("{} (default)", profile)
                } else {
                    profile.to_string()
                }
            })
            .collect();

        format!(
            "Profiles found in the configuration:\n - {}",
            available_profiles.join("\n - ")
        )
    } else {
        "No profiles available inside configuration".to_string()
    };

    // Make the string a static reference (&'static str) to make it available to the entire life
    // cycle of the program. Clap needs this.
    let available_profiles: &'static str = available_profiles.leak();

    // Put the list at the end of the build help menu.
    cmd = cmd.mut_subcommand("build", |subcmd| subcmd.after_help(available_profiles));

    // Put the list at the end of the run help menu.
    cmd = cmd.mut_subcommand("run", |subcmd| subcmd.after_help(available_profiles));

    // Restart the parsing of arguments
    let matches = cmd.get_matches();

    // Convert the "matches" into the struct
    match ClapArgs::from_arg_matches(&matches) {
        Ok(value) => Ok(value),
        Err(e) => Err(e.to_string()),
    }
}
