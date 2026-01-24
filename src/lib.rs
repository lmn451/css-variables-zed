use std::fs;
use std::path::Path;
use std::time::SystemTime;

use zed::serde_json::Value;
use zed::settings::{BinarySettings, LspSettings};
use zed_extension_api as zed;

const CSS_VARIABLES_BINARY_NAME: &str = "css-variable-lsp";
const CSS_VARIABLES_RELEASE_REPO: &str = "lmn451/css-lsp-rust";
const CSS_VARIABLES_RELEASE_TAG: &str = "v0.1.6";
const CSS_VARIABLES_CACHE_PREFIX: &str = "css-variable-lsp-";

struct CssVariablesExtension {
    cached_binary_path: Option<String>,
}

impl CssVariablesExtension {
    fn resolve_css_variables_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
        user_settings: Option<&Value>,
        binary_settings: Option<&BinarySettings>,
    ) -> zed::Result<String> {
        if let Some(path) = binary_settings.and_then(|settings| settings.path.as_ref()) {
            return Ok(path.clone());
        }

        if let Some(path) = css_variables_binary_from_settings(user_settings) {
            return Ok(path);
        }

        let (platform, arch) = zed::current_platform();

        if let Some(path) = find_local_dev_binary(platform) {
            return Ok(path);
        }

        if let Some(path) = worktree.which(CSS_VARIABLES_BINARY_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let binary_name = binary_name_for_platform(platform);
        let version_dir = format!("{CSS_VARIABLES_CACHE_PREFIX}{CSS_VARIABLES_RELEASE_TAG}");

        if let Some(path) = find_binary_in_dir(&version_dir, binary_name)? {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let asset_name = asset_name_for_platform(platform, arch)?;
        let (download_type, is_archive) = download_type_for_asset(asset_name);
        let download_url = format!(
            "https://github.com/{CSS_VARIABLES_RELEASE_REPO}/releases/download/{CSS_VARIABLES_RELEASE_TAG}/{asset_name}"
        );
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("failed to create directory '{version_dir}': {err}"))?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        let binary_path = if is_archive {
            zed::download_file(&download_url, &version_dir, download_type)
                .map_err(|err| format!("failed to download {asset_name}: {err}"))?;

            find_binary_in_dir(&version_dir, binary_name)?.ok_or_else(|| {
                format!("downloaded archive did not contain expected binary '{binary_name}'")
            })?
        } else {
            let binary_path = format!("{version_dir}/{binary_name}");
            if !Path::new(&binary_path).exists() {
                zed::download_file(&download_url, &binary_path, download_type)
                    .map_err(|err| format!("failed to download {asset_name}: {err}"))?;
            }
            binary_path
        };

        if platform != zed::Os::Windows {
            zed::make_file_executable(&binary_path)?;
        }

        prune_cached_versions(CSS_VARIABLES_CACHE_PREFIX, &version_dir);
        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for CssVariablesExtension {
    fn new() -> Self {
        CssVariablesExtension {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != "css-variables" {
            return Err(format!("Unknown language server id: {language_server_id}"));
        }

        let lsp_settings = LspSettings::for_worktree("css-variables", worktree).ok();
        let user_settings = lsp_settings
            .as_ref()
            .and_then(|lsp_settings| lsp_settings.settings.clone());
        let binary_settings = lsp_settings
            .as_ref()
            .and_then(|lsp_settings| lsp_settings.binary.as_ref());

        build_css_variables_command(
            self,
            language_server_id,
            worktree,
            user_settings,
            binary_settings,
        )
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        if let Ok(lsp_settings) = LspSettings::for_worktree("css-variables", worktree) {
            return Ok(Some(build_workspace_settings(lsp_settings.settings)));
        }

        Ok(Some(build_workspace_settings(None)))
    }
}

fn build_workspace_settings(user_settings: Option<Value>) -> Value {
    // Return default settings matching css-variables-language-server's defaultSettings.
    // We nest them under the `cssVariables` key because the server calls
    // `connection.workspace.getConfiguration('cssVariables')`, and Zed's
    // bridge likely indexes into this object by that key.
    let mut settings = zed::serde_json::json!({
        "cssVariables": {
            "lookupFiles": [
                "**/*.less",
                "**/*.scss",
                "**/*.sass",
                "**/*.css",
                "**/*.html",
                "**/*.vue",
                "**/*.svelte",
                "**/*.astro",
                "**/*.ripple"
            ],
            "blacklistFolders": [
                "**/.cache/**",
                "**/.DS_Store",
                "**/.git/**",
                "**/.hg/**",
                "**/.next/**",
                "**/.svn/**",
                "**/bower_components/**",
                "**/CVS/**",
                "**/dist/**",
                "**/node_modules/**",
                "**/tests/**",
                "**/tmp/**",
            ],
        }
    });

    if let Some(user_settings) = user_settings {
        if user_settings.get("cssVariables").is_some() {
            merge_json_value(&mut settings, &user_settings);
        }
    }

    settings
}

fn merge_json_value(base: &mut Value, overlay: &Value) {
    if let Value::Object(overlay_map) = overlay {
        if let Value::Object(base_map) = base {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_json_value(base_value, overlay_value),
                    None => {
                        base_map.insert(key.clone(), overlay_value.clone());
                    }
                }
            }
            return;
        }
    }

    *base = overlay.clone();
}

fn build_css_variables_command(
    extension: &mut CssVariablesExtension,
    language_server_id: &zed::LanguageServerId,
    worktree: &zed::Worktree,
    user_settings: Option<Value>,
    binary_settings: Option<&BinarySettings>,
) -> zed::Result<zed::Command> {
    let command = extension.resolve_css_variables_binary(
        language_server_id,
        worktree,
        user_settings.as_ref(),
        binary_settings,
    )?;

    // Build merged settings with defaults so CLI args include defaults when user has no custom settings
    let merged_settings = build_workspace_settings(user_settings);
    let mut args = build_css_variables_args(Some(merged_settings));

    if let Some(extra_args) = binary_settings.and_then(|settings| settings.arguments.clone()) {
        args.extend(extra_args);
    }

    Ok(zed::Command {
        command,
        args,
        env: worktree.shell_env(),
    })
}

fn build_css_variables_args(user_settings: Option<Value>) -> Vec<String> {
    let mut args = vec!["--color-only-variables".to_string(), "--stdio".to_string()];

    args.extend(build_settings_args(user_settings));
    args
}

fn build_settings_args(user_settings: Option<Value>) -> Vec<String> {
    let mut args = Vec::new();

    let css_variables = user_settings
        .as_ref()
        .and_then(|settings| settings.get("cssVariables"));

    let lookup_files = css_variables
        .and_then(|settings| settings.get("lookupFiles"))
        .map(extract_string_array)
        .unwrap_or_default();

    let blacklist_folders = css_variables
        .and_then(|settings| settings.get("blacklistFolders"))
        .map(extract_string_array)
        .unwrap_or_default();

    for glob in lookup_files {
        args.push("--lookup-file".to_string());
        args.push(glob);
    }

    for glob in blacklist_folders {
        args.push("--ignore-glob".to_string());
        args.push(glob);
    }

    args
}

fn extract_string_array(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|value| value.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

fn css_variables_binary_from_settings(user_settings: Option<&Value>) -> Option<String> {
    let settings = user_settings?;
    if let Some(path) = extract_binary_path(settings.get("binary")) {
        return Some(path);
    }
    let css_variables = settings.get("cssVariables")?;
    extract_binary_path(css_variables.get("binary"))
}

fn extract_binary_path(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(path) => Some(path.clone()),
        Value::Object(binary) => binary
            .get("path")
            .and_then(|path| path.as_str())
            .map(|path| path.to_string()),
        _ => None,
    }
}

fn binary_name_for_platform(platform: zed::Os) -> &'static str {
    match platform {
        zed::Os::Windows => "css-variable-lsp.exe",
        _ => CSS_VARIABLES_BINARY_NAME,
    }
}

