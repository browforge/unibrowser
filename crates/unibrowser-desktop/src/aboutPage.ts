export function renderAbout(root: HTMLElement) {
  root.innerHTML = "";
  const lead = document.createElement("p");
  lead.textContent =
    "Unibrowser is a unified file and web browser shell. This build is open source under the Apache License 2.0.";
  root.appendChild(lead);

  const licence = document.createElement("section");
  licence.className = "about-section";
  const lh = document.createElement("h3");
  lh.textContent = "Licence";
  licence.appendChild(lh);
  const lp = document.createElement("p");
  lp.textContent =
    "Copyright 2026 Unibrowser contributors. Licensed under the Apache License, Version 2.0. You may obtain a copy at https://www.apache.org/licenses/LICENSE-2.0";
  licence.appendChild(lp);
  root.appendChild(licence);

  const third = document.createElement("section");
  third.className = "about-section";
  const th = document.createElement("h3");
  th.textContent = "Third-party notices (stub)";
  third.appendChild(th);
  const ul = document.createElement("ul");
  ul.className = "about-third-party";
  [
    "Monaco Editor — MIT",
    "pdf.js — Apache-2.0",
    "Tauri — MIT / Apache-2.0",
    "ffmpeg (sidecar, when used) — LGPL 2.1+",
  ].forEach((line) => {
    const li = document.createElement("li");
    li.textContent = line;
    ul.appendChild(li);
  });
  third.appendChild(ul);
  const note = document.createElement("p");
  note.className = "viewer-note";
  note.textContent = "Full NOTICE file generation is not wired yet.";
  third.appendChild(note);
  root.appendChild(third);
}
