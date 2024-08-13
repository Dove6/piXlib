use std::ops::Deref;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ScenePath {
    pub dir_path: Path,
    pub file_path: Path,
}

impl ScenePath {
    pub fn new(dir_path: &str, file_path: &str) -> Self {
        Self {
            dir_path: dir_path.into(),
            file_path: file_path.into(),
        }
    }

    pub fn with_file_path(&self, file_path: &str) -> Self {
        Self {
            dir_path: self.dir_path.clone(),
            file_path: file_path.into(),
        }
    }

    pub fn to_string(&self) -> String {
        self.dir_path.with_appended(&self.file_path).to_string()
    }
}

impl From<ScenePath> for String {
    fn from(value: ScenePath) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path {
    buffer: String,
}

lazy_static! {
    static ref MULTIPLE_SLASH_REGEX: Regex =
        Regex::new(r"/+").expect("The regex for multiple slashes should be defined correctly.");
    static ref START_SLASH_REGEX: Regex =
        Regex::new(r"^/").expect("The regex for a slash at the start should be defined correctly.");
    static ref END_SLASH_REGEX: Regex =
        Regex::new(r"/$").expect("The regex for a slash at the end should be defined correctly.");
    static ref SAME_DIR_REGEX: Regex =
        Regex::new(r"/(\./)+").expect("The regex for same dir should be defined correctly.");
    static ref SAME_DIR_START_REGEX: Regex = Regex::new(r"^(\./)+")
        .expect("The regex for same dir at the start should be defined correctly.");
    static ref SAME_DIR_END_REGEX: Regex = Regex::new(r"(/\.)+$")
        .expect("The regex for same dir at the end should be defined correctly.");
    static ref PARENT_DIR_REGEX: Regex =
        Regex::new(r"/(([^/]{3,}|[^/\.]|\.[^/\.]|[^/\.]\.)/\.\./)+")
            .expect("The regex for parent dir should be defined correctly.");
    static ref PARENT_DIR_START_REGEX: Regex =
        Regex::new(r"^([^/]{3,}|[^/\.]|\.[^/\.]|[^/\.]\.)/\.\./")
            .expect("The regex for parent dir at the start should be defined correctly.");
    static ref PARENT_DIR_END_REGEX: Regex =
        Regex::new(r"/([^/]{3,}|[^/\.]|\.[^/\.]|[^/\.]\.)/\.\.$")
            .expect("The regex for parent dir at the end should be defined correctly.");
    static ref PARENT_DIR_START_END_REGEX: Regex =
        Regex::new(r"^([^/]{3,}|[^/\.]|\.[^/\.]|[^/\.]\.)/\.\.$")
            .expect("The regex for parent dir at the end should be defined correctly.");
}

fn canonicalize_str(path_str: &str) -> String {
    let result = path_str.to_uppercase().replace('\\', "/");
    let result = MULTIPLE_SLASH_REGEX.replace_all(&result, "/");
    let result = START_SLASH_REGEX.replace(&result, "");
    let result = END_SLASH_REGEX.replace(&result, "");
    let result = SAME_DIR_START_REGEX.replace(&result, "");
    let result = SAME_DIR_END_REGEX.replace(&result, "");
    let mut result = SAME_DIR_REGEX.replace_all(&result, "/").to_string();
    while PARENT_DIR_REGEX.is_match(&result) {
        result = PARENT_DIR_REGEX.replace_all(&result, "/").to_string();
    }
    let result = PARENT_DIR_START_REGEX.replace(&result, "");
    let result = PARENT_DIR_END_REGEX.replace(&result, "");
    let result = PARENT_DIR_START_END_REGEX.replace(&result, ".");
    result.to_string()
}

fn iter_segments(path_str: &str) -> impl Iterator<Item = &str> + DoubleEndedIterator {
    path_str.split('/')
}

fn remove_first_n_segments(path_str: &mut String, n: usize) {
    if n == 0 {
        return;
    }
    if let Some((split_position, _)) = path_str.match_indices('/').skip(n).next() {
        path_str.drain(..(split_position + 1));
    } else {
        path_str.clear();
        path_str.push('.');
    }
}

fn remove_last_n_segments(path_str: &mut String, n: usize) {
    if n == 0 {
        return;
    }
    if let Some((split_position, _)) = path_str.rmatch_indices('/').skip(n).next() {
        path_str.drain(split_position..);
    } else {
        path_str.clear();
        path_str.push('.');
    }
}

impl Path {
    pub fn from(path_str: &str) -> Self {
        Self {
            buffer: canonicalize_str(path_str),
        }
    }

