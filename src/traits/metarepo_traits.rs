use crate::models::structs::RepositoryItem;
use crate::services::n::*;
use crate::services::yarn::*;
use crate::traits::string_traits::StringExtension;
use semver::Version;
use std::collections::HashMap;
use std::fs;
use std::os::unix;
use std::path::PathBuf;
use std::process::Command;

pub trait MetarepoExtension {
    fn get_projects(self) -> Vec<String>;
    fn set_node_version(self);
    fn map_repository(self) -> HashMap<String, RepositoryItem>;
    fn package_to_link(self, project: &String);
    fn link_to_package(self, project: &String);
    fn copy_packages(self, project: &String);
    fn update_dependencies(self, project: &String);
    fn build_tree(self, project: &String);
    fn clean_tree(self, project: &String);
}

impl MetarepoExtension for &PathBuf {
    fn get_projects(self) -> Vec<String> {
        let mut root_path = PathBuf::from(self);
        println!("Getting projects from .meta file...");
        root_path.push(".meta");
        let meta_content =
            std::fs::read_to_string(root_path).expect("Can't find .meta file in given path!");
        let json: serde_json::Value =
            serde_json::from_str(&meta_content).expect("File .meta was not well-formatted!");
        let projects = json["projects"]
            .as_object()
            .expect("Projects are not in valid format!");
        return projects.keys().map(|k| k.to_string()).collect();
    }

    fn set_node_version(self) {
        let mut root_path = PathBuf::from(self);
        println!("Getting .node-version file...");
        root_path.push(".node-version");
        let node_version = std::fs::read_to_string(root_path)
            .expect("Can't find .node-version file in given path!");
        set_node_version(&node_version);
    }

