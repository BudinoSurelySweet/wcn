use crate::pending;
use crate::success;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

const CONFIG_FILE_PATH: &str = "build.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
enum ProfileType {
    Executable,
    StaticLib,
    SharedLib,
}

// TODO: Make some variables not `Option<_>`: name.
#[derive(Serialize, Deserialize, Debug)]
struct ProjectSettings {
    pub name: Option<String>,
    pub version: Option<String>,
    pub compiler: Option<String>,
    pub default_profile: Option<String>,
}

// TODO: Remove `build_dir` and put it inside `ProjectSettings`. Every profile will have it's own
// folder inside the build directory.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProfileSettings {
    #[serde(rename = "type")]
    pub profile_type: ProfileType,
    pub source_dir: PathBuf,
    pub include_dir: PathBuf,
    pub lib_dir: Option<PathBuf>,
    pub build_dir: Option<PathBuf>,
    pub flags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    pub project: Option<ProjectSettings>,
    pub profiles: HashMap<String, ProfileSettings>,
}

// TODO: Try to make this function a `const fn` to optimize performance
fn generate_default_config(project_name: &str) -> Settings {
    let project = Some(ProjectSettings {
        name: Some(project_name.to_string()),
        version: Some("0.1.0".to_string()),
        compiler: Some("gcc".to_string()),
        default_profile: Some("debug".into()),
    });

    let mut profiles = HashMap::new();

    profiles.insert(
        "debug".into(),
        ProfileSettings {
            profile_type: ProfileType::Executable,
            source_dir: PathBuf::from("src"),
            include_dir: PathBuf::from("include"),
            lib_dir: Some(PathBuf::from("lib")),
            build_dir: Some(PathBuf::from("build")),
            flags: Some(vec![
                "-g".to_string(),      // Include debug symbols for gdb/lldb
                "-O0".to_string(),     // Disable optimizations for easier debugging
                "-Wall".to_string(),   // Enable all major warnings
                "-Wextra".to_string(), // Enable extra important warnings
            ]),
        },
    );

    profiles.insert(
        "release".into(),
        ProfileSettings {
            profile_type: ProfileType::Executable,
            source_dir: PathBuf::from("src"),
            include_dir: PathBuf::from("include"),
            lib_dir: Some(PathBuf::from("lib")),
            build_dir: Some(PathBuf::from("build")),
            flags: Some(vec![
                "-O3".to_string(),      // Maximum optimization for performance
                "-DNDEBUG".to_string(), // Disable standard C assert() macros
            ]),
        },
    );

    Settings { project, profiles }
}

// Write configuration (`settings` or default) to the config file
fn setup_config_file(project_name: &str, settings: Option<Settings>) -> Result<(), String> {
    let config = match settings {
        Some(settings) => settings,
        None => generate_default_config(project_name),
    };

    let config = toml::to_string_pretty(&config)
        .map_err(|e| format!("Error during serialization of the config file: {}", e))?;

    fs::write(format!("{}/{}", project_name, CONFIG_FILE_PATH), config)
        .map_err(|e| format!("Cannot write in file: {}", e))?;

    Ok(())
}

fn get_profile(profile_name: Option<&str>) -> Result<ProfileSettings, String> {
    let config = get_config(CONFIG_FILE_PATH)?;
    let default_profile_name = config.project.and_then(|project| project.default_profile);
    let profile;

    if let Some(p) = profile_name.and_then(|name| config.profiles.get(name)) {
        profile = p.clone();
    } else if let Some(p) = default_profile_name.and_then(|def| config.profiles.get(&def)) {
        profile = p.clone();
    } else {
        return Err(
            "No profile specified. Please provide a profile as a command-line argument or set a default_profile in the configuration file.".into(),
        );
    };

    Ok(profile)
}

fn get_config<P: AsRef<Path>>(path: P) -> Result<Settings, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Cannot read the configuration file: {}", e))?;

    let config: Settings = toml::from_str(&content)
        .map_err(|e| format!("Syntax error inside the .toml file: {}", e))?;

    Ok(config)
}

