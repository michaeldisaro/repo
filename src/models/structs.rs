use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::ParseError;
use structopt::StructOpt;

fn parse_path(src: &str) -> Result<PathBuf, ParseError> {
    return PathBuf::from_str(&src)
        .map(|p| fs::canonicalize(p).expect(""))
        .map_err(|e| e);
}

#[derive(StructOpt)]
pub struct Params {
    pub action: String,
    pub project: String,
    #[structopt(parse(try_from_str = parse_path),default_value=".")]
    pub path: PathBuf,
}

pub struct RepositoryItem {
    pub project: String,
    pub module: String,
    pub dependencies: Vec<String>,
}

impl RepositoryItem {
    pub fn new(project: String, module: String, dependencies: Vec<String>) -> Self {
        Self {
            project,
            module,
            dependencies,
        }
    }
}
