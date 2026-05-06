//! Update check against the GitHub Releases API.
//!
//! Hits `https://api.github.com/repos/{owner}/{repo}/releases/latest` and
//! compares the returned `tag_name` against the running build's
//! `CARGO_PKG_VERSION`. The check is fired off-thread; results land back on
//! the UI via a `Receiver`.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/stormixus/FerrumGrid/releases/latest";
const HTTP_TIMEOUT: Duration = Duration::from_secs(8);
const USER_AGENT: &str = concat!("FerrumGrid/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate {
        current: String,
    },
    UpdateAvailable {
        current: String,
        latest: String,
        url: String,
    },
    Error(String),
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug)]
pub struct UpdateCheck {
    pub status: UpdateStatus,
    rx: Option<mpsc::Receiver<UpdateStatus>>,
}

impl Default for UpdateCheck {
    fn default() -> Self {
        Self {
            status: UpdateStatus::Idle,
            rx: None,
        }
    }
}

impl UpdateCheck {
    /// Fire off a check on a background thread. Subsequent calls while a check
    /// is in flight are no-ops.
    pub fn start(&mut self) {
        if matches!(self.status, UpdateStatus::Checking) {
            return;
        }
        self.status = UpdateStatus::Checking;
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        thread::spawn(move || {
            let result = fetch_latest_status();
            let _ = tx.send(result);
        });
    }

    /// Drain any pending result from the worker thread. Call once per UI frame.
    pub fn poll(&mut self) {
        let Some(rx) = self.rx.as_ref() else {
            return;
        };
        match rx.try_recv() {
            Ok(status) => {
                self.status = status;
                self.rx = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                if matches!(self.status, UpdateStatus::Checking) {
                    self.status = UpdateStatus::Error("worker dropped".to_string());
                }
                self.rx = None;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
}

fn fetch_latest_status() -> UpdateStatus {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let agent = ureq::AgentBuilder::new()
        .timeout(HTTP_TIMEOUT)
        .user_agent(USER_AGENT)
        .build();

    let response = match agent
        .get(RELEASES_LATEST_URL)
        .set("Accept", "application/vnd.github+json")
        .call()
    {
        Ok(r) => r,
        Err(ureq::Error::Status(code, _)) => {
            return UpdateStatus::Error(format!("GitHub API HTTP {code}"));
        }
        Err(err) => {
            return UpdateStatus::Error(format!("network: {err}"));
        }
    };

    let json: serde_json::Value = match response.into_json() {
        Ok(v) => v,
        Err(err) => return UpdateStatus::Error(format!("parse: {err}")),
    };

    let tag = json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if tag.is_empty() {
        return UpdateStatus::Error("missing tag_name".to_string());
    }
    let latest_str = tag.trim_start_matches('v').to_string();
    let url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/stormixus/FerrumGrid/releases/latest")
        .to_string();

    let cur_v = match semver::Version::parse(&current) {
        Ok(v) => v,
        Err(err) => return UpdateStatus::Error(format!("bad current version: {err}")),
    };
    let lat_v = match semver::Version::parse(&latest_str) {
        Ok(v) => v,
        Err(err) => return UpdateStatus::Error(format!("bad latest version: {err}")),
    };

    if lat_v > cur_v {
        UpdateStatus::UpdateAvailable {
            current,
            latest: latest_str,
            url,
        }
    } else {
        UpdateStatus::UpToDate { current }
    }
}
