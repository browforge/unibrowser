//! Magic-byte sniffing when file extensions are wrong or missing.

use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SniffedFormat {
    Pdf,
    Png,
    Jpeg,
    Zip,
    Epub,
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
    if h.starts_with(b"PK\x03\x04") {
        if is_epub_zip(h, path) {
            return Some(SniffedFormat::Epub);
        }
        return Some(SniffedFormat::Zip);
    }
    None
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
}
