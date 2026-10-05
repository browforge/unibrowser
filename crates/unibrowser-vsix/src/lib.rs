//! Install `.vsix` packages to the user config dir and import V1 metadata ([ADR 0014](../../docs/adr/0014-vsix-compatibility-layer.md)).

use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use unibrowser_engine::CommandRegistry;
use unibrowser_mod_api::CommandMeta;
use zip::ZipArchive;

pub const VSIX_HOST_REQUIRED_REASON: &str = "Needs extension host (ADR 0018)";

#[derive(Debug, Error)]
pub enum VsixError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Msg(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsixCommandContrib {
    pub command: String,
    pub title: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsixKeybindingContrib {
    pub command: String,
    pub key: String,
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VsixContributions {
    pub extension_id: String,
    pub commands: Vec<VsixCommandContrib>,
    pub keybindings: Vec<VsixKeybindingContrib>,
    #[serde(default)]
    pub activation_events: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct VsixCommandDetail {
    pub command_id: String,
    pub title: String,
    pub extension_id: String,
    pub activation_events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledVsixRecord {
    pub extension_id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub display_name: Option<String>,
    pub license: Option<String>,
    pub install_dir: String,
    pub contributions: VsixContributions,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub compatibility_level: String,
    #[serde(default)]
    pub compatibility_missing_apis: Vec<String>,
    #[serde(default)]
    pub theme_notes: Vec<String>,
    #[serde(default)]
    pub grammar_notes: Vec<String>,
}

fn default_enabled() -> bool {
    true
}

pub fn extensions_root(config_dir: &Path) -> PathBuf {
    config_dir.join("vsix-extensions")
}

pub fn list_installed(root: &Path) -> Result<Vec<InstalledVsixRecord>, VsixError> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let manifest = path.join("install.json");
        if !manifest.is_file() {
            continue;
        }
        let data = fs::read_to_string(&manifest)?;
        let rec: InstalledVsixRecord = serde_json::from_str(&data)?;
        out.push(rec);
    }
    out.sort_by(|a, b| a.extension_id.cmp(&b.extension_id));
    Ok(out)
}

pub fn install_vsix_bytes(
    root: &Path,
    bytes: &[u8],
    license_hint: Option<&str>,
) -> Result<InstalledVsixRecord, VsixError> {
    fs::create_dir_all(root)?;
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;
    let package_json = read_package_json_from_archive(&mut archive)?;
    let extension_id = package_json.extension_id();
    let install_dir = root.join(sanitize_dir_name(&format!(
        "{}-{}",
        extension_id,
        package_json.version
    )));
    if install_dir.exists() {
        fs::remove_dir_all(&install_dir)?;
    }
    fs::create_dir_all(&install_dir)?;
    let vsix_copy = install_dir.join("package.vsix");
    fs::write(&vsix_copy, bytes)?;
    extract_extension_tree(&mut archive, &install_dir)?;
    let contributions = extract_contributions(&package_json, &extension_id);
    let contrib_path = install_dir.join("contributions.json");
    fs::write(&contrib_path, serde_json::to_string_pretty(&contributions)?)?;
    let (theme_notes, grammar_notes) = asset_notes_from_package(&package_json);
    let compatibility = compatibility_for_install(&install_dir);
    let record = InstalledVsixRecord {
        extension_id,
        name: package_json.name.clone(),
        version: package_json.version.clone(),
        publisher: package_json.publisher.clone(),
        display_name: package_json.display_name.clone(),
        license: package_json.license.clone().or_else(|| license_hint.map(str::to_string)),
        install_dir: install_dir.to_string_lossy().into_owned(),
        contributions,
        enabled: true,
        compatibility_level: compatibility.level,
        compatibility_missing_apis: compatibility.missing_apis,
        theme_notes,
        grammar_notes,
    };
    write_install_record(&install_dir, &record)?;
    Ok(record)
}

pub fn write_install_record(install_dir: &Path, record: &InstalledVsixRecord) -> Result<(), VsixError> {
    fs::write(
        install_dir.join("install.json"),
        serde_json::to_string_pretty(record)?,
    )?;
    Ok(())
}

pub fn set_extension_enabled(root: &Path, extension_id: &str, enabled: bool) -> Result<(), VsixError> {
    for rec in list_installed(root)? {
        if rec.extension_id != extension_id {
            continue;
        }
        let dir = PathBuf::from(&rec.install_dir);
        let mut updated = rec;
        updated.enabled = enabled;
        write_install_record(&dir, &updated)?;
        return Ok(());
    }
    Err(VsixError::Msg(format!("extension not installed: {}", extension_id)))
}

pub fn uninstall_extension(root: &Path, extension_id: &str) -> Result<(), VsixError> {
    for rec in list_installed(root)? {
        if rec.extension_id != extension_id {
            continue;
        }
        let dir = PathBuf::from(&rec.install_dir);
        if dir.is_dir() {
            fs::remove_dir_all(&dir)?;
        }
        return Ok(());
    }
    Err(VsixError::Msg(format!("extension not installed: {}", extension_id)))
}

#[derive(Debug, Clone)]
pub struct VsixCommandContext {
    pub extension_id: String,
    pub activation_events: Vec<String>,
    pub install_dir: PathBuf,
}

pub fn lookup_vsix_command(root: &Path, command_id: &str) -> Option<(String, Vec<String>)> {
    lookup_vsix_command_context(root, command_id).map(|c| {
        (c.extension_id.clone(), c.activation_events.clone())
    })
}

pub fn lookup_vsix_command_context(root: &Path, command_id: &str) -> Option<VsixCommandContext> {
    for rec in list_installed(root).unwrap_or_default() {
        if !rec.enabled {
            continue;
        }
        for cmd in &rec.contributions.commands {
            if cmd.command == command_id {
                return Some(VsixCommandContext {
                    extension_id: rec.extension_id.clone(),
                    activation_events: rec.contributions.activation_events.clone(),
                    install_dir: PathBuf::from(&rec.install_dir),
                });
            }
        }
    }
    None
}

/// Relative `main` path inside the `extension/` folder (VS Code layout).
pub fn read_extension_main(install_dir: &Path) -> Option<String> {
    let pkg_path = install_dir.join("extension/package.json");
    let data = fs::read_to_string(pkg_path).ok()?;
    let pkg: serde_json::Value = serde_json::from_str(&data).ok()?;
    pkg.get("main")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}

pub fn enabled_command_details(installed: &[InstalledVsixRecord]) -> Vec<VsixCommandDetail> {
    let mut out = Vec::new();
    for rec in installed {
        if !rec.enabled {
            continue;
        }
        for cmd in &rec.contributions.commands {
            let title = if let Some(cat) = &cmd.category {
                format!("{}: {}", cat, cmd.title)
            } else {
                cmd.title.clone()
            };
            out.push(VsixCommandDetail {
                command_id: cmd.command.clone(),
                title,
                extension_id: rec.extension_id.clone(),
                activation_events: rec.contributions.activation_events.clone(),
            });
        }
    }
    out.sort_by(|a, b| a.command_id.cmp(&b.command_id));
    out
}

pub fn enabled_keybindings(installed: &[InstalledVsixRecord]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for rec in installed {
        if !rec.enabled {
            continue;
        }
        for kb in &rec.contributions.keybindings {
            out.push((kb.command.clone(), kb.key.clone()));
        }
    }
    out
}

pub fn install_vsix_file(root: &Path, vsix_path: &Path) -> Result<InstalledVsixRecord, VsixError> {
    let bytes = fs::read(vsix_path)?;
    install_vsix_bytes(root, &bytes, None)
}

pub fn register_enabled_contributions(
    registry: &mut CommandRegistry,
    root: &Path,
    host_command_ids: &Mutex<HashSet<String>>,
) {
    let installed = list_installed(root).unwrap_or_default();
    if let Ok(mut ids) = host_command_ids.lock() {
        ids.clear();
        for rec in installed {
            if !rec.enabled {
                continue;
            }
            register_contributions(registry, &rec.contributions);
            for cmd in &rec.contributions.commands {
                ids.insert(cmd.command.clone());
            }
        }
    }
}

pub fn register_contributions(registry: &mut CommandRegistry, contributions: &VsixContributions) {
    for cmd in &contributions.commands {
        let title = if let Some(cat) = &cmd.category {
            format!("{}: {}", cat, cmd.title)
        } else {
            cmd.title.clone()
        };
        let id = leak_str(&cmd.command);
        let title = leak_str(&title);
        registry.register_command(CommandMeta {
            id,
            title,
            icon: "vsix",
            default_keybinding: None,
            aliases: &[],
        });
    }
}

fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

fn sanitize_dir_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    name: String,
    publisher: String,
    version: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default, rename = "activationEvents")]
    activation_events: Vec<String>,
    #[serde(default)]
    contributes: Option<Contributes>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct Contributes {
    #[serde(default)]
    commands: Vec<PackageCommand>,
    #[serde(default)]
    keybindings: Vec<VsixKeybindingContrib>,
    #[serde(default)]
    themes: Vec<ThemeContrib>,
    #[serde(default)]
    grammars: Vec<GrammarContrib>,
}

#[derive(Debug, Clone, Deserialize)]
struct ThemeContrib {
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GrammarContrib {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    scope_name: Option<String>,
    #[serde(default)]
    path: Option<String>,
}

struct InstallCompatibility {
    level: String,
    missing_apis: Vec<String>,
}

fn compatibility_for_install(install_dir: &Path) -> InstallCompatibility {
    let main_rel = read_extension_main(install_dir);
    let mut source = String::new();
    if let Some(main_rel) = main_rel {
        let main_path = install_dir.join("extension").join(main_rel);
        if main_path.is_file() {
            source = fs::read_to_string(&main_path).unwrap_or_default();
        }
    }
    let report = unibrowser_extension_host::compatibility_report(&source);
    let level = match report.level {
        unibrowser_extension_host::CompatibilityLevel::Works => "works",
        unibrowser_extension_host::CompatibilityLevel::Partial => "partial",
        unibrowser_extension_host::CompatibilityLevel::Unsupported => "unsupported",
    };
    InstallCompatibility {
        level: level.into(),
        missing_apis: report.missing_apis,
    }
}

fn asset_notes_from_package(package: &PackageJson) -> (Vec<String>, Vec<String>) {
    let contributes = package.contributes.as_ref().cloned().unwrap_or_default();
    let themes = contributes
        .themes
        .into_iter()
        .map(|t| {
            let label = t.label.unwrap_or_else(|| "theme".into());
            let path = t.path.unwrap_or_else(|| "?".into());
            format!("{label} ({path}) — recognised, not applied")
        })
        .collect();
    let grammars = contributes
        .grammars
        .into_iter()
        .map(|g| {
            let lang = g.language.unwrap_or_else(|| "unknown".into());
            let path = g.path.unwrap_or_else(|| "?".into());
            format!("{lang} grammar ({path}) — recognised, not applied")
        })
        .collect();
    (themes, grammars)
}

#[derive(Debug, Clone, Deserialize)]
struct PackageCommand {
    command: String,
    title: String,
    #[serde(default)]
    category: Option<String>,
}

impl PackageJson {
    fn extension_id(&self) -> String {
        format!("{}.{}", self.publisher, self.name)
    }
}

fn read_package_json_from_archive(archive: &mut ZipArchive<std::io::Cursor<&[u8]>>) -> Result<PackageJson, VsixError> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().replace('\\', "/");
        if name.ends_with("/package.json") && name.contains("extension/") {
            let mut buf = String::new();
            file.read_to_string(&mut buf)?;
            return Ok(serde_json::from_str(&buf)?);
        }
    }
    Err(VsixError::Msg("package.json not found in VSIX".into()))
}

