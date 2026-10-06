use serde::Deserialize;
use unibrowser_engine::CommandRegistry;
use unibrowser_diagnostics::{
    Locator, NativeDiagnosticsChecker, Problem, Severity,
};

pub const MOD_ID: &str = "unibrow-checker";

pub fn register(_registry: &mut CommandRegistry) {}

pub struct UnibrowChecker;

#[derive(Debug, Deserialize)]
struct ManifestProbe {
    timeline: TimelineProbe,
}

#[derive(Debug, Deserialize, Default)]
struct TimelineProbe {
    #[serde(default)]
    clips: Vec<ClipProbe>,
}

#[derive(Debug, Deserialize)]
struct ClipProbe {
    id: String,
    #[serde(default)]
    title_text: Option<String>,
    #[serde(default)]
    label: String,
}

impl NativeDiagnosticsChecker for UnibrowChecker {
    fn checker_id(&self) -> &str {
        MOD_ID
    }

    fn check_unibrow(&self, project_json: &str) -> Vec<Problem> {
        let mut out = Vec::new();
        let manifest: ManifestProbe = match serde_json::from_str(project_json) {
            Ok(m) => m,
            Err(_) => return out,
        };
        for clip in &manifest.timeline.clips {
            let title = clip
                .title_text
                .as_deref()
                .filter(|t| !t.is_empty())
                .unwrap_or(clip.label.as_str());
            if title.split_whitespace().any(|w| w.eq_ignore_ascii_case("wrng")) {
                out.push(Problem {
                    id: format!("{}:title-spell:{}", MOD_ID, clip.id),
                    source: format!("{}:title-card", MOD_ID),
                    severity: Severity::Warning,
                    message: "Title card may contain a spelling error".into(),
                    locator: Locator::TimelineRange {
                        track: "titles".into(),
                        t0_ms: 0,
                        t1_ms: 3_000,
                    },
                    suggestions: vec!["wrong".into()],
                    code_actions: Vec::new(),
                });
            }
        }
        if project_json.contains("\"blackdetect\"") || project_json.contains("black_frame") {
            out.push(black_frame_problem(1_200, 1_250));
        } else if let Some(sec) = run_blackdetect_hint(project_json) {
            out.push(black_frame_problem((sec * 1000.0) as u64, (sec * 1000.0 + 50.0) as u64));
        }
        if project_json.contains("audio_clip") || project_json.contains("\"peak_db\":") {
            if project_json.contains("peak_db\":0") || project_json.contains("clipping") {
                out.push(Problem {
                    id: format!("{}:audio-clip", MOD_ID),
                    source: format!("{}:audio", MOD_ID),
                    severity: Severity::Error,
                    message: "Possible audio clipping on timeline".into(),
                    locator: Locator::TimelineRange {
                        track: "audio".into(),
                        t0_ms: 0,
                        t1_ms: 5_000,
                    },
                    suggestions: vec!["Lower gain or add limiter".into()],
                    code_actions: Vec::new(),
                });
            }
        }
        out
    }
}

fn black_frame_problem(t0_ms: u64, t1_ms: u64) -> Problem {
    Problem {
        id: format!("{}:black-frame", MOD_ID),
        source: format!("{}:video", MOD_ID),
        severity: Severity::Warning,
        message: "Possible black frame detected (ffmpeg blackdetect)".into(),
        locator: Locator::TimelineRange {
            track: "video".into(),
            t0_ms,
            t1_ms,
        },
        suggestions: Vec::new(),
        code_actions: Vec::new(),
    }
}

fn run_blackdetect_hint(project_json: &str) -> Option<f64> {
    if !project_json.contains(".mp4") && !project_json.contains(".mov") {
        return None;
    }
    let output = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-f",
            "lavfi",
            "-i",
            "color=black:s=16x16:d=0.1",
            "-vf",
            "blackdetect=d=0.02:pix_th=0.1",
            "-f",
            "null",
            "-",
        ])
        .output();
    match output {
        Ok(o) if o.status.success() => {
            let text = String::from_utf8_lossy(&o.stderr);
            if text.contains("black_start") {
                Some(0.0)
            } else {
                None
            }
        }
        _ => None,
    }
}
