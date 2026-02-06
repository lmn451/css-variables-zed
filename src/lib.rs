use std::fs;
use std::path::Path;
use std::time::SystemTime;

use zed::serde_json::Value;
use zed::settings::{CommandSettings, LspSettings};
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
        _user_settings: Option<&Value>,
        binary_settings: Option<&CommandSettings>,
    ) -> zed::Result<String> {
        if let Some(path) = binary_settings.and_then(|settings| settings.path.as_ref()) {
            return Ok(path.clone());
        }

        let (platform, arch) = zed::current_platform();

        // 2) Local dev binary (for extension developers)
        if let Some(path) = find_local_dev_binary(platform) {
            return Ok(path);
        }

        // 3) Check cached binary
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).map_or(false, |stat| stat.is_file()) {
                return Ok(path.clone());
            }
        }

        let binary_name = binary_name_for_platform(platform);
        let version_dir = format!("{CSS_VARIABLES_CACHE_PREFIX}{CSS_VARIABLES_RELEASE_TAG}");

        // 3) Already downloaded Rust binary
        if let Some(path) = find_binary_in_dir(&version_dir, binary_name)? {
            self.cached_binary_path = Some(path.clone());
            return Ok(path);
        }

        // 4) Download pinned Rust release
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

        // Try Rust binary first, then PATH, then fall back to npm
        let command = match self.resolve_css_variables_binary(
            language_server_id,
            worktree,
            user_settings.as_ref(),
            binary_settings,
        ) {
            Ok(path) => path,
            Err(_rust_err) => {
                // 4) Check PATH before npm fallback
                if let Some(path) = worktree.which(CSS_VARIABLES_BINARY_NAME) {
                    path
                } else {
                    // 5) npm fallback
                    return build_npm_fallback_command(worktree, user_settings);
                }
            }
        };

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
            "undefinedVarFallback": "warning",
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

fn build_npm_fallback_command(
    worktree: &zed::Worktree,
    user_settings: Option<Value>,
) -> zed::Result<zed::Command> {
    let package = "css-variable-lsp";
    let npm_version =
        npm_version_from_settings(user_settings.as_ref()).unwrap_or_else(|| "latest".to_string());

    // Install the package if it's missing or on a different version.
    if npm_version == "latest" {
        let latest_version = zed::npm_package_latest_version(package);
        let installed_version = zed::npm_package_installed_version(package)?;
        match (latest_version, installed_version) {
            (Ok(latest), Some(installed)) if installed == latest => {
                // already correct version
            }
            (Ok(latest), _) => {
                zed::npm_install_package(package, &latest)?;
            }
            (Err(_), Some(_)) => {
                // Fall back to the installed version if we can't reach npm.
            }
            (Err(err), None) => {
                return Err(format!(
                    "Unable to resolve latest npm version for {package}: {err}"
                ));
            }
        }
    } else if is_npm_version(&npm_version) {
        let installed_version = zed::npm_package_installed_version(package)?;
        if installed_version.as_deref() != Some(npm_version.as_str()) {
            zed::npm_install_package(package, &npm_version)?;
        }
    } else {
        let installed_version = zed::npm_package_installed_version(package)?;
        match zed::npm_install_package(package, &npm_version) {
            Ok(()) => {}
            Err(err) => {
                if installed_version.is_some() {
                    // Fall back to the installed version if we can't reach npm.
                } else {
                    return Err(format!(
                        "Unable to install npm package {package}@{npm_version}: {err}"
                    ));
                }
            }
        }
    }

    let node = zed::node_binary_path()?;

    // Use JS entrypoint directly to avoid npm .bin shell shim issues on Windows.
    // Get the extension's working directory and construct path to entrypoint
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Could not get current directory: {}", e))?;
    let entrypoint_path = current_dir
        .join("node_modules")
        .join(package)
        .join("out")
        .join("server.js");

    if !entrypoint_path.exists() {
        return Err(format!(
            "Language server entrypoint does not exist: {:?} (current_dir: {:?})",
            entrypoint_path, current_dir
        ));
    }

    let env = worktree.shell_env();
    let mut args = vec![entrypoint_path.to_string_lossy().to_string()];

    // Build merged settings with defaults so CLI args include defaults when user has no custom settings
    let merged_settings = build_workspace_settings(user_settings);
    args.extend(build_css_variables_args(Some(merged_settings)));

    Ok(zed::Command {
        command: node,
        args,
        env,
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

    let undefined_var_fallback = css_variables
        .and_then(|settings| settings.get("undefinedVarFallback"))
        .and_then(|value| value.as_str());

    for glob in lookup_files {
        args.push("--lookup-file".to_string());
        args.push(glob);
    }

    for glob in blacklist_folders {
        args.push("--ignore-glob".to_string());
        args.push(glob);
    }

    if let Some(mode) = undefined_var_fallback {
        args.push("--undefined-var-fallback".to_string());
        args.push(mode.to_string());
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

fn npm_version_from_settings(user_settings: Option<&Value>) -> Option<String> {
    user_settings
        .and_then(|settings| settings.get("npmVersion"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
}

fn is_npm_version(value: &str) -> bool {
    value
        .chars()
        .next()
        .map(|first| first.is_ascii_digit())
        .unwrap_or(false)
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
    fn overrides_undefined_var_fallback_setting() {
        let user_settings = json!({
            "cssVariables": {
                "undefinedVarFallback": "off"
            }
        });

        let settings = build_workspace_settings(Some(user_settings));

        assert_eq!(
            settings["cssVariables"]["undefinedVarFallback"],
            json!("off")
        );
    }

    #[test]
    fn builds_cli_args_from_settings() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": ["a.css", "b.css"],
                "blacklistFolders": ["**/dist/**"],
                "undefinedVarFallback": "info"
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
                "--undefined-var-fallback",
                "info",
            ]
        );
    }

    #[test]
    fn ignores_non_array_settings_for_cli_args() {
        let user_settings = json!({
            "cssVariables": {
                "lookupFiles": "a.css",
                "blacklistFolders": 42,
                "undefinedVarFallback": 123
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
        assert!(args.contains(&"--undefined-var-fallback".to_string()));
        assert!(args.contains(&"warning".to_string()));
    }

    #[test]
    fn reads_npm_version_setting() {
        let user_settings = json!({
            "npmVersion": "beta"
        });

        let dist_tag = npm_version_from_settings(Some(&user_settings));

        assert_eq!(dist_tag, Some("beta".to_string()));
    }

    #[test]
    fn detects_npm_version_strings() {
        assert!(is_npm_version("1.0.9"));
        assert!(is_npm_version("1.0.9-beta.3"));
        assert!(!is_npm_version("beta"));
        assert!(!is_npm_version("latest"));
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