fn asset_name_for_platform(
    platform: zed::Os,
    arch: zed::Architecture,
) -> zed::Result<&'static str> {
    match (platform, arch) {
        (zed::Os::Mac, zed::Architecture::Aarch64) => Ok("css-variable-lsp-macos-aarch64.tar.gz"),
        (zed::Os::Mac, zed::Architecture::X8664) => Ok("css-variable-lsp-macos-x86_64.tar.gz"),
        (zed::Os::Linux, zed::Architecture::Aarch64) => Ok("css-variable-lsp-linux-aarch64.tar.gz"),
        (zed::Os::Linux, zed::Architecture::X8664) => Ok("css-variable-lsp-linux-x86_64.tar.gz"),
        (zed::Os::Windows, zed::Architecture::X8664) => {
            Ok("css-variable-lsp-windows-x86_64.exe.zip")
        }
        (zed::Os::Windows, zed::Architecture::Aarch64) => {
            Ok("css-variable-lsp-windows-aarch64.exe.zip")
        }
        (platform, arch) => Err(format!("unsupported platform {platform:?} {arch:?}")),
    }
}

fn download_type_for_asset(asset_name: &str) -> (zed::DownloadedFileType, bool) {
    if asset_name.ends_with(".tar.gz") || asset_name.ends_with(".tgz") {
        (zed::DownloadedFileType::GzipTar, true)
    } else if asset_name.ends_with(".zip") {
        (zed::DownloadedFileType::Zip, true)
    } else if asset_name.ends_with(".gz") {
        (zed::DownloadedFileType::Gzip, false)
    } else {
        (zed::DownloadedFileType::Uncompressed, false)
    }
}

