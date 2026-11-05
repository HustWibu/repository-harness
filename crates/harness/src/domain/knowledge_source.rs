//! Source markers in hub knowledge pages (decision 0039): each marker pins a
//! cited line range in a member repository so the hub can tell when the
//! knowledge built on it has gone stale.

use sha2::{Digest, Sha256};

pub const SOURCE_MARKER_PREFIX: &str = "<!-- harness-source: ";
const SOURCE_MARKER_SUFFIX: &str = " -->";

/// `<!-- harness-source: <repository>@<revision>:<path>#L<start>-L<end> sha256=<hex> -->`
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceMarker {
    pub repository: String,
    pub revision: String,
    pub path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub sha256: String,
}

impl SourceMarker {
    pub fn parse(line: &str) -> Result<Self, String> {
        let body = line
            .trim()
            .strip_prefix(SOURCE_MARKER_PREFIX)
            .and_then(|rest| rest.strip_suffix(SOURCE_MARKER_SUFFIX))
            .ok_or("marker must be one line: <!-- harness-source: ... -->")?;
        let (location, hash) = body
            .split_once(" sha256=")
            .ok_or("marker needs ` sha256=<hex>`")?;
        let (repository, rest) = location.split_once('@').ok_or("marker needs `@`")?;
        let (revision, rest) = rest.split_once(':').ok_or("marker needs `:<path>`")?;
        let (path, range) = rest
            .rsplit_once("#L")
            .ok_or("marker needs `#L<start>-L<end>`")?;
        let (start, end) = range.split_once("-L").ok_or("marker needs `-L<end>`")?;
        let marker = Self {
            repository: repository.to_owned(),
            revision: revision.to_owned(),
            path: path.to_owned(),
            start_line: start.parse().map_err(|_| "start line is not a number")?,
            end_line: end.parse().map_err(|_| "end line is not a number")?,
            sha256: hash.to_owned(),
        };
        let hex = |value: &str, len: usize| {
            value.len() == len
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };
        if marker.repository.is_empty()
            || !marker
                .repository
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        {
            return Err("repository name is invalid".to_owned());
        }
        if !hex(&marker.revision, 40) {
            return Err("revision must be 40 lowercase hex characters".to_owned());
        }
        if !hex(&marker.sha256, 64) {
            return Err("sha256 must be 64 lowercase hex characters".to_owned());
        }
        if marker.path.is_empty()
            || marker.path.starts_with('/')
            || marker.path.split('/').any(|part| part == "..")
        {
            return Err("path must be repository-relative without `..`".to_owned());
        }
        if marker.start_line == 0 || marker.end_line < marker.start_line {
            return Err("line range must start at 1 and not end before it starts".to_owned());
        }
        Ok(marker)
    }
}

/// Every marker line in a page, with its 1-based line number.
pub fn source_markers(text: &str) -> Vec<(usize, Result<SourceMarker, String>)> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with(SOURCE_MARKER_PREFIX))
        .map(|(index, line)| (index + 1, SourceMarker::parse(line)))
        .collect()
}

