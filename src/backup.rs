use url::Url;
use rdedup_lib::{
    Repo,
    settings::Repo as RepoSettings
};
use slog::Logger;

pub fn rdedup_init(url: Url, settings: RepoSettings, passphrase: String, log: Logger) -> std::io::Result<Repo> {
    Repo::init(
        &url,
        &move || Ok(passphrase.clone()),
        settings,
        log,
    )
}
