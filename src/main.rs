mod interp;
mod tree;

use anyhow::{Context, Result};
use git::Repository;

fn main() -> Result<()> {
    let path = std::env::args().nth(1).context("path required")?;

    let repo = Repository::open(path)?;

    let start = repo
        .find_reference("refs/tags/_start")
        .and_then(|r| r.peel_to_commit())
        .context("missing _start tag")?;
    let end = repo
        .find_reference("refs/tags/_end")
        .and_then(|r| r.peel_to_commit())
        .context("missing _end tag")?;

    let mut instance = interp::Instance::new();
    instance.run(&repo, start, end)
}

fn replace<'a>(repo: &'a Repository, commit: &mut git::Commit<'a>) -> Option<git::Oid> {
    let id = commit.id();
    let mut replaced = false;
    while let Ok(replace) = repo
        .find_reference(&format!("refs/replace/{}", commit.id()))
        .and_then(|r| r.peel_to_commit())
    {
        *commit = replace;
        replaced = true;
    }
    replaced.then(|| id)
}