/// Lines with their endings, split like Python's `bytes.splitlines(keepends=True)`,
/// which the onboarding emitter uses for the same digest.
fn lines(blob: &[u8]) -> Vec<&[u8]> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < blob.len() {
        let end = match blob[index] {
            b'\n' => index + 1,
            b'\r' if blob.get(index + 1) == Some(&b'\n') => index + 2,
            b'\r' => index + 1,
            _ => {
                index += 1;
                continue;
            }
        };
        out.push(&blob[start..end]);
        start = end;
        index = end;
    }
    if start < blob.len() {
        out.push(&blob[start..]);
    }
    out
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn range(blob: &[u8], start: usize, end: usize) -> Option<Vec<u8>> {
    let lines = lines(blob);
    (end <= lines.len()).then(|| lines[start - 1..end].concat())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Freshness {
    Current,
    /// The cited lines are unchanged but now start at this line.
    Moved(usize),
    Stale(String),
}

/// `pinned` is the file at the marker's revision, when that commit exists in
/// the local clone; `head` is the file at the member's HEAD.
pub fn assess(marker: &SourceMarker, pinned: Option<&[u8]>, head: Option<&[u8]>) -> Freshness {
    if let Some(pinned) = pinned {
        match range(pinned, marker.start_line, marker.end_line) {
            Some(cited) if sha256_hex(&cited) == marker.sha256 => {}
            _ => return Freshness::Stale("marker hash does not match its pinned revision".into()),
        }
    }
    let Some(head) = head else {
        return Freshness::Stale(format!("{} no longer exists at HEAD", marker.path));
    };
    if range(head, marker.start_line, marker.end_line)
        .is_some_and(|cited| sha256_hex(&cited) == marker.sha256)
    {
        return Freshness::Current;
    }
    let Some(cited) = pinned.and_then(|p| range(p, marker.start_line, marker.end_line)) else {
        return Freshness::Stale(
            "cited lines changed; the pinned revision is not in the local clone".into(),
        );
    };
    let head_lines = lines(head);
    let width = marker.end_line - marker.start_line + 1;
    head_lines
        .windows(width)
        .position(|window| window.concat() == cited)
        .map(|index| Freshness::Moved(index + 1))
        .unwrap_or_else(|| Freshness::Stale("cited lines changed at HEAD".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(start: usize, end: usize, cited: &[u8]) -> SourceMarker {
        SourceMarker {
            repository: "orders".into(),
            revision: "a".repeat(40),
            path: "src/App.java".into(),
            start_line: start,
            end_line: end,
            sha256: sha256_hex(cited),
        }
    }

    #[test]
    fn parses_markers_and_rejects_malformed_ones() {
        let line = format!(
            "<!-- harness-source: orders@{}:src/App.java#L2-L3 sha256={} -->",
            "a".repeat(40),
            "b".repeat(64)
        );
        let parsed = SourceMarker::parse(&line).unwrap();
        assert_eq!((parsed.start_line, parsed.end_line), (2, 3));
        assert_eq!(parsed.path, "src/App.java");
        for bad in [
            line.replace("#L2-L3", "#L0-L3"),
            line.replace("#L2-L3", "#L3-L2"),
            line.replace("src/App.java", "../App.java"),
            line.replace(&"a".repeat(40), "HEAD"),
            line.replace(" sha256=", " sha="),
        ] {
            assert!(SourceMarker::parse(&bad).is_err(), "{bad}");
        }
        let page = format!("# Page\nclaim\n{line}\n<!-- harness-source: broken -->\n");
        let markers = source_markers(&page);
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].0, 3);
        assert!(markers[1].1.is_err());
    }

    #[test]
    fn splits_lines_like_python_splitlines() {
        assert_eq!(
            lines(b"a\nb\r\nc\rd"),
            vec![&b"a\n"[..], b"b\r\n", b"c\r", b"d"]
        );
    }

    #[test]
    fn assesses_current_moved_and_stale_sources() {
        let pinned = b"class A {\n  send();\n}\n";
        let m = marker(2, 2, b"  send();\n");
        assert_eq!(assess(&m, Some(pinned), Some(pinned)), Freshness::Current);
        assert_eq!(assess(&m, None, Some(pinned)), Freshness::Current);

        let moved = b"// header\nclass A {\n  send();\n}\n";
        assert_eq!(assess(&m, Some(pinned), Some(moved)), Freshness::Moved(3));

        let changed = b"class A {\n  sendAsync();\n}\n";
        assert!(matches!(
            assess(&m, Some(pinned), Some(changed)),
            Freshness::Stale(_)
        ));
        assert!(matches!(
            assess(&m, None, Some(changed)),
            Freshness::Stale(_)
        ));
        assert!(matches!(
            assess(&m, Some(pinned), None),
            Freshness::Stale(_)
        ));

        let forged = marker(2, 2, b"other\n");
        assert!(matches!(
            assess(&forged, Some(pinned), Some(pinned)),
            Freshness::Stale(reason) if reason.contains("pinned revision")
        ));
    }
}
