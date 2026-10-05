//! Standard MIDI File (SMF) header parse — proving stub for native-until-encode lane.

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SmfSummary {
    pub format: u16,
    pub track_count: u16,
    pub ticks_per_quarter: u16,
}

pub fn read_smf_summary(path: &Path) -> Result<SmfSummary, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    parse_smf_header(&data)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NoteOnEvent {
    pub track_index: u16,
    pub tick: u32,
    pub note: u8,
    pub velocity: u8,
}

fn read_vlq(data: &[u8], pos: &mut usize) -> Option<u32> {
    let mut value = 0u32;
    for _ in 0..4 {
        if *pos >= data.len() {
            return None;
        }
        let b = data[*pos];
        *pos += 1;
        value = (value << 7) | (b & 0x7f) as u32;
        if b & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

pub fn parse_note_on_events(data: &[u8], max_events: usize) -> Result<Vec<NoteOnEvent>, String> {
    let _header = parse_smf_header(data)?;
    let mut events = Vec::new();
    let mut offset = 14usize;
    let mut track_index = 0u16;
    while offset + 8 <= data.len() && events.len() < max_events {
        let kind = &data[offset..offset + 4];
        let len = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;
        offset += 8;
        if offset + len > data.len() {
            break;
        }
        if kind == b"MTrk" {
            let track = &data[offset..offset + len];
            let mut pos = 0usize;
            let mut tick: u32 = 0;
            while pos < track.len() && events.len() < max_events {
                let delta = read_vlq(track, &mut pos).unwrap_or(0);
                tick += delta;
                if pos >= track.len() {
                    break;
                }
                let status = track[pos];
                if status == 0xff || status == 0xf0 || status == 0xf7 {
                    pos += 1;
                    if pos >= track.len() {
                        break;
                    }
                    let meta_len = track[pos] as usize;
                    pos += 1 + meta_len;
                    continue;
                }
                if status & 0x80 == 0 {
                    break;
                }
                pos += 1;
                let cmd = status & 0xf0;
                let params = if cmd == 0xc0 || cmd == 0xd0 { 1 } else { 2 };
                if pos + params > track.len() {
                    break;
                }
                let p1 = track[pos];
                let p2 = if params == 2 { track[pos + 1] } else { 0 };
                pos += params;
                if cmd == 0x90 && p2 > 0 {
                    events.push(NoteOnEvent {
                        track_index,
                        tick,
                        note: p1,
                        velocity: p2,
                    });
                }
            }
            track_index += 1;
        }
        offset += len;
    }
    Ok(events)
}

pub fn read_note_on_events(path: &Path, max_events: usize) -> Result<Vec<NoteOnEvent>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    parse_note_on_events(&data, max_events)
}

/// Map scrub time within a clip to SMF ticks (120 BPM default) and return nearby note-ons.
pub fn note_on_events_near_time(
    path: &Path,
    offset_sec: f64,
    clip_duration_sec: f64,
    window_ticks: u32,
) -> Result<Vec<NoteOnEvent>, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let header = parse_smf_header(&data)?;
    let events = parse_note_on_events(&data, 4096)?;
    if events.is_empty() {
        return Ok(events);
    }
    let max_tick = events.iter().map(|e| e.tick).max().unwrap_or(1);
    let dur = clip_duration_sec.max(0.001) as f64;
    let ratio = (offset_sec.max(0.0).min(dur)) / dur;
    let center = (ratio * max_tick as f64).round() as u32;
    let bpm = 120.0;
    let tpq = header.ticks_per_quarter.max(1) as f64;
    let tick_from_time = (offset_sec * bpm / 60.0 * tpq).round() as u32;
    let center_tick = if max_tick > 0 {
        (center + tick_from_time) / 2
    } else {
        tick_from_time
    };
    Ok(events
        .into_iter()
        .filter(|e| {
            let d = if e.tick >= center_tick {
                e.tick - center_tick
            } else {
                center_tick - e.tick
            };
            d <= window_ticks
        })
        .take(24)
        .collect())
}

pub fn parse_smf_header(data: &[u8]) -> Result<SmfSummary, String> {
    if data.len() < 14 {
        return Err("file too short for SMF header".into());
    }
    if &data[0..4] != b"MThd" {
        return Err("missing MThd chunk".into());
    }
    let chunk_len = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if chunk_len != 6 {
        return Err(format!("unexpected MThd length {}", chunk_len));
    }
    let format = u16::from_be_bytes([data[8], data[9]]);
    let track_count = u16::from_be_bytes([data[10], data[11]]);
    let division = u16::from_be_bytes([data[12], data[13]]);
    if format > 2 {
        return Err(format!("unsupported SMF format type {}", format));
    }
    if division & 0x8000 != 0 {
        return Err("SMPTE division not supported in stub".into());
    }
    Ok(SmfSummary {
        format,
        track_count,
        ticks_per_quarter: division,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_smf() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"MThd");
        v.extend_from_slice(&6u32.to_be_bytes());
        v.extend_from_slice(&1u16.to_be_bytes()); // format 1
        v.extend_from_slice(&2u16.to_be_bytes()); // 2 tracks
        v.extend_from_slice(&480u16.to_be_bytes());
        v.extend_from_slice(b"MTrk");
        v.extend_from_slice(&0u32.to_be_bytes());
        v
    }

    #[test]
    fn parses_header() {
        let s = parse_smf_header(&minimal_smf()).expect("parse");
        assert_eq!(s.format, 1);
        assert_eq!(s.track_count, 2);
        assert_eq!(s.ticks_per_quarter, 480);
    }

    #[test]
    fn notes_near_time_filters() {
        let mut v = minimal_smf();
        v.truncate(14 + 8);
        v.extend_from_slice(b"MTrk");
        let track: Vec<u8> = vec![
            0x00,
            0x90,
            60,
            100,
            0x00,
            0x80,
            60,
            0,
        ];
        v.extend_from_slice(&(track.len() as u32).to_be_bytes());
        v.extend(track);
        let dir = std::env::temp_dir().join("unibrowser-smf-near.mid");
        std::fs::write(&dir, &v).expect("write");
        let near = note_on_events_near_time(&dir, 0.0, 5.0, 480).expect("near");
        assert_eq!(near.len(), 1);
        let _ = std::fs::remove_file(&dir);
    }

    #[test]
    fn parses_note_on_in_track() {
        let mut v = minimal_smf();
        // Replace empty MTrk with one note-on at tick 0
        v.truncate(14 + 8);
        v.extend_from_slice(b"MTrk");
        let track: Vec<u8> = vec![
            0x00, // delta 0
            0x90,
            60,
            100, // note on
            0x00,
            0x80,
            60,
            0, // note off
        ];
        v.extend_from_slice(&(track.len() as u32).to_be_bytes());
        v.extend(track);
        let events = parse_note_on_events(&v, 8).expect("events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].note, 60);
        assert_eq!(events[0].velocity, 100);
    }
}
