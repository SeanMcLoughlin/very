use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use sv_parser::{ParseError, SourceUnit, SystemVerilogParser};

pub mod ast;
pub mod macros;

pub struct TestHarness {
    parser: SystemVerilogParser,
    fixtures_root: PathBuf,
}

impl TestHarness {
    pub fn new() -> Self {
        Self::with_parser(SystemVerilogParser::new(vec![], HashMap::new()))
    }

    pub fn with_parser(parser: SystemVerilogParser) -> Self {
        Self {
            parser,
            fixtures_root: default_fixtures_root(),
        }
    }

    #[allow(dead_code)]
    pub fn with_fixtures_root(mut self, root: PathBuf) -> Self {
        self.fixtures_root = root;
        self
    }

    pub fn fixtures_root(&self) -> &Path {
        &self.fixtures_root
    }

    pub fn fixture_path(&self, relative: &str) -> PathBuf {
        self.fixtures_root.join(relative)
    }

    pub fn read_fixture(&self, relative: &str) -> String {
        let path = self.fixture_path(relative);
        fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("Failed to read fixture {}: {}", path.display(), err);
        })
    }

    pub fn parse_fixture(&self, relative: &str) -> Result<SourceUnit, ParseError> {
        let content = self.read_fixture(relative);
        self.parser.parse_content(&content)
    }

    #[allow(dead_code)]
    pub fn parse_fixture_ok(&self, relative: &str) -> SourceUnit {
        self.parse_fixture(relative)
            .unwrap_or_else(|err| panic!("Failed to parse {}: {}", relative, err))
    }

    #[allow(dead_code)]
    pub fn parse_fixture_err(&self, relative: &str) -> ParseError {
        self.parse_fixture(relative).unwrap_err()
    }
}

impl Default for TestHarness {
    fn default() -> Self {
        Self::new()
    }
}

pub fn default_fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test_files")
}

pub fn iter_sv_files(relative_dir: &str) -> Vec<PathBuf> {
    let root = default_fixtures_root().join(relative_dir);
    if !root.exists() {
        return Vec::new();
    }

    let mut files: Vec<PathBuf> = fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("Failed to read directory {}: {}", root.display(), err))
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("sv") => Some(path),
                _ => None,
            }
        })
        .collect();
    files.sort();
    files
}

pub fn for_each_sv_file<F>(relative_dir: &str, mut visit: F)
where
    F: FnMut(&Path, Result<SourceUnit, ParseError>),
{
    let harness = TestHarness::default();
    let fixtures_root = harness.fixtures_root().to_path_buf();

    for path in iter_sv_files(relative_dir) {
        let relative = path
            .strip_prefix(&fixtures_root)
            .unwrap_or_else(|_| path.as_path());
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        let result = harness.parse_fixture(&relative_str);
        visit(&path, result);
    }
}

#[allow(dead_code)]
pub fn assert_directory_parses(relative_dir: &str) {
    for_each_sv_file(relative_dir, |path, result| {
        if let Err(err) = result {
            panic!(
                "Expected fixture {} to parse successfully: {}",
                path.display(),
                err
            );
        }
    });
}

#[allow(dead_code)]
pub fn assert_directory_fails(relative_dir: &str) {
    for_each_sv_file(relative_dir, |path, result| {
        if result.is_ok() {
            panic!(
                "Expected fixture {} to fail parsing but it succeeded",
                path.display()
            );
        }
    });
}

#[allow(dead_code)]
pub fn assert_parse_ok(relative: &str) -> SourceUnit {
    TestHarness::default().parse_fixture_ok(relative)
}

#[allow(dead_code)]
pub fn assert_parse_err(relative: &str) -> ParseError {
    TestHarness::default().parse_fixture_err(relative)
}
