#!/usr/bin/env python3
"""Create minimal DOCX for e2e/screenshots (no external deps)."""
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
out = ROOT / "fixtures" / "minimal.docx"
out.parent.mkdir(parents=True, exist_ok=True)

content_types = """<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"""

rels = """<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"""

document = """<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>E2E DOCX sample paragraph for font commands.</w:t></w:r></w:p>
  </w:body>
</w:document>"""

with zipfile.ZipFile(out, "w") as zf:
    zf.writestr("[Content_Types].xml", content_types)
    zf.writestr("_rels/.rels", rels)
    zf.writestr("word/document.xml", document)

print(f"wrote {out}")
