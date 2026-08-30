use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::PathBuf;
use zvariant::{OwnedValue, Value};

pub type Options = HashMap<String, OwnedValue>;

pub fn as_bool(opts: &Options, key: &str) -> Option<bool> {
    opts.get(key).and_then(from_value)
}

pub fn bool_or(opts: &Options, key: &str, default: bool) -> bool {
    as_bool(opts, key).unwrap_or(default)
}

pub fn as_str(opts: &Options, key: &str) -> Option<String> {
    opts.get(key).and_then(from_value)
}

pub fn as_u32(opts: &Options, key: &str) -> Option<u32> {
    opts.get(key).and_then(from_value)
}

pub fn as_str_vec(opts: &Options, key: &str) -> Option<Vec<String>> {
    opts.get(key).and_then(from_value)
}

pub fn as_bytes(opts: &Options, key: &str) -> Option<Vec<u8>> {
    opts.get(key).and_then(from_value)
}

pub fn as_path(opts: &Options, key: &str) -> Option<PathBuf> {
    as_bytes(opts, key).map(path_from_ay)
}

pub fn as_filters(opts: &Options, key: &str) -> Vec<(String, Vec<(u32, String)>)> {
    opts.get(key).and_then(from_value).unwrap_or_default()
}

pub fn as_filter(opts: &Options, key: &str) -> Option<(String, Vec<(u32, String)>)> {
    opts.get(key).and_then(from_value)
}

pub fn as_choices(opts: &Options, key: &str) -> Vec<Choice> {
    let raw: Vec<(String, String, Vec<(String, String)>, String)> =
        opts.get(key).and_then(from_value).unwrap_or_default();
    raw.into_iter()
        .map(|(id, label, options, selected)| Choice {
            id,
            label,
            options,
            selected,
        })
        .collect()
}

pub fn as_files(opts: &Options, key: &str) -> Vec<String> {
    let arrays: Vec<Vec<u8>> = opts.get(key).and_then(from_value).unwrap_or_default();
    arrays
        .into_iter()
        .map(|bytes| {
            path_from_ay(bytes)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
    pub options: Vec<(String, String)>,
    pub selected: String,
}

pub fn path_from_ay(mut bytes: Vec<u8>) -> PathBuf {
    while bytes.last() == Some(&0) {
        bytes.pop();
    }
    PathBuf::from(OsString::from_vec(bytes))
}

fn from_value<T>(v: &OwnedValue) -> Option<T>
where
    T: TryFrom<Value<'static>>,
{
    let value: Value<'static> = v.try_clone().ok()?.into();
    T::try_from(value).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_nul_from_ay_paths() {
        let path = path_from_ay(b"/tmp/foo\0".to_vec());
        assert_eq!(path, PathBuf::from("/tmp/foo"));
    }
}
