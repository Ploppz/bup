use anyhow::Context;
use rdedup_lib::{settings::Repo as RepoSettings, Repo};
use slog::Logger;
use std::path::Path;
use url::Url;

pub fn init(
    path: &Path,
    settings: RepoSettings,
    passphrase: String,
    log: Logger,
) -> anyhow::Result<Repo> {
    let url = Url::from_directory_path(path)
        .ok()
        .context("RDEDUP_DIR url from path")?;
    Repo::init(&url, &move || Ok(passphrase.clone()), settings, log)
        .context("Initialing Rdedup Repo")
}
