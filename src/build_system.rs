use crate::{
    args::{Action, ClapArgs},
    arguments, config, pending, running, success,
};
use clap::{Parser, ValueEnum};
use std::{fs, path::PathBuf, process::Command};
use walkdir::WalkDir;

#[derive(ValueEnum, Clone, Debug)]
pub enum VersionControlSystem {
    None,
    Git,
}

fn new_project(name: &str, _vcs: VersionControlSystem) -> Result<(), String> {
    // All files and directories will go under a root folder named after `name` parameter
    let directories = vec![
        PathBuf::from("src"),
        PathBuf::from("include"),
        PathBuf::from("lib"),
        PathBuf::from("build"),
    ];
    let files = vec![
        PathBuf::from("src").join("main.c"),
        config::default_config_file_path(),
        PathBuf::from("README.md"),
    ];

    // FIXME: Check if there is another project with the same name as the parametere `name`

    pending!("Creating defaults directories and files");

    for dir in directories {
        let dir_path = PathBuf::from(name).join(dir);

        if let Err(error) = fs::create_dir_all(dir_path) {
            return Err(error.to_string());
        }
    }

    for file in files {
        let file_path = PathBuf::from(name).join(file);

        if let Err(error) = fs::File::create(file_path) {
            return Err(error.to_string());
        }
    }

    match config::setup_config_file(name, None) {
        Ok(_) => (),
        Err(error) => return Err(error.to_string()),
    }

    success!("All directories and files are ready");

    // TODO: Start a `vcs` repo if vcs != None

    Ok(())
}

fn build_project(profile_name: Option<&String>) -> Result<PathBuf, String> {
    let config = config::get_config_content()?;
    let profile = config::get_profile(profile_name)?;
    let mut command;

    match profile.compiler {
        Some(compiler) => {
            command = Command::new(compiler.clone());
        }
        None => todo!("Auto-detect the compiler to use"),
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

    let output_file_path = config.project.build_dir.join(config.project.name);

    command.arg("-o");
    command.arg(output_file_path.clone());

    // TEST: Errors not handled
    let _status = command.status();

    // TODO: If the command gives an error, bring error number and put it inside `Err(_)`

    Ok(output_file_path)
}

fn run_executable(path: PathBuf, args: Vec<String>) {
    let mut command = Command::new(path);

    if !args.is_empty() {
        command.args(args);
    }

    // TEST: Errors not handled
    let _status = command.status();
}

pub fn entry_point() -> Result<(), String> {
    // Get default arguments with or without configuration file addition
    let args = if let Ok(config) = config::get_config_content() {
        config::update_clap_with_config_content(config)?
    } else {
        ClapArgs::parse()
    };

    match args.action {
        Action::Build(args) => {
            match build_project(args.profile.as_ref()) {
                Ok(_) => success!("Project built"),
                Err(e) => return Err(e),
            };
        }
        Action::Init => todo!(
            "Make an interactive experience to start \
            the build system in an already started project"
        ),
        Action::Run(args) => {
            match build_project(args.profile.as_ref()) {
                Ok(path) => {
                    success!("Project built");

                    if !args.args.is_empty() {
                        arguments!("{:?}", args.args);
                    }

                    running!(path.display());

                    run_executable(path, args.args);
                }
                Err(e) => return Err(e),
            };
        }
        Action::Clean => todo!(),
        Action::New(args) => {
            match new_project(&args.name, args.vcs) {
                Ok(_) => success!("Project created"),
                Err(e) => return Err(e),
            };
        }
    }

    Ok(())
}
