//! DiagnosticsPanel — 5-channel runtime diagnostics surface.
//!
//! Plan v7 Phase 0 scaffold → Phase 4b4 5-channel 통합. 각 채널 (echo_timeout /
//! dangling_tx / cache_stale / backup_error / mutation_diagnostic) 의 push API +
//! ring buffer (capacity 100) + 채널별 filter + render.

use std::collections::VecDeque;
use std::time::SystemTime;

use eframe::egui;

use super::theme;

/// 진단 항목의 심각도.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagSeverity {
    Info,
    Warn,
    Error,
}

/// 진단 항목이 발생한 채널 (5 source).
///
/// Plan v7 §10:
/// - `EchoTimeout` — Phase 1.3 invalidate echo 가 timeout 내 도착 안 함
/// - `DanglingTx` — Phase 3 명시 BEGIN 이 30s/60s 임계 초과
/// - `CacheStale` — Phase 1.3 cache invalidation 누락 의심
/// - `BackupError` — Phase 4a BackupStatus::Failed
/// - `MutationDiagnostic` — Phase 1.1 apply_data_edits 실패 / 부분 성공
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagChannel {
    EchoTimeout,
    DanglingTx,
    CacheStale,
    BackupError,
    MutationDiagnostic,
}

impl DiagChannel {
    pub fn label(self) -> &'static str {
        match self {
            DiagChannel::EchoTimeout => "echo_timeout",
            DiagChannel::DanglingTx => "dangling_tx",
            DiagChannel::CacheStale => "cache_stale",
            DiagChannel::BackupError => "backup_error",
            DiagChannel::MutationDiagnostic => "mutation_diagnostic",
        }
    }
}

/// 단일 진단 항목.
#[derive(Debug, Clone)]
pub struct DiagEntry {
    pub timestamp: SystemTime,
    pub channel: DiagChannel,
    pub severity: DiagSeverity,
    pub message: String,
}

/// Ring buffer 용량.
const RING_CAPACITY: usize = 100;

/// 진단 패널 상태.
#[derive(Debug)]
pub struct DiagnosticsPanel {
    pub visible: bool,
    /// Plan v7 Phase 1.3 — `ferrumgrid.unsafe_ctid` settings flag 가 ON 일 때
    /// 영구 배너 표시. settings 변경 시 app.rs 가 setter 로 동기화한다.
    pub unsafe_ctid_active: bool,
    entries: VecDeque<DiagEntry>,
    filter: Option<DiagChannel>,
}

const ALL_CHANNELS: [DiagChannel; 5] = [
    DiagChannel::EchoTimeout,
    DiagChannel::DanglingTx,
    DiagChannel::CacheStale,
    DiagChannel::BackupError,
    DiagChannel::MutationDiagnostic,
];

impl Default for DiagnosticsPanel {
    fn default() -> Self {
        Self {
            visible: false,
            unsafe_ctid_active: false,
            entries: VecDeque::with_capacity(RING_CAPACITY),
            filter: None,
        }
    }
}

impl DiagnosticsPanel {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// 신규 진단 항목 push. Ring buffer 가 차면 가장 오래된 항목을 제거.
    fn push(&mut self, channel: DiagChannel, severity: DiagSeverity, message: impl Into<String>) {
        if self.entries.len() == RING_CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(DiagEntry {
            timestamp: SystemTime::now(),
            channel,
            severity,
            message: message.into(),
        });
    }

    pub fn push_echo_timeout(&mut self, message: impl Into<String>) {
        self.push(DiagChannel::EchoTimeout, DiagSeverity::Warn, message);
    }

    pub fn push_dangling_tx(&mut self, severity: DiagSeverity, message: impl Into<String>) {
        self.push(DiagChannel::DanglingTx, severity, message);
    }

    pub fn push_cache_stale(&mut self, message: impl Into<String>) {
        self.push(DiagChannel::CacheStale, DiagSeverity::Info, message);
    }

    pub fn push_backup_error(&mut self, message: impl Into<String>) {
        self.push(DiagChannel::BackupError, DiagSeverity::Error, message);
    }

