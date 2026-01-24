use std::fs;
use zed::serde_json::Value;
use zed::settings::LspSettings;
use zed_extension_api as zed;

struct CssVariablesExtension;

impl CssVariablesExtension {
    fn language_server_binary_path(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> zed::Result<String> {
        let release = zed::latest_github_release(
            "lmn451/css-lsp-rust",
            zed::GithubReleaseOptions {
                pre_release: false,
                require_assets: true,
            },
        )?;
        let (os, arch) = zed::current_platform();
        let os_name = match os {
            zed::Os::Mac => "macos",
            zed::Os::Linux => "linux",
            zed::Os::Windows => "windows",
        };
        let arch_name = match arch {
            zed::Architecture::Aarch64 => "aarch64",
            zed::Architecture::X8664 => "x86_64",
            _ => return Err(format!("unsupported architecture: {:?}", arch)),
        };
        let suffix = if os == zed::Os::Windows {
            "exe.zip"
        } else {
            "tar.gz"
        };
        let asset_name = format!("css-variable-lsp-{os_name}-{arch_name}.{suffix}");

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {}", asset_name))?;

        let version_dir = format!("css-lsp-rust-{}", release.version);
        let binary_path = format!(
            "{version_dir}/css-variable-lsp{}",
            if os == zed::Os::Windows { ".exe" } else { "" }
        );

        if !fs::metadata(&binary_path).map_or(false, |m| m.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::download_file(
                &asset.download_url,
                &version_dir,
                zed::DownloadedFileType::Uncompressed,
            )?;

            zed::make_file_executable(&binary_path)?;

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory: {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to read directory entry: {e}"))?;
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("css-lsp-rust-") && file_name != version_dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        Ok(binary_path)
    }
}

impl zed::Extension for CssVariablesExtension {
    fn new() -> Self {
        CssVariablesExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != "css-variables" {
            return Err(format!("Unknown language server id: {language_server_id}"));
        }

        let user_settings = LspSettings::for_worktree("css-variables", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);

        match self.language_server_binary_path(language_server_id, worktree) {
            Ok(path) => {
                let env = worktree.shell_env();
                let merged_settings = build_workspace_settings(user_settings);
                let args = build_css_variables_args(Some(merged_settings));

                Ok(zed::Command {
                    command: path,
                    args,
                    env,
                })
            }
            Err(_) => build_css_variables_command(worktree, user_settings),
        }
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

fn build_css_variables_command(
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

fn extract_string_array(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(|value| value.to_string()))
            .collect(),
        _ => Vec::new(),
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
}
