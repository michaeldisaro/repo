use crate::traits::metarepo_traits::MetarepoExtension;
use std::path::PathBuf;

pub fn link(project: &String, root_path: &PathBuf) {
    println!("\n-------------------------------------------------");
    println!("Linking modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    root_path.package_to_link(project);
}

pub fn unlink(project: &String, root_path: &PathBuf) {
    println!("\n-------------------------------------------------");
    println!("Unlinking modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    root_path.link_to_package(project);
}

pub fn copy(project: &String, root_path: &PathBuf) {
    println!("\n-------------------------------------------------");
    println!("Copying modules...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    root_path.copy_packages(project);
}

pub fn update(project: &String, root_path: &PathBuf) {
    println!("\n-------------------------------------------------");
    println!("Updating repository to latest minor release...");
    println!("Path: {}", root_path.display().to_string());
    println!("Project: {}", project);
    println!("-------------------------------------------------\n");
    root_path.update_dependencies(project);
}