    pub fn append(&mut self, path_str: &str) {
        let mut canonicalized_suffix = canonicalize_str(path_str);
        let appended_path_parent_path_segment_count = iter_segments(&canonicalized_suffix)
            .take_while(|s| *s == "..")
            .count();
        let removable_segment_count = iter_segments(&self.buffer)
            .rev()
            .take_while(|s| *s != ".." && *s != ".")
            .count();
        let number_of_segments_to_nullify =
            removable_segment_count.min(appended_path_parent_path_segment_count);
        remove_first_n_segments(&mut canonicalized_suffix, number_of_segments_to_nullify);
        remove_last_n_segments(&mut self.buffer, number_of_segments_to_nullify);
        if self.buffer != "." && canonicalized_suffix != "." {
            self.buffer.push('/');
        } else if canonicalized_suffix != "." {
            self.buffer.clear();
        } else {
            return;
        }
        self.buffer.push_str(&canonicalized_suffix);
    }

    pub fn with_appended(&self, path_str: &str) -> Self {
        let mut cloned = self.clone();
        cloned.append(path_str);
        cloned
    }

    pub fn prepend(&mut self, path_str: &str) {
        let mut canonicalized_prefix = canonicalize_str(path_str);
        let parent_path_segment_count = iter_segments(&self.buffer)
            .take_while(|s| *s == "..")
            .count();
        let prepended_path_removable_segment_count = iter_segments(&canonicalized_prefix)
            .rev()
            .take_while(|s| *s != ".." && *s != ".")
            .count();
        let number_of_segments_to_nullify =
            parent_path_segment_count.min(prepended_path_removable_segment_count);
        remove_first_n_segments(&mut self.buffer, number_of_segments_to_nullify);
        remove_last_n_segments(&mut canonicalized_prefix, number_of_segments_to_nullify);
        if self.buffer != "." && canonicalized_prefix != "." {
            self.buffer.insert(0, '/');
        } else if canonicalized_prefix != "." {
            self.buffer.clear();
        } else {
            return;
        }
        self.buffer.insert_str(0, &canonicalized_prefix);
    }

    pub fn with_prepended(&self, path_str: &str) -> Self {
        let mut cloned = self.clone();
        cloned.prepend(path_str);
        cloned
    }

    pub fn to_string(&self) -> String {
        self.buffer.clone()
    }
}

impl From<Path> for String {
    fn from(value: Path) -> Self {
        value.to_string()
    }
}

impl From<&str> for Path {
    fn from(value: &str) -> Self {
        Self::from(value)
    }
}

impl AsRef<str> for Path {
    fn as_ref(&self) -> &str {
        &self.buffer
    }
}

impl Deref for Path {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("slashes only", "A/B/C", "A/B/C")]
    #[test_case("backslashes only", "A\\B\\C", "A/B/C")]
    #[test_case("mixed", "A/B\\C", "A/B/C")]
    #[test_case("multiple slashes", "A////B//C/D", "A/B/C/D")]
    #[test_case("multiple backslashes", "A\\\\\\\\B\\\\C\\D", "A/B/C/D")]
    #[test_case("multiple mixed", "A/\\\\/B//C\\D", "A/B/C/D")]
    #[test_case("slash at the start", "/A/B/C", "A/B/C")]
    #[test_case("slash at the end", "A/B/C/", "A/B/C")]
    fn test_slashes_are_canonicalized_correctly(
        _description: &str,
        path_str: &str,
        expected: &str,
    ) {
        assert_eq!(canonicalize_str(path_str), expected);
    }

    #[test_case("lowercase only", "a1/b-c/d.txt", "A1/B-C/D.TXT")]
    #[test_case("mixed case", "a/B/C", "A/B/C")]
    #[test_case("uppercase only", "AAAAAAAAAAA", "AAAAAAAAAAA")]
    fn test_letter_case_is_canonicalized_correctly(
        _description: &str,
        path_str: &str,
        expected: &str,
    ) {
        assert_eq!(canonicalize_str(path_str), expected);
    }

    #[test]
    fn test_same_dir_remains_same_dir() {
        assert_eq!(canonicalize_str("."), ".");
    }