    fn map_repository(self) -> HashMap<String, RepositoryItem> {
        let projects = self.get_projects();
        let mut repository = HashMap::<String, RepositoryItem>::new();

        // map repository
        let filter = |_: &&String| true;
        let map_modules = |project_relative_path: &String| {
            let mut project_path = PathBuf::from(self);
            project_path.push(project_relative_path.to_string());
            let pkg = get_package_json(&project_path);
            let ri = RepositoryItem::new(
                project_relative_path.strip(),
                pkg["name"].strip(),
                Vec::<String>::new(),
            );
            repository.insert(project_relative_path.strip(), ri);
        };
        iterate_projects(&projects, filter, map_modules);

        // map dependencies
        let repo_modules: Vec<String> = repository.values().map(|k| k.module.to_string()).collect();
        let filter = |_: &&String| true;
        let map_dependencies = |project_relative_path: &String| {
            let mut project_path = PathBuf::from(self);
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
        iterate_projects(&projects, filter, map_dependencies);

        return repository;
    }

    fn package_to_link(self, project: &String) {
        let mapped_repository = self.map_repository();
        let link = |dep_path: &PathBuf, dep_repo_path: &PathBuf| {
            println!(
                "=> Linking {} to {}",
                dep_path.display().to_string(),
                dep_repo_path.display().to_string()
            );
            fs::rename(&dep_path, dep_path.display().to_string() + "_orig")
                .expect("error renaming directory");
            unix::fs::symlink(&dep_repo_path, &dep_path).expect("error symlinking directory");
        };
        recurse_projects(self, &project, &mapped_repository, &link);
    }

    fn link_to_package(self, project: &String) {
        let mapped_repository = self.map_repository();
        let unlink = |dep_path: &PathBuf, _: &PathBuf| {
            println!(
                "=> Unlinking {} and restoring {}",
                dep_path.display().to_string(),
                dep_path.display().to_string() + "_orig"
            );
            Command::new("rm")
                .arg("-rf")
                .arg(dep_path)
                .output()
                .expect("error deleting symlink");
            fs::rename(dep_path.display().to_string() + "_orig", &dep_path)
                .expect("error renaming directory");
        };
        recurse_projects(self, &project, &mapped_repository, &unlink);
    }

    fn copy_packages(self, project: &String) {
        let mapped_repository = self.map_repository();
        let copy = |dep_path: &PathBuf, dep_repo_path: &PathBuf| {
            println!(
                "=> Copying {} to {}",
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
        recurse_projects(self, &project, &mapped_repository, &copy);
    }

    fn update_dependencies(self, project: &String) {
        let mapped_repository = self.map_repository();
        let mut root_repo_path = PathBuf::from(self);
        root_repo_path.push(project);
        yarn_outdated_upgrade(&root_repo_path, &root_repo_path);
        recurse_projects(self, &project, &mapped_repository, &yarn_outdated_upgrade);
    }

    fn build_tree(self, project: &String) {
        let mapped_repository = self.map_repository();
        recurse_projects_no_clean(self, &project, &mapped_repository, &yarn_build_project);
    }

    fn clean_tree(self, project: &String) {
        let mapped_repository = self.map_repository();
        recurse_clean_projects(self, &project, &mapped_repository);
    }
}

const MODULES_FOLDER: &str = "node_modules";

fn recurse_projects<F>(
    root_path: &PathBuf,
    project: &String,
    mapped_repository: &HashMap<String, RepositoryItem>,
    function: &F,
) where
    F: Fn(&PathBuf, &PathBuf),
{
    let mut root_project_path = PathBuf::from(root_path);
    root_project_path.push(project);
    let mut node_modules_path = PathBuf::from(&root_project_path);
    node_modules_path.push(MODULES_FOLDER);
    Command::new("rm")
        .arg("-rf")
        .arg(&node_modules_path)
        .output()
        .expect("error removing node_modules directory");
    yarn_install(&root_project_path.display().to_string());
    mapped_repository[project]
        .dependencies
        .iter()
        .for_each(|dep| {
            let mut dep_path = PathBuf::from(&node_modules_path);
            dep_path.push(dep);
            dep_path.as_path().exists().then(|| {
                mapped_repository
                    .values()
                    .find(|ri| &ri.module == dep)
                    .map(|ri| {
                        // link nested dependencies
                        recurse_projects(root_path, &ri.project, &mapped_repository, function);
                        // link root dependencies
                        let mut dep_repo_path = PathBuf::from(root_path);
                        dep_repo_path.push(&ri.project);
                        dep_repo_path
                            .as_path()
                            .is_dir()
                            .then(|| function(&dep_path, &dep_repo_path));
                    });
            });
        });
}

fn recurse_projects_no_clean<F>(
    root_path: &PathBuf,
    project: &String,
    mapped_repository: &HashMap<String, RepositoryItem>,
    function: &F,
) where
    F: Fn(&PathBuf, &PathBuf),
{
    let mut root_project_path = PathBuf::from(root_path);
    root_project_path.push(project);
    let mut node_modules_path = PathBuf::from(&root_project_path);
    node_modules_path.push(MODULES_FOLDER);
    mapped_repository[project]
        .dependencies
        .iter()
        .for_each(|dep| {
            let mut dep_path = PathBuf::from(&node_modules_path);
            dep_path.push(dep);
            dep_path.as_path().exists().then(|| {
                mapped_repository
                    .values()
                    .find(|ri| &ri.module == dep)
                    .map(|ri| {
                        // link nested dependencies
                        recurse_projects_no_clean(
                            root_path,
                            &ri.project,
                            &mapped_repository,
                            function,
                        );
                        // link root dependencies
                        let mut dep_repo_path = PathBuf::from(root_path);
                        dep_repo_path.push(&ri.project);
                        dep_repo_path
                            .as_path()
                            .is_dir()
                            .then(|| function(&dep_path, &dep_repo_path));
                    });
            });
        });
}

fn recurse_clean_projects(
    root_path: &PathBuf,
    project: &String,
    mapped_repository: &HashMap<String, RepositoryItem>,
) {
    let mut root_project_path = PathBuf::from(root_path);
    root_project_path.push(project);
    let mut node_modules_path = PathBuf::from(&root_project_path);
    node_modules_path.push(MODULES_FOLDER);
    Command::new("rm")
        .arg("-rf")
        .arg(&node_modules_path)
        .output()
        .expect("error removing node_modules directory");
    mapped_repository[project]
        .dependencies
        .iter()
        .for_each(|dep| {
            mapped_repository
                .values()
                .find(|ri| &ri.module == dep)
                .map(|ri| {
                    // clean nested dependencies
                    recurse_clean_projects(root_path, &ri.project, &mapped_repository);
                });
        });
}

fn iterate_projects<I, P, F>(projects: &Vec<I>, predicate: P, f: F)
where
    P: FnMut(&&I) -> bool,
    F: FnMut(&I),
{
    return projects.iter().filter(predicate).for_each(f);
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

fn yarn_build_project(_: &PathBuf, dep_repo_path: &PathBuf) {
    let path = dep_repo_path.display().to_string();
    let message = format!("I'm in repo {}", path);
    println!();
    println!("{}", "#".repeat(message.len()));
    println!("{}", message);
    println!("{}", "-".repeat(message.len()));
    yarn_build(&path);
}

fn yarn_outdated_upgrade(_: &PathBuf, dep_repo_path: &PathBuf) {
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
        let actual = Version::parse(package[1].as_str().expect("")).unwrap();
        let wanted = Version::parse(package[2].as_str().expect("")).unwrap();
        let latest = Version::parse(package[3].as_str().expect("")).unwrap();
        println!();
        if wanted > actual {
            println!("Will upgrade {} from ^{} to ^{}", name, actual, wanted);
            yarn_upgrade(&path, &name.to_string(), &wanted.to_string());
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