fn find_local_dev_binary(platform: zed::Os) -> Option<String> {
    let binary_name = binary_name_for_platform(platform);
    let cwd = std::env::current_dir().ok()?;

    let repo_candidates = ["../css-lsp-rust", "../rust-css-lsp"];
    let build_kinds = ["release", "debug"];
    let mut best: Option<(SystemTime, String)> = None;

    for repo in repo_candidates {
        let repo_root = cwd.join(repo);
        for build_kind in build_kinds {
            let candidate = repo_root
                .join("target")
                .join(build_kind)
                .join(binary_name);

            let metadata = match fs::metadata(&candidate) {
                Ok(metadata) if metadata.is_file() => metadata,
                _ => continue,
            };

            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let path = candidate.to_string_lossy().to_string();
            let use_candidate = match &best {
                Some((best_time, _)) => modified > *best_time,
                None => true,
            };

            if use_candidate {
                best = Some((modified, path));
            }
        }
    }

    best.map(|(_, path)| path)
}

fn find_binary_in_dir(dir: &str, binary_name: &str) -> zed::Result<Option<String>> {
    let root = Path::new(dir);
    if !root.exists() {
        return Ok(None);
    }
    find_binary_in_tree(root, binary_name)
}

fn find_binary_in_tree(root: &Path, binary_name: &str) -> zed::Result<Option<String>> {
    let entries =
        fs::read_dir(root).map_err(|err| format!("failed to read directory {root:?}: {err}"))?;
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|err| format!("failed to read file type for {path:?}: {err}"))?;
        if file_type.is_dir() {
            if let Some(found) = find_binary_in_tree(&path, binary_name)? {
                return Ok(Some(found));
            }
        } else if file_type.is_file() {
            if entry.file_name().to_str() == Some(binary_name) {
                return Ok(Some(path.to_string_lossy().to_string()));
            }
        }
    }
    Ok(None)
}

fn prune_cached_versions(prefix: &str, keep: &str) {
    let entries = match fs::read_dir(".") {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        if name.starts_with(prefix) && name != keep {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(path).ok();
            }
        }
    }
}