    #[test_case("same dir repeated", "./././", ".")]
    #[test_case("same dir repeated with a filename", "./././A.TXT", "A.TXT")]
    #[test_case("same dir repeated after dirname", "H/././.", "H")]
    #[test_case("dirname surrounded by same dir", "./A/.", "A")]
    #[test_case("same dir surrounded by dirnames", "A/./B", "A/B")]
    #[test_case("multiple same dirs surrounded by dirnames", "A/./././B", "A/B")]
    #[test_case("mixed dirname/ same dir", "A/./B/./C", "A/B/C")]
    fn test_same_dir_pattern_is_removed_correctly(
        _description: &str,
        path_str: &str,
        expected: &str,
    ) {
        assert_eq!(canonicalize_str(path_str), expected);
    }

    #[test_case("parent dir after a dirname only", "A/..", ".")]
    #[test_case("parent dir after a dirname between other dirnames", "A/B/../C", "A/C")]
    #[test_case("parent dir after a dirname after other dirnames", "A/B/C/..", "A/B")]
    #[test_case("parent dir after a dirname before other dirnames", "A/../B/C", "B/C")]
    #[test_case("multiple parent dirs after multiple dirnames", "A/B/C/../../..", ".")]
    #[test_case(
        "multiple parent dirs after multiple dirnames, before some more dirnames",
        "A/B/C/../../../D/E",
        "D/E"
    )]
    #[test_case(
        "multiple parent dirs after multiple dirnames, after some more dirnames",
        "A/B/C/D/E/../../..",
        "A/B"
    )]
    #[test_case("dirname and then parent dir multiple times", "A/../B/../C/..", ".")]
    #[test_case(
        "dirname and then parent dir multiple times, before some more dirnames",
        "A/../B/../C/../D/E",
        "D/E"
    )]
    #[test_case(
        "dirname and then parent dir multiple times, after some more dirnames",
        "A/B/C/../D/../E/..",
        "A/B"
    )]
    fn test_parent_dir_pattern_is_removed_correctly(
        _description: &str,
        path_str: &str,
        expected: &str,
    ) {
        assert_eq!(canonicalize_str(path_str), expected);
    }

    #[test_case("single parent dir repeated", "..", "..")]
    #[test_case("parent dir repeated", "../../..", "../../..")]
    #[test_case("parent dir before a filename", "../A.TXT", "../A.TXT")]
    #[test_case(
        "multiple parent dirs before a filename",
        "../../../A.TXT",
        "../../../A.TXT"
    )]
    #[test_case(
        "more than one parent dir after a single dirname",
        "A/../../..",
        "../.."
    )]
    #[test_case("parent dir and same dir mixed", "./../../././../..", "../../../..")]
    fn test_same_dir_pattern_is_not_removed_in_impossible_places(
        _description: &str,
        path_str: &str,
        expected: &str,
    ) {
        assert_eq!(canonicalize_str(path_str), expected);
    }

    #[test_case(
        "multiple dirnames and multiple dirnames",
        "a/b/c",
        "d/e/f",
        "A/B/C/D/E/F"
    )]
    #[test_case("same dir and same dir", ".", ".", ".")]
    #[test_case("parent dir and parent dir", "..", "..", "../..")]
    #[test_case("a dirname and parent dir", "a", "..", ".")]
    #[test_case("parent dir and a dirname", "..", "a", "../A")]
    #[test_case("same dir and parent dir", ".", "..", "..")]
    #[test_case("parent dir and same dir", "..", ".", "..")]
    fn test_appending_path_works_correctly(
        _description: &str,
        original_path: &str,
        appended_path: &str,
        expected: &str,
    ) {
        let mut path = Path::from(original_path);
        path.append(appended_path);
        assert_eq!(path.to_string(), expected);
    }

    #[test_case(
        "prepend multiple dirnames with multiple dirnames",
        "a/b/c",
        "d/e/f",
        "D/E/F/A/B/C"
    )]
    #[test_case("prepending same dir with same dir", ".", ".", ".")]
    #[test_case("prepending parent dir with parent dir", "..", "..", "../..")]
    #[test_case("prepending a dirname with parent dir", "a", "..", "../A")]
    #[test_case("prepending parent dir with a dirname", "..", "a", ".")]
    #[test_case("prepending parent dir with same dir", "..", ".", "..")]
    #[test_case("prepending same dir with parent dir", ".", "..", "..")]
    fn test_prepending_path_works_correctly(
        _description: &str,
        original_path: &str,
        prepended_path: &str,
        expected: &str,
    ) {
        let mut path = Path::from(original_path);
        path.prepend(prepended_path);
        assert_eq!(path.to_string(), expected);
    }
}