fn extract_extension_tree(
    archive: &mut ZipArchive<std::io::Cursor<&[u8]>>,
    install_dir: &Path,
) -> Result<(), VsixError> {
    let prefix = find_extension_prefix(archive)?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().replace('\\', "/");
        if !name.starts_with(&prefix) {
            continue;
        }
        let rel = name.strip_prefix(&prefix).unwrap_or(&name);
        if rel.is_empty() {
            continue;
        }
        let out = install_dir.join("extension").join(rel);
        if name.ends_with('/') {
            fs::create_dir_all(&out)?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut dest = fs::File::create(&out)?;
        std::io::copy(&mut file, &mut dest)?;
    }
    Ok(())
}

fn find_extension_prefix(archive: &mut ZipArchive<std::io::Cursor<&[u8]>>) -> Result<String, VsixError> {
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name().replace('\\', "/");
        if name.ends_with("extension/package.json") {
            return Ok(name
                .strip_suffix("package.json")
                .unwrap_or("extension/")
                .to_string());
        }
    }
    Err(VsixError::Msg("extension/ folder not found in VSIX".into()))
}

fn extract_contributions(package: &PackageJson, extension_id: &str) -> VsixContributions {
    let contributes = package
        .contributes
        .as_ref()
        .cloned()
        .unwrap_or_default();
    let commands = contributes
        .commands
        .into_iter()
        .map(|c| VsixCommandContrib {
            command: c.command,
            title: c.title,
            category: c.category,
        })
        .collect();
    VsixContributions {
        extension_id: extension_id.to_string(),
        commands,
        keybindings: contributes.keybindings,
        activation_events: package.activation_events.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn sample_vsix_bytes() -> Vec<u8> {
        let buf = Vec::new();
        let mut zip = ZipWriter::new(std::io::Cursor::new(buf));
        let opts = SimpleFileOptions::default();
        zip.start_file("extension/package.json", opts)
            .unwrap();
        zip.write_all(
            br#"{
  "name": "demo",
  "publisher": "unibrowser",
  "version": "1.0.0",
  "activationEvents": ["onCommand:demo.hello"],
  "contributes": {
    "commands": [{"command": "demo.hello", "title": "Hello", "category": "Demo"}],
    "keybindings": [{"command": "demo.hello", "key": "ctrl+shift+h"}]
  }
}"#,
        )
        .unwrap();
        zip.start_file("extension/readme.md", opts).unwrap();
        zip.write_all(b"# demo").unwrap();
        let finished = zip.finish().unwrap().into_inner();
        finished
    }

    #[test]
    fn enabled_keybindings_skips_disabled() {
        let dir = std::env::temp_dir().join("vsix-kb-test");
        let _ = fs::remove_dir_all(&dir);
        let rec = install_vsix_bytes(&dir, &sample_vsix_bytes(), None).expect("install");
        set_extension_enabled(&dir, &rec.extension_id, false).expect("disable");
        let list = list_installed(&dir).expect("list");
        assert!(enabled_keybindings(&list).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn sample_extension_declares_main() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/vsix-sample-extension");
        let main = read_extension_main(&root).expect("main");
        assert!(main.ends_with("main.js"));
    }

    #[test]
    fn install_and_list_roundtrip() {
        let dir = std::env::temp_dir().join("vsix-install-test");
        let _ = fs::remove_dir_all(&dir);
        let rec = install_vsix_bytes(&dir, &sample_vsix_bytes(), Some("MIT".into())).expect("install");
        assert_eq!(rec.extension_id, "unibrowser.demo");
        assert_eq!(rec.contributions.commands.len(), 1);
        assert_eq!(
            rec.contributions.activation_events,
            vec!["onCommand:demo.hello".to_string()]
        );
        let details = enabled_command_details(&list_installed(&dir).expect("list"));
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].command_id, "demo.hello");
        let list = list_installed(&dir).expect("list");
        assert_eq!(list.len(), 1);
        let _ = fs::remove_dir_all(&dir);
    }
}
