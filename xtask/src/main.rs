use std::env;
use std::fs;
use std::process::{Command, exit};

const APP_REMOTE: &str = "origin";
const APP_RELEASE_BRANCH: &str = "main";
const APP_DEV_BRANCH: &str = "dev";
const APP_MANIFEST: &str = "Cargo.toml";
const APP_LOCKFILE: &str = "Cargo.lock";
const APP_TAG_PREFIX: &str = "v";
const CHANGELOG: &str = "CHANGELOG.md";

fn run_git(args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .status()
        .expect("Failed to execute git command");

    if !status.success() {
        eprintln!("Error: 'git {}' failed. Aborting.", args.join(" "));
        exit(1);
    }
}

fn run_cargo(args: &[&str]) {
    let status = Command::new("cargo")
        .args(args)
        .status()
        .expect("Failed to execute cargo command");

    if !status.success() {
        eprintln!("Error: 'cargo {}' failed. Aborting.", args.join(" "));
        exit(1);
    }
}

fn git_succeeds(args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn released_versions() -> Vec<(u64, u64, u64)> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", "--refs", APP_REMOTE])
        .output()
        .expect("Failed to execute git ls-remote");

    if !output.status.success() {
        eprintln!("Error: could not list the tags on {APP_REMOTE}. Is GitHub reachable?");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.rsplit_once("refs/tags/")?.1.strip_prefix(APP_TAG_PREFIX).and_then(parse_version))
        .collect()
}

fn this_month() -> (u64, u64) {
    let output = Command::new("date").arg("+%y %-m").output().expect("Failed to execute date");
    let text = String::from_utf8_lossy(&output.stdout);
    let mut parts = text.split_whitespace().map(|part| part.parse::<u64>().ok());

    let (Some(Some(year)), Some(Some(month))) = (parts.next(), parts.next()) else {
        eprintln!("Error: could not read the current year and month from 'date'.");
        exit(1);
    };

    (year, month)
}

fn next_version(released: &[(u64, u64, u64)], current: Option<(u64, u64, u64)>) -> String {
    let (year, month) = this_month();
    let count = released
        .iter()
        .copied()
        .chain(current)
        .filter(|&(y, m, _)| (y, m) == (year, month))
        .map(|(_, _, n)| n + 1)
        .max()
        .unwrap_or(0);

    format!("{year}.{month}.{count}")
}

fn ensure_changelog(last: Option<(u64, u64, u64)>) {
    let notes = fs::read_to_string(CHANGELOG).unwrap_or_default();

    if notes.trim().is_empty() {
        eprintln!("Error: {CHANGELOG} is empty. It becomes the release notes, so write them first.");
        exit(1);
    }

    run_git(&["add", CHANGELOG]);

    let Some((major, minor, patch)) = last else { return; };
    let tag = format!("{APP_TAG_PREFIX}{major}.{minor}.{patch}");

    run_git(&["fetch", "--no-tags", APP_REMOTE, "tag", &tag]);

    if git_succeeds(&["diff", "--cached", "--quiet", &tag, "--", CHANGELOG]) {
        eprintln!("Error: {CHANGELOG} is unchanged since {tag}. Write the notes for this release first.");
        exit(1);
    }
}

fn stamp_version(old: &str, new: &str) {
    let manifest = fs::read_to_string(APP_MANIFEST).expect("Failed to read the workspace manifest");
    let stamped = manifest.replacen(&format!("version = \"{old}\""), &format!("version = \"{new}\""), 1);

    fs::write(APP_MANIFEST, stamped).expect("Failed to write the workspace manifest");
}

fn ship() {
    println!("Starting BCC Release...");

    println!("> Reading the releases on {APP_REMOTE}...");
    let released = released_versions();
    ensure_changelog(released.iter().copied().max());

    let current = manifest_version(APP_MANIFEST);
    let version = next_version(&released, parse_version(&current));

    if version != current {
        println!("> Stamping {APP_TAG_PREFIX}{version} (was {current})...");
        stamp_version(&current, &version);
    }

    println!("> Verifying build state with cargo check...");
    run_cargo(&["check"]);

    let release_files = [APP_MANIFEST, APP_LOCKFILE, CHANGELOG];
    let tag = format!("{APP_TAG_PREFIX}{version}");

    run_git(&[&["add"][..], &release_files].concat());

    if !git_succeeds(&[&["diff", "--cached", "--quiet", "--"][..], &release_files].concat()) {
        println!("> Committing {tag}...");
        run_git(&[&["commit", "-m", &tag, "--"][..], &release_files].concat());
    }

    println!("> Checking out {APP_RELEASE_BRANCH} and pulling latest changes...");
    run_git(&["checkout", APP_RELEASE_BRANCH]);
    run_git(&["pull"]);

    println!("> Merging {APP_DEV_BRANCH} into {APP_RELEASE_BRANCH}...");
    run_git(&["merge", APP_DEV_BRANCH]);

    println!("> Pushing {APP_RELEASE_BRANCH} to remote...");
    run_git(&["push"]);

    println!("> Returning to {APP_DEV_BRANCH} branch...");
    run_git(&["checkout", APP_DEV_BRANCH]);

    println!("{tag} shipped! The release workflow builds and publishes it with {CHANGELOG} as the notes.");
}

fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.trim().split('.').map(|part| part.parse::<u64>().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);

    parts.next().is_none().then_some(version)
}

fn manifest_version(path: &str) -> String {
    let manifest = fs::read_to_string(path).expect("Failed to read a Cargo manifest");

    manifest
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version")?.trim_start().strip_prefix('=')?.trim().strip_prefix('"')?.strip_suffix('"'))
        .expect("The Cargo manifest has no version line")
        .to_string()
}

fn main() {
    if let Some(unknown) = env::args().nth(1) {
        eprintln!("Unknown xtask command '{unknown}'. Use 'cargo ship'.");
        exit(2);
    }

    ship();
}
