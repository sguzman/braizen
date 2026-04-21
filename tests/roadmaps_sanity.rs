use std::path::PathBuf;

fn roadmap_files() -> Vec<PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir("docs/roadmaps")
        .expect("docs/roadmaps must exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
        .collect();
    files.sort();
    files
}

#[test]
fn roadmaps_have_checklists() {
    let files = roadmap_files();
    assert!(!files.is_empty(), "expected at least one roadmap file");

    for path in files {
        let text = std::fs::read_to_string(&path).expect("read roadmap");
        let total = text
            .lines()
            .filter(|line| line.trim_start().starts_with("- ["))
            .count();
        assert!(
            total > 0,
            "expected checkboxes in roadmap: {}",
            path.display()
        );
    }
}

