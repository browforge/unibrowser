//! Magic-byte sniffing when file extensions are wrong or missing.

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffedFormat {
    Pdf,
    Png,
    Jpeg,
    Gif,
    Webp,
    Zip,
    Epub,
    Wav,
    Mp3,
    RiffAvi,
    Ogg,
    Flac,
}

pub fn sniff_format(path: &Path) -> Option<SniffedFormat> {
    let mut file = File::open(path).ok()?;
    let mut head = [0u8; 64];
    let n = file.read(&mut head).ok()?;
    if n < 4 {
        return None;
    }
    let h = &head[..n];
    if h.starts_with(b"%PDF") {
        return Some(SniffedFormat::Pdf);
    }
    if h.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some(SniffedFormat::Png);
    }
    if h.starts_with(b"\xff\xd8\xff") {
        return Some(SniffedFormat::Jpeg);
    }
    if h.starts_with(b"GIF87a") || h.starts_with(b"GIF89a") {
        return Some(SniffedFormat::Gif);
    }
    if h.starts_with(b"OggS") {
        return Some(SniffedFormat::Ogg);
    }
    if h.starts_with(b"fLaC") {
        return Some(SniffedFormat::Flac);
    }
    if h.starts_with(b"ID3") {
        return Some(SniffedFormat::Mp3);
    }
    if looks_like_mp3_frame(h) {
        return Some(SniffedFormat::Mp3);
    }
    if h.starts_with(b"RIFF") && n >= 12 {
        let form = &h[8..12];
        if form == b"WAVE" {
            return Some(SniffedFormat::Wav);
        }
        if form == b"WEBP" {
            return Some(SniffedFormat::Webp);
        }
        if form == b"AVI " {
            return Some(SniffedFormat::RiffAvi);
        }
    }
    if h.starts_with(b"PK\x03\x04") {
        if is_epub_zip(h, path) {
            return Some(SniffedFormat::Epub);
        }
        return Some(SniffedFormat::Zip);
    }
    None
}

fn looks_like_mp3_frame(h: &[u8]) -> bool {
    if h.len() < 2 {
        return false;
    }
    if h[0] != 0xff {
        return false;
    }
    (h[1] & 0xe0) == 0xe0
}

fn is_epub_zip(head: &[u8], path: &Path) -> bool {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("epub"))
    {
        return true;
    }
    let sample = String::from_utf8_lossy(head);
    sample.contains("mimetype")
        || sample.contains("META-INF/container.xml")
        || sample.contains("application/epub+zip")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_pdf_magic() {
        let dir = std::env::temp_dir();
        let p = dir.join(format!("ub-sniff-{}.pdf", std::process::id()));
        std::fs::write(&p, b"%PDF-1.4\n").unwrap();
        assert_eq!(sniff_format(&p), Some(SniffedFormat::Pdf));
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detects_png_magic() {
        let dir = std::env::temp_dir();
        let p = dir.join(format!("ub-sniff-{}.bin", std::process::id()));
        let mut f = File::create(&p).unwrap();
        f.write_all(b"\x89PNG\r\n\x1a\n").unwrap();
        assert_eq!(sniff_format(&p), Some(SniffedFormat::Png));
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detects_gif_and_webp_and_wav() {
        let dir = std::env::temp_dir();
        let gif = dir.join(format!("ub-sniff-{}-gif.bin", std::process::id()));
        std::fs::write(&gif, b"GIF89a").unwrap();
        assert_eq!(sniff_format(&gif), Some(SniffedFormat::Gif));

        let wav = dir.join(format!("ub-sniff-{}-wav.bin", std::process::id()));
        std::fs::write(&wav, b"RIFF\x00\x00\x00\x00WAVEfmt ").unwrap();
        assert_eq!(sniff_format(&wav), Some(SniffedFormat::Wav));

        let webp = dir.join(format!("ub-sniff-{}-webp.bin", std::process::id()));
        std::fs::write(&webp, b"RIFF\x00\x00\x00\x00WEBP").unwrap();
        assert_eq!(sniff_format(&webp), Some(SniffedFormat::Webp));

        let _ = std::fs::remove_file(gif);
        let _ = std::fs::remove_file(wav);
        let _ = std::fs::remove_file(webp);
    }

    #[test]
    fn detects_mp3_id3() {
        let dir = std::env::temp_dir();
        let p = dir.join(format!("ub-sniff-{}-mp3.bin", std::process::id()));
        std::fs::write(&p, b"ID3\x04\x00\x00\x00\x00\x00\x00").unwrap();
        assert_eq!(sniff_format(&p), Some(SniffedFormat::Mp3));
        let _ = std::fs::remove_file(p);
    }

    #[test]
    fn detects_ogg_flac_and_avi() {
        let dir = std::env::temp_dir();
        let ogg = dir.join(format!("ub-sniff-{}-ogg.bin", std::process::id()));
        std::fs::write(&ogg, b"OggS\x00").unwrap();
        assert_eq!(sniff_format(&ogg), Some(SniffedFormat::Ogg));

        let flac = dir.join(format!("ub-sniff-{}-flac.bin", std::process::id()));
        std::fs::write(&flac, b"fLaC").unwrap();
        assert_eq!(sniff_format(&flac), Some(SniffedFormat::Flac));

        let avi = dir.join(format!("ub-sniff-{}-avi.bin", std::process::id()));
        std::fs::write(&avi, b"RIFF\x00\x00\x00\x00AVI ").unwrap();
        assert_eq!(sniff_format(&avi), Some(SniffedFormat::RiffAvi));

        let _ = std::fs::remove_file(ogg);
        let _ = std::fs::remove_file(flac);
        let _ = std::fs::remove_file(avi);
    }
}

#[cfg(test)]
mod open_path_tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sniffed_avi_uses_video_backend() {
        let dir = std::env::temp_dir();
        let p = dir.join(format!("ub-avi-{}.wrongext", std::process::id()));
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(b"RIFF\x00\x00\x00\x00AVI \x00").unwrap();
        let doc = crate::state::open_file_document(&p).expect("document");
        assert_eq!(doc.backend.backend_id(), base_video::BACKEND_ID);
        let _ = std::fs::remove_file(p);
    }
}