    pub fn push_mutation_diagnostic(&mut self, severity: DiagSeverity, message: impl Into<String>) {
        self.push(DiagChannel::MutationDiagnostic, severity, message);
    }

    pub fn entries(&self) -> impl Iterator<Item = &DiagEntry> {
        self.entries.iter()
    }

    #[allow(dead_code)]
    pub fn entries_for(&self, channel: DiagChannel) -> impl Iterator<Item = &DiagEntry> {
        self.entries.iter().filter(move |e| e.channel == channel)
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn render(&mut self, ui: &mut egui::Ui) {
        if self.unsafe_ctid_active {
            ui.colored_label(
                theme::ACCENT_RED,
                "⚠ unsafe_ctid 모드 활성 — PK 부재 테이블 편집 시 \
                 VACUUM FULL 또는 동시 INSERT 가 데이터 손상을 유발할 수 있습니다.",
            );
        }
        if !self.visible {
            return;
        }

        let mut should_clear = false;
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("Diagnostics")
                    .strong()
                    .size(12.0)
                    .color(theme::text_primary()),
            );
            let count = self.entries.len();
            if count > 0 {
                ui.label(
                    egui::RichText::new(format!("({count})"))
                        .size(10.0)
                        .color(theme::text_muted()),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Clear").clicked() {
                    should_clear = true;
                }
            });
        });
        if should_clear {
            self.clear();
        }

        ui.separator();

        // Channel filter tabs
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let all_selected = self.filter.is_none();
            if ui
                .add(channel_filter_button("All", all_selected))
                .clicked()
            {
                self.filter = None;
            }
            for ch in ALL_CHANNELS {
                let selected = self.filter == Some(ch);
                if ui
                    .add(channel_filter_button(ch.label(), selected))
                    .clicked()
                {
                    self.filter = Some(ch);
                }
            }
        });

        ui.add_space(2.0);

        let visible_entries: Vec<&DiagEntry> = self
            .entries
            .iter()
            .filter(|e| self.filter.is_none() || self.filter == Some(e.channel))
            .collect();

        if visible_entries.is_empty() {
            ui.label(
                egui::RichText::new("No diagnostics entries")
                    .color(theme::text_disabled())
                    .size(11.0),
            );
            return;
        }

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for entry in &visible_entries {
                    let (icon, color) = severity_icon_color(entry.severity);
                    let time = format_timestamp(entry.timestamp);
                    ui.colored_label(
                        color,
                        format!(
                            "{time}  {icon} [{}] {}",
                            entry.channel.label(),
                            entry.message,
                        ),
                    );
                }
            });
    }
}

fn channel_filter_button(label: &str, selected: bool) -> egui::Button<'_> {
    let text = egui::RichText::new(label)
        .size(10.0)
        .monospace()
        .color(if selected {
            theme::text_primary()
        } else {
            theme::text_muted()
        });
    egui::Button::new(text)
        .fill(if selected {
            theme::with_alpha(theme::ACCENT_BLUE, 30)
        } else {
            egui::Color32::TRANSPARENT
        })
        .stroke(if selected {
            egui::Stroke::new(1.0, theme::with_alpha(theme::ACCENT_BLUE, 120))
        } else {
            egui::Stroke::NONE
        })
        .corner_radius(egui::CornerRadius::same(theme::RADIUS_SM))
}

fn severity_icon_color(severity: DiagSeverity) -> (&'static str, egui::Color32) {
    match severity {
        DiagSeverity::Info => ("i", theme::ACCENT_BLUE),
        DiagSeverity::Warn => ("!", theme::ACCENT_YELLOW),
        DiagSeverity::Error => ("x", theme::ACCENT_RED),
    }
}

