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
}