zed::register_extension!(CssVariablesExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use zed::serde_json::json;

    #[test]
    fn merges_nested_css_variables_settings() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": ["**/*.css"],
                "blacklistFolders": ["**/dist"]
            }
        });

        let settings = build_workspace_settings(Some(user_settings));

        assert_eq!(settings["cssVariables"]["lookupFiles"], json!(["**/*.css"]));
        assert_eq!(
            settings["cssVariables"]["blacklistFolders"],
            json!(["**/dist"])
        );
    }

    #[test]
    fn ignores_top_level_settings() {
        let user_settings = json!({
            "lookupFiles": ["**/*.scss"],
            "blacklistFolders": ["**/vendor"]
        });

        let settings = build_workspace_settings(Some(user_settings));

        assert_eq!(
            settings["cssVariables"]["lookupFiles"][0],
            json!("**/*.less")
        );
        assert!(settings["cssVariables"]["blacklistFolders"].is_array());
    }

    #[test]
    fn keeps_defaults_when_only_one_setting_is_overridden() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": ["**/*.vue"]
            }
        });

        let settings = build_workspace_settings(Some(user_settings));

        assert_eq!(settings["cssVariables"]["lookupFiles"], json!(["**/*.vue"]));
        assert!(settings["cssVariables"]["blacklistFolders"].is_array());
    }

    #[test]
    fn builds_cli_args_from_settings() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": ["a.css", "b.css"],
                "blacklistFolders": ["**/dist/**"]
            }
        });

        let args = build_css_variables_args(Some(user_settings));

        assert_eq!(
            args,
            vec![
                "--color-only-variables",
                "--stdio",
                "--lookup-file",
                "a.css",
                "--lookup-file",
                "b.css",
                "--ignore-glob",
                "**/dist/**",
            ]
        );
    }

    #[test]
    fn ignores_non_array_settings_for_cli_args() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": "a.css",
                "blacklistFolders": 42
            }
        });

        let args = build_css_variables_args(Some(user_settings));

        assert_eq!(args, vec!["--color-only-variables", "--stdio"]);
    }

    #[test]
    fn default_blacklist_globs_are_passed_to_cli_args() {
        let settings = build_workspace_settings(None);
        let args = build_css_variables_args(Some(settings));

        assert!(args.contains(&"--ignore-glob".to_string()));
        assert!(args.contains(&"**/node_modules/**".to_string()));
        assert!(args.contains(&"**/dist/**".to_string()));
    }

    // Asset naming contract tests - these MUST match the LSP repo's release workflow
    // If these fail, the extension won't be able to download the binary

    #[test]
    fn asset_name_macos_aarch64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Mac, zed::Architecture::Aarch64).unwrap(),
            "css-variable-lsp-macos-aarch64.tar.gz"
        );
    }

    #[test]
    fn asset_name_macos_x86_64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Mac, zed::Architecture::X8664).unwrap(),
            "css-variable-lsp-macos-x86_64.tar.gz"
        );
    }

    #[test]
    fn asset_name_linux_aarch64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Linux, zed::Architecture::Aarch64).unwrap(),
            "css-variable-lsp-linux-aarch64.tar.gz"
        );
    }

    #[test]
    fn asset_name_linux_x86_64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Linux, zed::Architecture::X8664).unwrap(),
            "css-variable-lsp-linux-x86_64.tar.gz"
        );
    }

    #[test]
    fn asset_name_windows_aarch64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Windows, zed::Architecture::Aarch64).unwrap(),
            "css-variable-lsp-windows-aarch64.exe.zip"
        );
    }

    #[test]
    fn asset_name_windows_x86_64() {
        assert_eq!(
            asset_name_for_platform(zed::Os::Windows, zed::Architecture::X8664).unwrap(),
            "css-variable-lsp-windows-x86_64.exe.zip"
        );
    }
}
