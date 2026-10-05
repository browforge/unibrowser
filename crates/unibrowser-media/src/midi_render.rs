//! MIDI → audio render lane probes (fluidsynth + ffmpeg). Execution remains a stub.

use serde::Serialize;

use crate::{find_binary, FfmpegProbe, probe_ffmpeg_install};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MidiAudioRenderProbe {
    pub available: bool,
    pub ffmpeg: bool,
    pub fluidsynth: bool,
    pub hint: String,
}

pub fn probe_midi_audio_render() -> MidiAudioRenderProbe {
    let ffmpeg: FfmpegProbe = probe_ffmpeg_install();
    let fluidsynth = find_binary("fluidsynth").is_some();
    let available = ffmpeg.available && fluidsynth;
    let hint = if available {
        "Render to WAV is not wired yet; fluidsynth and ffmpeg are on PATH.".into()
    } else if !ffmpeg.available && !fluidsynth {
        "Install ffmpeg and fluidsynth on PATH to enable the render stub.".into()
    } else if !ffmpeg.available {
        "Install ffmpeg on PATH (fluidsynth is present).".into()
    } else {
        "Install fluidsynth on PATH (ffmpeg is present).".into()
    };
    MidiAudioRenderProbe {
        available,
        ffmpeg: ffmpeg.available,
        fluidsynth,
        hint,
    }
}
