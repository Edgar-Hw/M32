use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    let repo_root = manifest_dir.join("../..");

    register_git_rerun_paths(&repo_root);
    println!("cargo:rerun-if-env-changed=RUSTC");

    let git_commit = git_commit(&repo_root).unwrap_or_else(|| "unknown".to_owned());
    let rust_version = rust_version().unwrap_or_else(|| "unknown".to_owned());
    let target = env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_owned());

    println!("cargo:rustc-env=M32_GIT_COMMIT={git_commit}");
    println!("cargo:rustc-env=M32_RUST_VERSION={rust_version}");
    println!("cargo:rustc-env=M32_BUILD_TARGET={target}");
    println!("cargo:rustc-env=M32_BUILD_PROFILE={profile}");
}

fn register_git_rerun_paths(repo_root: &Path) {
    let git_dir = repo_root.join(".git");
    let head_path = git_dir.join("HEAD");

    println!("cargo:rerun-if-changed={}", head_path.display());

    let Ok(head) = fs::read_to_string(&head_path) else {
        return;
    };

    let Some(reference) = head.trim().strip_prefix("ref: ") else {
        return;
    };

    println!("cargo:rerun-if-changed={}", git_dir.join(reference).display());
}

fn git_commit(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;

    successful_stdout(output)
}

fn rust_version() -> Option<String> {
    let rustc = env::var("RUSTC").ok()?;
    let output = Command::new(rustc).arg("--version").output().ok()?;

    successful_stdout(output)
}

fn successful_stdout(output: std::process::Output) -> Option<String> {
    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();

    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}
