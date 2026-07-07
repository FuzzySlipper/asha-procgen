use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let repo_root = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    match run_preflight(&repo_root) {
        Ok(summary) => {
            println!(
                "asha-procgen preflight OK: engine source {}, rust lane {}",
                summary.engine_source, summary.rust_dir
            );
        }
        Err(error) => {
            eprintln!("asha-procgen preflight failed:");
            eprintln!("- {error}");
            std::process::exit(1);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct PreflightSummary {
    engine_source: String,
    rust_dir: String,
}

fn run_preflight(repo_root: &Path) -> Result<PreflightSummary, String> {
    let engine_source = "../asha-engine";
    reject_private_engine_path("engine source", engine_source)?;

    let engine_root = repo_root.join(engine_source);
    if !engine_root.exists() {
        return Err(format!(
            "expected sibling ASHA engine checkout at {}",
            engine_root.display()
        ));
    }

    Ok(PreflightSummary {
        engine_source: engine_source.to_owned(),
        rust_dir: "procgen-rs".to_owned(),
    })
}

fn reject_private_engine_path(label: &str, value: &str) -> Result<(), String> {
    let forbidden_fragments = [
        "../asha-engine/engine-rs",
        "../asha-engine/ts/packages",
        "../asha/engine-rs",
        "../asha/ts/packages",
    ];
    for fragment in forbidden_fragments {
        if value.contains(fragment) {
            return Err(format!(
                "{label} must not reference private ASHA internals: {value}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_engine_paths() {
        let error = reject_private_engine_path("demo", "../asha-engine/engine-rs/crates/state")
            .expect_err("private engine path should be rejected");
        assert!(error.contains("private ASHA internals"));
    }
}
