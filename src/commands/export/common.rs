use std::path::PathBuf;

#[derive(PartialEq)]
pub(in crate::commands::export) enum Status {
    Before,
    Header,
    Repository,
    Group,
    Relation,
    Footer,
}

pub(in crate::commands::export) fn map_path(p: PathBuf, mapper: fn(String) -> String) -> String {
    p.to_str()
        .map(|s| s.to_string())
        .map(mapper)
        .unwrap_or(String::from(""))
}