fn format_timestamp(t: SystemTime) -> String {
    let dt: chrono::DateTime<chrono::Local> = t.into();
    dt.format("%H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panel_is_empty_and_hidden() {
        let panel = DiagnosticsPanel::new();
        assert!(!panel.visible);
        assert!(!panel.unsafe_ctid_active);
        assert_eq!(panel.entry_count(), 0);
    }

    #[test]
    fn push_echo_timeout_adds_warn_entry() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_echo_timeout("invalidate echo 5s timeout");
        assert_eq!(panel.entry_count(), 1);
        let entry = panel.entries().next().unwrap();
        assert_eq!(entry.channel, DiagChannel::EchoTimeout);
        assert_eq!(entry.severity, DiagSeverity::Warn);
        assert_eq!(entry.message, "invalidate echo 5s timeout");
    }

    #[test]
    fn push_dangling_tx_uses_caller_severity() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_dangling_tx(DiagSeverity::Error, "60s ROLLBACK fired");
        let entry = panel.entries().next().unwrap();
        assert_eq!(entry.channel, DiagChannel::DanglingTx);
        assert_eq!(entry.severity, DiagSeverity::Error);
    }

    #[test]
    fn push_cache_stale_uses_info_severity() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_cache_stale("table_oid 16384 invalidated");
        let entry = panel.entries().next().unwrap();
        assert_eq!(entry.channel, DiagChannel::CacheStale);
        assert_eq!(entry.severity, DiagSeverity::Info);
    }

    #[test]
    fn push_backup_error_uses_error_severity() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_backup_error("pg_dump exit 1");
        let entry = panel.entries().next().unwrap();
        assert_eq!(entry.channel, DiagChannel::BackupError);
        assert_eq!(entry.severity, DiagSeverity::Error);
    }

    #[test]
    fn push_mutation_diagnostic_uses_caller_severity() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_mutation_diagnostic(DiagSeverity::Warn, "partial INSERT (3/5 rows)");
        let entry = panel.entries().next().unwrap();
        assert_eq!(entry.channel, DiagChannel::MutationDiagnostic);
        assert_eq!(entry.severity, DiagSeverity::Warn);
    }

    #[test]
    fn entries_for_filters_by_channel() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_echo_timeout("a");
        panel.push_dangling_tx(DiagSeverity::Warn, "b");
        panel.push_echo_timeout("c");
        panel.push_cache_stale("d");
        let echo: Vec<_> = panel.entries_for(DiagChannel::EchoTimeout).collect();
        assert_eq!(echo.len(), 2);
        assert_eq!(echo[0].message, "a");
        assert_eq!(echo[1].message, "c");
        let cache: Vec<_> = panel.entries_for(DiagChannel::CacheStale).collect();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn ring_buffer_drops_oldest_when_full() {
        let mut panel = DiagnosticsPanel::new();
        for i in 0..(RING_CAPACITY + 5) {
            panel.push_cache_stale(format!("entry {i}"));
        }
        assert_eq!(panel.entry_count(), RING_CAPACITY);
        // 가장 오래된 5개 (entry 0..4) 가 drop 됨
        let first = panel.entries().next().unwrap();
        assert_eq!(first.message, "entry 5");
    }

    #[test]
    fn clear_removes_all_entries() {
        let mut panel = DiagnosticsPanel::new();
        panel.push_echo_timeout("x");
        panel.push_backup_error("y");
        assert_eq!(panel.entry_count(), 2);
        panel.clear();
        assert_eq!(panel.entry_count(), 0);
    }

    #[test]
    fn channel_labels_are_stable_strings() {
        assert_eq!(DiagChannel::EchoTimeout.label(), "echo_timeout");
        assert_eq!(DiagChannel::DanglingTx.label(), "dangling_tx");
        assert_eq!(DiagChannel::CacheStale.label(), "cache_stale");
        assert_eq!(DiagChannel::BackupError.label(), "backup_error");
        assert_eq!(DiagChannel::MutationDiagnostic.label(), "mutation_diagnostic");
    }

    #[test]
    fn unsafe_ctid_field_independent_of_entries() {
        let mut panel = DiagnosticsPanel::new();
        panel.unsafe_ctid_active = true;
        assert!(panel.unsafe_ctid_active);
        assert_eq!(panel.entry_count(), 0);
    }
}
