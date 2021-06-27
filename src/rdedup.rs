use rdedup_lib::{settings::Repo as RepoSettings, Repo};
use slog::Logger;
use std::path::Path;
use url::Url;

pub fn init(
    path: &Path,
    settings: RepoSettings,
    passphrase: String,
    log: Logger,
) -> Result<Repo, String> {
    let url = Url::from_file_path(path).map_err(|()| "RDEDUP_DIR url from path".to_string())?;
    Repo::init(&url, &move || Ok(passphrase.clone()), settings, log)
        .map_err(|_| "Initialing Rdedup Repo".to_string())
}
