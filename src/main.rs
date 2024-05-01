use std::{error::Error, fmt::Debug, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
use tokio::task::JoinHandle;

const XDG_NAME: &str = "flake-updates";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to a nix flake.lock file, or it's parent directory
    #[arg(short, long, default_value_t = String::from("."))]
    flake: String,

    /// How often to check GitHub for updates
    #[arg(short, long, default_value_t = 60)]
    poll: u32,

    /// Output string format ("%s is replaced with the number of updates")
    #[arg(short, long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let xdg_dir: &'static xdg::BaseDirectories = {
        let dir = xdg::BaseDirectories::with_prefix(XDG_NAME)
            .context("Failed to identify XDG state directory.")?;
        Box::leak(Box::new(dir))
    };

    let cache_dir = xdg_dir.get_state_file("cache");
    fs::create_dir_all(cache_dir.clone())?;

    let cache_dir_str = cache_dir
        .to_str()
        .context("Failed to parse XDG state directory.")?;
    let mins = &format!("+{}", args.poll);
    let output = std::process::Command::new("find")
        .args([cache_dir_str, "-type", "f", "-mmin", mins, "-delete"])
        .output()
        .context("Failed to obtain output of process that would have cleaned the cache.")?;

    if !output.status.success() {
        bail!(
            "Failed to clean XDG state directory of files older than 60 minutes: {:?}",
            String::from_utf8(output.stderr)
        );
    }

    let mut lock_file_path = PathBuf::from(args.flake);
    if lock_file_path.is_dir() {
        lock_file_path = lock_file_path.join("flake.lock");
    }
    let lock_file_parent = lock_file_path
        .parent()
        .context("Failed to find parent directory of lock file.")?;

    let lock: &'static serde_json::Value = {
        let lock_str =
            fs::read_to_string(&lock_file_path).context("Failed to read flake lock file.")?;
        let lock: serde_json::Value =
            serde_json::from_str(&lock_str).context("Failed to parse flake lock file JSON")?;
        Box::leak(Box::new(lock))
    };

    let inputs = lock["nodes"]["root"]["inputs"]
        .as_object()
        .context("Failed to deserialize node.root")?
        .clone();
    let nodes = &lock["nodes"];

    let mut handlers: Vec<JoinHandle<Result<Option<Update>>>> = vec![];
    for (name, id_value) in inputs {
        handlers.push(tokio::spawn(async move {
            let id = id_value
                .as_str()
                .context("Failed to deserialize input id")?;
            let input = match GitHubInput::new(&nodes, &name, id) {
                Err(e) => match e.downcast() {
                    Ok(UnsupportedInputType { r#type: _ }) => None,
                    Err(e) => bail!(e),
                },
                Ok(i) => Some(i),
            };
            let Some(input) = input else {
                return Ok(None);
            };

            let cache_path = xdg_dir.get_state_file(input.cache_path());

            let behind_by = {
                if cache_path.exists() {
                    let cache_str =
                        fs::read_to_string(cache_path).context("Failed to read from cache.")?;
                    let cache: serde_json::Value =
                        serde_json::from_str(&cache_str).context("Failed to deserialize cache.")?;
                    cache
                        .as_i64()
                        .context("Faield to deserialize commit data.")?
                } else {
                    let octocrab = octocrab::instance();
                    let commits = octocrab.commits(&input.owner, &input.repo);
                    let comparison = commits
                        .compare(&input.r#ref, &input.rev)
                        .send()
                        .await
                        .context("Failed to compare git commits via GitHub.")?;
                    let behind_by = comparison.behind_by;
                    let cache_str = serde_json::to_string(&behind_by)
                        .context("Failed to serialize commit data.")?;
                    let parent = cache_path
                        .parent()
                        .context("Failed to get cache parent directory.")?;
                    fs::create_dir_all(parent).context("Failed to create cache directory.")?;
                    fs::write(cache_path, &cache_str).context("Failed to write to cache ")?;
                    behind_by
                }
            };

            if behind_by == 0 {
                return Ok(None);
            }

            return Ok(Some(Update { input, behind_by }));
        }));
    }

    let mut updates: Vec<Update> = vec![];
    for handler in handlers.into_iter() {
        if let Some(update) = handler
            .await
            .context("Failed to await check for updates.")?
            .context("Failed to read input.")?
        {
            if args.output.is_none() {
                println!("{} has {} updates", update.input.name, update.behind_by);
            }
            updates.push(update);
        };
    }

    let total: i64 = updates.iter().map(|update| update.behind_by).sum();
    if let Some(output) = args.output {
        if total > 0 {
            let formatted_output = output.replace("%s", &total.to_string());
            println!("{formatted_output}");
        }
    } else {
        if total > 0 {
            let parent_str = lock_file_parent
                .to_str()
                .context("Failed to parse lock file parent directoy.")?;
            println!("Consider running `nix flake update {parent_str}`");
        } else {
            println!("Everything is up to date.");
        }
    }

    return Ok(());
}

struct Update {
    input: GitHubInput,
    behind_by: i64,
}

struct GitHubInput {
    name: String,
    _id: String,
    owner: String,
    repo: String,
    r#ref: String,
    rev: String,
}

impl GitHubInput {
    fn new(nodes: &serde_json::Value, name: &String, id: &str) -> Result<GitHubInput> {
        let node = &nodes[id];
        let original = &node["original"];
        let r#type = original["type"]
            .as_str()
            .context("Failed to deserialize original type.")?;
        if r#type != "github" {
            bail!(UnsupportedInputType {
                r#type: r#type.to_string()
            });
        }

        let owner = original["owner"]
            .as_str()
            .context("Failed to deserialize GitHub repository owner.")?;

        let repo = original["repo"]
            .as_str()
            .context("Failed to deserialize GitHub repository repo.")?;

        let r#ref = match original["ref"].as_str() {
            Some(r) => r,
            None => "HEAD",
        };

        let rev = node["locked"]["rev"]
            .as_str()
            .context("Failed to deserialize locked git commit.")?;

        Ok(GitHubInput {
            name: name.to_string(),
            _id: id.to_string(),
            owner: owner.to_string(),
            repo: repo.to_string(),
            r#ref: r#ref.to_string(),
            rev: rev.to_string(),
        })
    }

    fn cache_path(&self) -> String {
        format!(
            "cache/{}/{}/{}..{}.json",
            self.owner, self.repo, self.r#ref, self.rev
        )
    }
}

#[derive(Debug)]
struct UnsupportedInputType {
    r#type: String,
}

impl Error for UnsupportedInputType {}
impl std::fmt::Display for UnsupportedInputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unsupported input type '{}'", self.r#type)
    }
}
