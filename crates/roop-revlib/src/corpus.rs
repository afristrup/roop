use crate::{Error, Pla, Syrec, parse, parse_syrec};
use std::path::Path;

pub fn load(directory: &Path) -> Result<Vec<Pla>, Error> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "pla"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            Ok(parse(
                &path.display().to_string(),
                &std::fs::read_to_string(path)?,
            )?)
        })
        .collect()
}

pub fn load_syrec(directory: &Path) -> Result<Vec<Syrec>, Error> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "src"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            Ok(parse_syrec(
                &path.display().to_string(),
                &std::fs::read_to_string(path)?,
            )?)
        })
        .collect()
}
