use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(p) = stack.pop() {
        let entries = match fs::read_dir(&p) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for ent in entries.flatten() {
            let path = ent.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn qir_ai_does_not_depend_on_qir_core_metrics_modules() {
    // Guardrail: qir_ai must not import qir_core metric computation modules.
    // (AI may draft text, but must never compute deterministic metrics/rollups.)
    let src_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = collect_rs_files(&src_root);
    assert!(!files.is_empty());

    for f in files {
        let text = fs::read_to_string(&f).unwrap_or_default();
        assert!(
            !text.contains("qir_core::metrics"),
            "forbidden dependency found in {}",
            f.display()
        );
        assert!(
            !text.contains("compute_incident_metrics"),
            "forbidden metrics call found in {}",
            f.display()
        );
    }
}

#[test]
#[ignore]
fn phase5_placeholder_ai_outputs_require_citations_end_to_end() {
    // Placeholder for Phase 5: end-to-end flows that generate AI drafts must hard-fail
    // without citations. Enable once AI drafting endpoints are introduced.
    assert!(true);
}

