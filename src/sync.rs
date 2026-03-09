use std::path::PathBuf;

use clap::Args;
use thiserror::Error;

use crate::process::{exec, exec_result, ProcessError};

fn parse_bundle_branches(output: &str) -> Vec<String> {
    output
        .split('\n')
        .filter(|s| !s.is_empty() && s.contains("refs/heads/"))
        .map(|s| s.split(' ').collect::<Vec<&str>>()[1].replace("refs/heads/", ""))
        .collect()
}

fn parse_current_branches(output: &str) -> Vec<String> {
    output
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.replace(['*', ' '], ""))
        .collect()
}

fn find_main_branch(branches: &[String]) -> &str {
    branches
        .iter()
        .find(|b| *b == "develop" || *b == "master")
        .map_or("main", |b| b)
}

#[derive(Error, Debug)]
pub enum SyncError {
    #[error("ProcessError: {0}")]
    Process(#[from] ProcessError),
}

#[derive(Args)]
pub struct Sync {
    /// Path to the bundle file
    bundle: PathBuf,

    #[clap(skip)]
    bundle_branches: Vec<String>,
    #[clap(skip)]
    current_branches: Vec<String>,
}

impl Sync {
    pub fn new(bundle: PathBuf) -> Self {
        Self {
            bundle,
            bundle_branches: Vec::new(),
            current_branches: Vec::new(),
        }
    }

    fn get_bundle_branches(&mut self) -> Result<(), SyncError> {
        let bundle_branches_str = exec_result(
            "git",
            vec!["bundle", "list-heads", &self.bundle.display().to_string()],
        )?;

        self.bundle_branches = parse_bundle_branches(&bundle_branches_str);
        Ok(())
    }

    fn get_current_branches(&mut self) -> Result<(), SyncError> {
        let current_branches_str = exec_result("git", vec!["branch"])?;

        self.current_branches = parse_current_branches(&current_branches_str);
        Ok(())
    }

    fn get_current_branch(&self) -> Result<String, SyncError> {
        let current_branch =
            exec_result("git", vec!["rev-parse", "--abbrev-ref", "HEAD"])?.replace("\n", "");

        Ok(current_branch)
    }

    fn checkout_to_main_branch(&self) -> Result<(), SyncError> {
        let current_branch = self.get_current_branch()?;
        let main_branch = find_main_branch(&self.current_branches);

        if !exec_result("git", vec!["status", "--porcelain"])?.is_empty() {
            exec("git", vec!["stash", "--include-untracked"])?;
        }

        if current_branch != main_branch {
            exec("git", vec!["checkout", main_branch])?;
        }

        Ok(())
    }

    fn remove_old_branches(&self) -> Result<(), SyncError> {
        let removed_branches = self
            .current_branches
            .iter()
            .filter(|b| !self.bundle_branches.contains(b))
            .collect::<Vec<&String>>();

        for branch in removed_branches {
            exec("git", vec!["branch", "-D", branch])?;
            println!("Branch '{}' is removed.", branch);
        }

        Ok(())
    }

    fn add_new_branches(&self) -> Result<(), SyncError> {
        let added_branches = self
            .bundle_branches
            .iter()
            .filter(|b| !self.current_branches.contains(b))
            .collect::<Vec<&String>>();

        for branch in added_branches {
            exec("git", vec!["branch", branch])?;
            println!("Branch '{}' is added.", branch);
        }

        Ok(())
    }

    fn update_branches(&self) -> Result<(), SyncError> {
        for branch in self.bundle_branches.clone() {
            if branch != self.get_current_branch()? {
                exec("git", vec!["checkout", &branch])?;
            }

            exec(
                "git",
                vec!["pull", &self.bundle.display().to_string(), &branch],
            )?;
            println!("Branch '{}' is updated.", branch);
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), SyncError> {
        self.get_bundle_branches()?;
        self.get_current_branches()?;

        let current_branch = self.get_current_branch()?;

        self.checkout_to_main_branch()?;
        self.remove_old_branches()?;
        self.add_new_branches()?;
        self.update_branches()?;

        if self.bundle_branches.contains(&current_branch)
            && current_branch != self.get_current_branch()?
        {
            exec("git", vec!["checkout", &current_branch])?;
        }

        exec("git", vec!["fetch", "--prune"])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bundle_branches() {
        let output =
            "abc123 refs/heads/main\ndef456 refs/heads/feature-branch\nghi789 refs/heads/develop\n";
        let branches = parse_bundle_branches(output);

        assert_eq!(branches.len(), 3);
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"feature-branch".to_string()));
        assert!(branches.contains(&"develop".to_string()));
    }

    #[test]
    fn test_parse_bundle_branches_empty() {
        let output = "";
        let branches = parse_bundle_branches(output);
        assert!(branches.is_empty());
    }

    #[test]
    fn test_parse_bundle_branches_filters_non_heads() {
        let output = "abc123 refs/tags/v1.0.0\ndef456 refs/heads/main\n";
        let branches = parse_bundle_branches(output);

        assert_eq!(branches.len(), 1);
        assert!(branches.contains(&"main".to_string()));
    }

    #[test]
    fn test_parse_current_branches() {
        let output = "  feature-branch\n* main\n  develop\n";
        let branches = parse_current_branches(output);

        assert_eq!(branches.len(), 3);
        assert!(branches.contains(&"main".to_string()));
        assert!(branches.contains(&"feature-branch".to_string()));
        assert!(branches.contains(&"develop".to_string()));
    }

    #[test]
    fn test_parse_current_branches_empty() {
        let output = "";
        let branches = parse_current_branches(output);

        assert!(branches.is_empty());
    }

    #[test]
    fn test_find_main_branch_with_develop() {
        let branches = vec![
            "feature".to_string(),
            "develop".to_string(),
            "test".to_string(),
        ];

        assert_eq!(find_main_branch(&branches), "develop");
    }

    #[test]
    fn test_find_main_branch_with_master() {
        let branches = vec![
            "feature".to_string(),
            "master".to_string(),
            "test".to_string(),
        ];

        assert_eq!(find_main_branch(&branches), "master");
    }

    #[test]
    fn test_find_main_branch_fallback_to_main() {
        let branches = vec!["feature".to_string(), "test".to_string()];

        assert_eq!(find_main_branch(&branches), "main");
    }

    #[test]
    fn test_find_main_branch_empty() {
        let branches: Vec<String> = vec![];

        assert_eq!(find_main_branch(&branches), "main");
    }

    #[test]
    fn test_sync_new() {
        let sync = Sync::new(PathBuf::from("/tmp/test.bundle"));

        assert_eq!(sync.bundle, PathBuf::from("/tmp/test.bundle"));
        assert!(sync.bundle_branches.is_empty());
        assert!(sync.current_branches.is_empty());
    }
}
