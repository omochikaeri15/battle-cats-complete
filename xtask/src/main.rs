use std::env;
use std::fs;
use std::process::{Command, exit};

const NYANKO_PREFIX: &str = "data";
const NYANKO_REMOTE: &str = "nyanko";
const NYANKO_URL: &str = "git@github.com:omochikaeri15/nyanko.git";
const NYANKO_BRANCH: &str = "main";
const NYANKO_MANIFEST: &str = "data/Cargo.toml";
const NYANKO_TAG_PREFIX: &str = "v";

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

fn git_stdout(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .expect("Failed to execute git command");

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn ship() {
    println!("Starting BCC Release...");

    println!("> Verifying build state with cargo check...");
    run_cargo(&["check"]);

    println!("> Checking out main and pulling latest changes...");
    run_git(&["checkout", "main"]);
    run_git(&["pull"]);

    println!("> Merging nightly into main...");
    run_git(&["merge", "nightly"]);

    println!("> Pushing main to remote...");
    run_git(&["push"]);

    println!("> Returning to nightly branch...");
    run_git(&["checkout", "nightly"]);

    println!("Release successful! Branches are even.");
}

fn ensure_nyanko_remote() {
    if git_succeeds(&["remote", "get-url", NYANKO_REMOTE]) {
        return;
    }

    println!("> Adding the '{NYANKO_REMOTE}' remote ({NYANKO_URL})...");
    run_git(&["remote", "add", NYANKO_REMOTE, NYANKO_URL]);
}

fn nyanko_check() {
    println!("> Running the nyanko gate...");
    run_cargo(&["check", "-p", "nyanko", "--all-targets", "--all-features"]);
    run_cargo(&["clippy", "-p", "nyanko", "--all-targets", "--all-features", "--", "-D", "warnings"]);
    run_cargo(&["doc", "-p", "nyanko", "--no-deps", "--all-features"]);
    run_cargo(&["test", "-p", "nyanko", "--all-features"]);
    println!("nyanko gate passed.");
}

fn ensure_nyanko_committed() {
    let pending = git_stdout(&["status", "--porcelain", "--", NYANKO_PREFIX]);

    if !pending.is_empty() {
        eprintln!("Error: '{NYANKO_PREFIX}/' has uncommitted changes. Commit them first; a subtree push only carries committed history.");
        exit(1);
    }
}

fn subtree_push() {
    let prefix = format!("--prefix={NYANKO_PREFIX}");

    println!("> Pushing '{NYANKO_PREFIX}/' history to {NYANKO_REMOTE}/{NYANKO_BRANCH}...");
    run_git(&["subtree", "push", &prefix, NYANKO_REMOTE, NYANKO_BRANCH]);
}

fn nyanko_push() {
    ensure_nyanko_committed();
    nyanko_check();
    ensure_nyanko_remote();
    subtree_push();
    println!("nyanko pushed.");
}

fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let mut parts = text.trim().split('.').map(|part| part.parse::<u64>().ok());
    let version = (parts.next()??, parts.next()??, parts.next()??);

    parts.next().is_none().then_some(version)
}

fn local_version() -> String {
    let manifest = fs::read_to_string(NYANKO_MANIFEST).expect("Failed to read the nyanko manifest");

    manifest
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version")?.trim_start().strip_prefix('=')?.trim().strip_prefix('"')?.strip_suffix('"'))
        .expect("The nyanko manifest has no version line")
        .to_string()
}

fn published_version() -> Option<String> {
    let output = Command::new("cargo")
        .args(["search", "nyanko", "--limit", "1"])
        .output()
        .expect("Failed to execute cargo search");

    if !output.status.success() {
        eprintln!("Error: 'cargo search nyanko' failed. Is crates.io reachable?");
        exit(1);
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("nyanko = \"")?.split('"').next().map(str::to_string))
}

fn nyanko_publish() {
    ensure_nyanko_committed();

    let local = local_version();
    let Some(local_parsed) = parse_version(&local) else {
        eprintln!("Error: the nyanko manifest version '{local}' is not a plain major.minor.patch.");
        exit(1);
    };

    println!("> Comparing {local} against crates.io...");

    match published_version() {
        Some(published) if parse_version(&published).is_some_and(|parsed| parsed >= local_parsed) => {
            eprintln!("Error: crates.io already has nyanko {published}; bump the version in {NYANKO_MANIFEST} above it first.");
            exit(1);
        }
        Some(published) => println!("> {published} is the latest published; {local} is new."),
        None => println!("> nyanko is not on crates.io yet; {local} will be the first release."),
    }

    let tag = format!("{NYANKO_TAG_PREFIX}{local}");

    ensure_nyanko_remote();

    if !git_stdout(&["ls-remote", "--tags", NYANKO_REMOTE, &tag]).is_empty() {
        eprintln!("Error: tag {tag} already exists on {NYANKO_REMOTE}. Bump the version in {NYANKO_MANIFEST} first.");
        exit(1);
    }

    nyanko_check();
    subtree_push();

    let prefix = format!("--prefix={NYANKO_PREFIX}");
    let split = git_stdout(&["subtree", "split", &prefix]);

    if split.is_empty() {
        eprintln!("Error: could not split the '{NYANKO_PREFIX}/' history to find the commit to tag.");
        exit(1);
    }

    println!("> Tagging {tag} on {NYANKO_REMOTE}; its publish workflow releases the crate from the tag...");
    run_git(&["push", NYANKO_REMOTE, &format!("{split}:refs/tags/{tag}")]);
    println!("nyanko {local} tagged. Watch the 'Publish to crates.io' workflow on the nyanko repo for the release.");
}

fn nyanko_pull() {
    ensure_nyanko_remote();

    let prefix = format!("--prefix={NYANKO_PREFIX}");

    println!("> Pulling {NYANKO_REMOTE}/{NYANKO_BRANCH} into '{NYANKO_PREFIX}/'...");
    run_git(&["subtree", "pull", &prefix, NYANKO_REMOTE, NYANKO_BRANCH, "-m", "nyanko pulled from upstream"]);
    println!("nyanko pulled.");
}

fn nyanko(command: Option<&str>) {
    match command {
        Some("check") => nyanko_check(),
        Some("push") => nyanko_push(),
        Some("pull") => nyanko_pull(),
        Some("publish") => nyanko_publish(),
        other => {
            if let Some(unknown) = other {
                eprintln!("Unknown nyanko command '{unknown}'.");
            }

            eprintln!("Usage: cargo nyanko <check|push|pull|publish>");
            eprintln!("  check    run nyanko's own gate: check, clippy, doc and tests with every feature");
            eprintln!("  push     run the gate, then publish the '{NYANKO_PREFIX}/' history to {NYANKO_REMOTE}/{NYANKO_BRANCH}");
            eprintln!("  pull     merge {NYANKO_REMOTE}/{NYANKO_BRANCH} back into '{NYANKO_PREFIX}/'");
            eprintln!("  publish  push, then tag {NYANKO_TAG_PREFIX}<version> on {NYANKO_REMOTE} so its workflow releases to crates.io; refuses a version crates.io already has");
            exit(2);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        None => ship(),
        Some("nyanko") => nyanko(args.get(1).map(String::as_str)),
        Some(unknown) => {
            eprintln!("Unknown xtask command '{unknown}'. Use 'cargo ship' or 'cargo nyanko <check|push|pull|publish>'.");
            exit(2);
        }
    }
}