fn new_project(name: &str) -> Result<(), String> {
    // All files and directories will go under a root folder named after `name` parameter
    let directories = vec!["src", "include", "lib", "build"];
    let files = vec!["src/main.c", CONFIG_FILE_PATH, "README.md"];

    // FIXME: Check if there is another project with the same name as the parametere `name`

    pending!("Creating defaults directories and files");

    for dir in directories {
        let res = fs::create_dir_all(format!("{}/{}", name, dir));

        if let Err(error) = res {
            return Err(error.to_string());
        }
    }

    for file in files {
        let res = fs::File::create(format!("{}/{}", name, file));

        if let Err(error) = res {
            return Err(error.to_string());
        }
    }

    match setup_config_file(name, None) {
        Ok(_) => (),
        Err(error) => return Err(error.to_string()),
    }

    success!("All directories and files are ready");

    // TODO: Start a git repo by default. With the flag `--nogit` the build system will not create the repo.

    Ok(())
}

fn build_project(profile_name: Option<&str>) -> Result<PathBuf, String> {
    let config = get_config(CONFIG_FILE_PATH)?;
    let profile = get_profile(profile_name)?;
    let mut command;

    if let Some(project) = &config.project {
        match &project.compiler {
            Some(compiler) => {
                command = Command::new(compiler.clone());
            }
            None => todo!("Auto-detect the compiler to use"),
        }
    } else {
        return Err("There's no project header in build.toml".to_string());
    }

    // Fetch all `.c` files
    for entry in WalkDir::new(profile.source_dir) {
        let entry = match entry {
            Ok(result) => result,
            Err(error) => return Err(error.to_string()),
        };

        let Some(entry_name) = entry.file_name().to_str() else {
            continue;
        };

        if !entry_name.ends_with(".c") {
            continue;
        }

        command.arg(entry.path());
    }

    command.arg("-I").arg(profile.include_dir);

    if let Some(lib_dir) = profile.lib_dir {
        command.arg("-L").arg(&lib_dir);
    }

    let mut output_file_path = String::new();

    if let Some(build_dir) = profile.build_dir
        && let Some(project) = config.project
        && let Some(name) = project.name
    {
        output_file_path = format!("{}/{}", build_dir.display(), name);
        output_file_path.push_str(build_dir.to_str().unwrap());

        command.arg("-o");
        command.arg(output_file_path.clone());
    }

    // TEST: Errors not handled
    let _status = command.status();

    Ok(PathBuf::from(output_file_path.clone()))
}

fn run_executable(path: PathBuf, args: Option<Vec<String>>) {
    let mut command = Command::new(path);

    if let Some(args) = args {
        command.args(args);
    }

    // TEST: Errors not handled
    let _status = command.status();
}

pub fn entry_point() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    let action = args[1].clone();

    match action.as_str() {
        "new" => {
            if args.len() != 3 {
                return Err(format!(
                    "The number of args ({}) is not equal to 3. Consider adding or removing some arguments",
                    args.len()
                ));
            }

            let name = args[2].clone();

            match new_project(&name) {
                Ok(_) => success!("Project created"),
                Err(e) => return Err(e),
            };
        }
        "build" => {
            let profile_name = if args.len() > 2 { &args[2] } else { "" };

            match build_project(Some(profile_name)) {
                Ok(_) => success!("Project built"),
                Err(e) => return Err(e),
            }
        }
        "run" => {
            let profile_name = if args.len() > 2 { &args[2] } else { "" };
            let mut program_args = None;

            // WARNING: Remove this and put vec.find("-a") instead
            if args.len() > 3 {
                let start = if args[2] == "-a" || args[2] == "--args" {
                    3
                } else if args.len() > 4 && (args[3] == "-a" || args[3] == "--args") {
                    4
                } else {
                    return Err("No `-a` found at the expected index".to_string());
                };

                program_args = Some(args.clone());
                program_args = program_args.map(|mut args| args.split_off(start));
            }

            match build_project(Some(profile_name)) {
                Ok(path) => {
                    success!("Project built");
                    run_executable(path, program_args);
                }
                Err(e) => return Err(e),
            }
        }
        "clean" => {
            todo!("Clear the build directory")
        }
        "help" => {
            todo!("Help menu")
        }
        _ => {
            todo!("Call help menu on empty args")
        }
    }

    Ok(())
}
