use crate::models::structs::RepositoryItem;
use crate::traits::string_traits::*;
use std::collections::HashMap;
use std::fs;
use std::os::unix;
use std::path::PathBuf;
use std::process::{exit, Command};

const MODULES_FOLDER: &str = "node_modules";
const TAB: &str = "   ";

pub fn link(project: &String, root_path: &PathBuf) {
    let projects = get_projects(root_path);
    let mapped_repository = map_modules(root_path, projects);

    println!("\n-------------------------------------------------");
    println!("Linking modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    package_to_link(root_path, project, &mapped_repository);
}

pub fn unlink(project: &String, root_path: &PathBuf) {
    let projects = get_projects(root_path);
    let mapped_repository = map_modules(root_path, projects);

    println!("\n-------------------------------------------------");
    println!("Unlinking modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    link_to_package(root_path, project, &mapped_repository);
}

pub fn copy(project: &String, root_path: &PathBuf) {
    let projects = get_projects(root_path);
    let mapped_repository = map_modules(root_path, projects);

    println!("\n-------------------------------------------------");
    println!("Copying modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    copy_packages(root_path, project, &mapped_repository);
}

pub fn update(project: &String, root_path: &PathBuf) {
    let projects = get_projects(root_path);
    let mapped_repository = map_modules(root_path, projects);

    println!("\n-------------------------------------------------");
    println!("Updating repository to latest minor release...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    update_dependencies(root_path, project, &mapped_repository);
}

fn map_modules(path: &PathBuf, projects: Vec<String>) -> HashMap<String, RepositoryItem> {
    let mut repository = HashMap::<String, RepositoryItem>::new();

    // map repository
    let filter = |_: &&String| true;
    let map_modules = |project_relative_path: &String| {
        let mut project_path = PathBuf::from(path);
        project_path.push(project_relative_path.to_string());
        let pkg = get_package_json(&project_path);
        let ri = RepositoryItem::new(
            project_relative_path.strip(),
            pkg["name"].strip(),
            Vec::<String>::new(),
        );
        repository.insert(project_relative_path.strip(), ri);
    };
    process_projects(&projects, filter, map_modules);

    // map dependencies
    let repo_modules: Vec<String> = repository.values().map(|k| k.module.to_string()).collect();
    let filter = |_: &&String| true;
    let map_dependencies = |project_relative_path: &String| {
        let mut project_path = PathBuf::from(path);
        project_path.push(project_relative_path.to_string());
        let pkg = get_package_json(&project_path);
        let dependencies = get_cross_dependencies(&pkg, &repo_modules);
        let ri = RepositoryItem::new(
            project_relative_path.strip(),
            pkg["name"].strip(),
            dependencies,
        );
        repository.insert(project_relative_path.strip(), ri);
    };
    process_projects(&projects, filter, map_dependencies);

    return repository;
}

fn package_to_link(
    root_path: &PathBuf,
    project: &String,
    mapped_repo: &HashMap<String, RepositoryItem>,
) {
    let link = |dep_path: &PathBuf, dep_repo_path: &PathBuf, level: usize| {
        println!(
            "{}=> Linking {} to {}",
            TAB.repeat(level),
            dep_path.display().to_string(),
            dep_repo_path.display().to_string()
        );
        fs::rename(&dep_path, dep_path.display().to_string() + "_orig")
            .expect("error renaming directory");
        unix::fs::symlink(&dep_repo_path, &dep_path).expect("error symlinking directory");
    };
    recurse_packages(&root_path, &project, &mapped_repo, 0, &link);
}

fn link_to_package(
    root_path: &PathBuf,
    project: &String,
    mapped_repo: &HashMap<String, RepositoryItem>,
) {
    let unlink = |dep_path: &PathBuf, _: &PathBuf, level: usize| {
        println!(
            "{}=> Unlinking {} and restoring {}",
            TAB.repeat(level),
            dep_path.display().to_string(),
            dep_path.display().to_string() + "_orig"
        );
        fs::remove_file(&dep_path).expect("error deleting symlink");
        fs::rename(dep_path.display().to_string() + "_orig", &dep_path)
            .expect("error renaming directory");
    };
    recurse_packages(&root_path, &project, &mapped_repo, 0, &unlink);
}

fn copy_packages(
    root_path: &PathBuf,
    project: &String,
    mapped_repo: &HashMap<String, RepositoryItem>,
) {
    let copy = |dep_path: &PathBuf, dep_repo_path: &PathBuf, level: usize| {
        println!(
            "{}=> Copying {} to {}",
            TAB.repeat(level),
            dep_path.display().to_string(),
            dep_repo_path.display().to_string()
        );
        fs::rename(&dep_path, dep_path.display().to_string() + "_orig")
            .expect("error renaming directory");
        Command::new("cp")
            .arg("-r")
            .arg(dep_repo_path)
            .arg(dep_path)
            .output()
            .expect("error copying directory");
    };
    recurse_packages(&root_path, &project, &mapped_repo, 0, &copy);
}

fn update_dependencies(
    root_path: &PathBuf,
    project: &String,
    mapped_repo: &HashMap<String, RepositoryItem>,
) {
    let mut root_repo_path = PathBuf::from(&root_path);
    root_repo_path.push(project);
    yarn_outdated_upgrade(&root_repo_path, &root_repo_path, 0);
    recurse_packages(
        &root_path,
        &project,
        &mapped_repo,
        0,
        &yarn_outdated_upgrade,
    );
}

fn yarn_outdated_upgrade(_: &PathBuf, dep_repo_path: &PathBuf, level: usize) {
    let path = dep_repo_path.display().to_string();
    let message = format!("I'm in repo {}", path);
    println!();
    println!("{}", "#".repeat(message.len()));
    println!("{}", message);
    println!("{}", "-".repeat(message.len()));
    yarn_install(&path);
    let outdated = yarn_outdated(&path);
    let root_value: serde_json::Value =
        serde_json::from_str(&outdated).expect("json not formatted");
    let data_value: &serde_json::Value = &root_value.as_object().expect("")["data"];
    let body_value: &serde_json::Value = &data_value.as_object().expect("")["body"];
    let packages: &Vec<serde_json::Value> = &body_value.as_array().expect("");
    packages.iter().for_each(|p| {
        let package = p.as_array().expect("");
        let name = package[0].as_str().expect("");
        let actual = package[1].as_str().expect("");
        let wanted = package[2].as_str().expect("");
        let latest = package[3].as_str().expect("");
        println!();
        if wanted > actual {
            println!("Will upgrade {} from ^{} to ^{}", name, actual, wanted);
            yarn_add(&path, &name.to_string(), &wanted.to_string());
        } else {
            println!("Will not upgrade {}", name);
        }
        if latest > wanted {
            println!(
                "- (!) {}@^{} has been replaced by new major ^{}",
                name, wanted, latest
            );
        }
    });
}

fn yarn_install(path: &String) {
    println!("Installing dependencies...");
    let output = Command::new("yarn")
        .arg("install")
        .arg("--frozen-lockfile")
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("error with yarn");
}

fn yarn_outdated(path: &String) -> String {
    println!("Checking outdated dependencies...");
    let output = Command::new("yarn")
        .arg("outdated")
        .arg("--json")
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("error with yarn");
    let output_json: String = String::from_utf8(output.stdout).expect("error parsing output");
    return output_json
        .split('\n')
        .skip(1)
        .next()
        .expect("cannot split")
        .to_string();
}

fn yarn_add(path: &String, package: &String, version: &String) {
    println!("Adding {}@^{}...", package, version);
    let output = Command::new("yarn")
        .arg("add")
        .arg(format!("{}@^{}", package, version))
        .arg("--cwd")
        .arg(&path.to_string())
        .output()
        .expect("error with yarn");
    if output.stderr.len() > 0 {
        String::from_utf8(output.stderr).map(|s| {
            println!("{}", s);
        });
    };
    String::from_utf8(output.stdout).map(|s| {
        println!("{}", s);
    });
}

fn recurse_packages<F>(
    root_path: &PathBuf,
    project: &String,
    mapped_repo: &HashMap<String, RepositoryItem>,
    level: usize,
    function: &F,
) where
    F: Fn(&PathBuf, &PathBuf, usize),
{
    let mut root_repo_path = PathBuf::from(&root_path);
    root_repo_path.push(project);
    root_repo_path.push(MODULES_FOLDER);
    mapped_repo[project].dependencies.iter().for_each(|dep| {
        let mut dep_path = PathBuf::from(&root_repo_path);
        dep_path.push(dep);
        dep_path.as_path().exists().then(|| {
            mapped_repo.values().find(|ri| &ri.module == dep).map(|ri| {
                // link nested dependencies
                recurse_packages(&root_path, &ri.project, &mapped_repo, level + 1, function);
                // link root dependencies
                let mut dep_repo_path = PathBuf::from(&root_path);
                dep_repo_path.push(&ri.project);
                dep_repo_path
                    .as_path()
                    .is_dir()
                    .then(|| function(&dep_path, &dep_repo_path, level));
            });
        });
    });
}

fn process_projects<I, P, F>(projects: &Vec<I>, predicate: P, f: F)
where
    P: FnMut(&&I) -> bool,
    F: FnMut(&I),
{
    return projects.iter().filter(predicate).for_each(f);
}

fn get_projects(path: &PathBuf) -> Vec<String> {
    let mut root_path = PathBuf::from(path);
    println!("Getting projects from .meta file...");
    root_path.push(".meta");
    let meta_content =
        std::fs::read_to_string(root_path).expect("can't find .meta file in given path");
    let json: serde_json::Value =
        serde_json::from_str(&meta_content).expect(".meta was not well-formatted");
    let projects = json["projects"]
        .as_object()
        .expect("projects are not in valid format");
    return projects.keys().map(|k| k.to_string()).collect();
}

fn get_package_json(path: &PathBuf) -> serde_json::Value {
    let mut project_path = PathBuf::from(path);
    project_path.push("package.json");
    return std::fs::read_to_string(project_path).map_or_else(
        |_| serde_json::from_str("{}").expect("...not well formatted!"),
        |c| serde_json::from_str(&c).expect("...not well formatted!"),
    );
}

fn get_cross_dependencies(package: &serde_json::Value, modules: &Vec<String>) -> Vec<String> {
    let dependencies_section = package["dependencies"].as_object();
    let dependencies = dependencies_section.map_or_else(
        || Vec::<String>::new(),
        |d| d.keys().map(|k| k.to_string()).collect(),
    );

    return dependencies
        .iter()
        .filter(|d| modules.contains(d))
        .map(|d| d.strip())
        .collect();
}
