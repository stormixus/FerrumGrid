//! Update check against the GitHub Releases API and macOS auto-update installation.
//!
//! Hits `https://api.github.com/repos/{owner}/{repo}/releases/latest` and
//! compares the returned `tag_name` against the running build's
//! `CARGO_PKG_VERSION`. The check and installation are fired off-thread;
//! results land back on the UI via a `Receiver`.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RELEASES_LATEST_URL: &str =
    "https://api.github.com/repos/stormixus/FerrumGrid/releases/latest";
const HTTP_TIMEOUT: Duration = Duration::from_secs(8);
const USER_AGENT: &str = concat!("FerrumGrid/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    UpToDate {
        current: String,
    },
    UpdateAvailable {
        current: String,
        latest: String,
        dmg_url: Option<String>,
        url: String,
    },
    Downloading {
        latest: String,
    },
    Installing {
        latest: String,
    },
    Error(String),
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
        if matches!(self.status, UpdateStatus::Checking | UpdateStatus::Downloading { .. } | UpdateStatus::Installing { .. }) {
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

    /// Start the automatic update process in the background.
    pub fn start_update(&mut self, dmg_url: String, latest: String, ctx: eframe::egui::Context) {
        if matches!(self.status, UpdateStatus::Downloading { .. } | UpdateStatus::Installing { .. }) {
            return;
        }
        self.status = UpdateStatus::Downloading { latest: latest.clone() };
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        thread::spawn(move || {
            let result = download_and_install_dmg(tx.clone(), &dmg_url, &latest, ctx);
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
                // If it is a final state, clear the channel receiver
                if matches!(self.status, UpdateStatus::Idle | UpdateStatus::UpToDate { .. } | UpdateStatus::UpdateAvailable { .. } | UpdateStatus::Error(_)) {
                    self.rx = None;
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                if matches!(self.status, UpdateStatus::Checking | UpdateStatus::Downloading { .. } | UpdateStatus::Installing { .. })
                    && !matches!(self.status, UpdateStatus::Error(_))
                {
                    self.status = UpdateStatus::Error("update thread disconnected".to_string());
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
        // Detect current architecture for macOS auto-updates
        let target = if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        };

        let mut dmg_url = None;
        if let Some(assets) = json.get("assets").and_then(|a| a.as_array()) {
            for asset in assets {
                if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                    if name.contains(target) && name.ends_with(".dmg") {
                        if let Some(dl_url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                            dmg_url = Some(dl_url.to_string());
                            break;
                        }
                    }
                }
            }
        }

        UpdateStatus::UpdateAvailable {
            current,
            latest: latest_str,
            dmg_url,
            url,
        }
    } else {
        UpdateStatus::UpToDate { current }
    }
}

/// Download and install the DMG background task.
fn download_and_install_dmg(
    tx: mpsc::Sender<UpdateStatus>,
    dmg_url: &str,
    latest: &str,
    ctx: eframe::egui::Context,
) -> UpdateStatus {
    // 1. Download the DMG file
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(120))
        .user_agent(USER_AGENT)
        .build();

    let response = match agent.get(dmg_url).call() {
        Ok(r) => r,
        Err(err) => return UpdateStatus::Error(format!("download: {err}")),
    };

    let temp_dir = std::env::temp_dir();
    let dmg_path = temp_dir.join(format!("ferrumgrid-update-{}.dmg", latest));

    let mut file = match std::fs::File::create(&dmg_path) {
        Ok(f) => f,
        Err(err) => return UpdateStatus::Error(format!("create temp file: {err}")),
    };

    if let Err(err) = std::io::copy(&mut response.into_reader(), &mut file) {
        let _ = std::fs::remove_file(&dmg_path);
        return UpdateStatus::Error(format!("download write: {err}"));
    }

    // 2. We are now mounting/installing. Transition state!
    let _ = tx.send(UpdateStatus::Installing { latest: latest.to_string() });
    ctx.request_repaint();

    // Run mount command
    let dmg_path_str = dmg_path.to_string_lossy();
    let mount_output = match std::process::Command::new("hdiutil")
        .args(["attach", "-nobrowse", "-readonly", &dmg_path_str])
        .output()
    {
        Ok(out) => out,
        Err(err) => return UpdateStatus::Error(format!("hdiutil launch error: {err}")),
    };

    if !mount_output.status.success() {
        let err_msg = String::from_utf8_lossy(&mount_output.stderr).to_string();
        let _ = std::fs::remove_file(&dmg_path);
        return UpdateStatus::Error(format!("mount failed: {err_msg}"));
    }

    // Parse mount point from hdiutil output
    let stdout = String::from_utf8_lossy(&mount_output.stdout);
    let mut mount_point = None;
    for line in stdout.lines() {
        if line.contains("/Volumes/") {
            if let Some(idx) = line.find("/Volumes/") {
                mount_point = Some(line[idx..].trim().to_string());
                break;
            }
        }
    }

    let Some(mount_path) = mount_point else {
        let _ = std::fs::remove_file(&dmg_path);
        return UpdateStatus::Error("mount point unresolved".to_string());
    };

    let source_app = std::path::Path::new(&mount_path).join("FerrumGrid.app");
    if !source_app.exists() {
        let _ = std::process::Command::new("hdiutil")
            .args(["detach", &mount_path])
            .output();
        let _ = std::fs::remove_file(&dmg_path);
        return UpdateStatus::Error("App bundle not found in mounted volume".to_string());
    }

    // Copy to a temporary location before detaching
    let temp_app_dir = temp_dir.join("ferrumgrid_new_app");
    let _ = std::fs::remove_dir_all(&temp_app_dir);
    if let Err(err) = std::fs::create_dir_all(&temp_app_dir) {
        let _ = std::process::Command::new("hdiutil")
            .args(["detach", &mount_path])
            .output();
        let _ = std::fs::remove_file(&dmg_path);
        return UpdateStatus::Error(format!("failed to create extract path: {err}"));
    }

    let temp_app_path = temp_app_dir.join("FerrumGrid.app");
    let cp_output = std::process::Command::new("cp")
        .args(["-R", source_app.to_str().unwrap(), temp_app_path.to_str().unwrap()])
        .output();

    // Clean up DMG mount
    let _ = std::process::Command::new("hdiutil")
        .args(["detach", &mount_path])
        .output();
    let _ = std::fs::remove_file(&dmg_path);

    match cp_output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let err_msg = String::from_utf8_lossy(&out.stderr).to_string();
            return UpdateStatus::Error(format!("cp failed: {err_msg}"));
        }
        Err(err) => return UpdateStatus::Error(format!("cp execution failed: {err}")),
    }

    // 3. Resolve currently running executable and verify bundle
    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(err) => return UpdateStatus::Error(format!("resolve current executable: {err}")),
    };

    let old_app_path = match current_exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
    {
        Some(path) => path,
        None => return UpdateStatus::Error("current exe parent resolution failed".to_string()),
    };

    // Ensure we are actually running inside a .app bundle (safeguard for dev builds!)
    if old_app_path.extension().and_then(|e| e.to_str()) != Some("app") {
        return UpdateStatus::Error("Not running as a macOS .app bundle. Automatic update disabled in dev mode.".to_string());
    }

    let parent_pid = std::process::id();
    let old_app_str = old_app_path.to_string_lossy().into_owned();
    let temp_app_str = temp_app_path.to_string_lossy().into_owned();

    // Relaunch shell script daemon that waits for us to quit, replaces the bundle, and restarts
    let script = format!(
        "while kill -0 {parent_pid} 2>/dev/null; do sleep 0.1; done; \
         rm -rf \"{old_app_str}\" && \
         mv \"{temp_app_str}\" \"{old_app_str}\" && \
         open \"{old_app_str}\"",
        parent_pid = parent_pid,
        old_app_str = old_app_str,
        temp_app_str = temp_app_str
    );

    if let Err(err) = std::process::Command::new("sh")
        .args(["-c", &script])
        .spawn()
    {
        return UpdateStatus::Error(format!("updater daemon launch failed: {err}"));
    }

    // Gracefully exit immediately so the daemon can finish the job
    std::process::exit(0);
}
