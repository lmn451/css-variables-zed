use zed_extension_api as zed;
use zed::settings::LspSettings;
use zed::serde_json::Value;

struct CssVariablesExtension;

impl zed::Extension for CssVariablesExtension {
    fn new() -> Self {
        CssVariablesExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<zed::Command> {
        if language_server_id.as_ref() != "css_variables" {
            return Err(format!("Unknown language server id: {language_server_id}"));
        }

        build_css_variables_command(worktree)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
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
                    "**/.cache",
                    "**/.DS_Store",
                    "**/.git",
                    "**/.hg",
                    "**/.next",
                    "**/.svn",
                    "**/bower_components",
                    "**/CVS",
                    "**/dist",
                    "**/node_modules",
                    "**/tests",
                    "**/tmp",
                ],
            }
        });

        if let Ok(lsp_settings) = LspSettings::for_worktree("css_variables", worktree) {
            if let Some(user_settings) = lsp_settings.settings {
                let mut merged = user_settings;

                let needs_wrap = merged.get("cssVariables").is_none()
                    && (merged.get("lookupFiles").is_some()
                        || merged.get("blacklistFolders").is_some());
                if needs_wrap {
                    merged = zed::serde_json::json!({ "cssVariables": merged });
                }

                merge_json_value(&mut settings, &merged);
            }
        }

        Ok(Some(settings))
    }
}

fn merge_json_value(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_json_value(base_value, overlay_value),
                    None => {
                        base_map.insert(key.clone(), overlay_value.clone());
                    }
                }
            }
        }
        _ => {
            *base = overlay.clone();
        }
    }
}

fn build_css_variables_command(worktree: &zed::Worktree) -> zed::Result<zed::Command> {
    let package = "css-variable-lsp";
    let version = "1.0.7";

    // Install the package if it's missing or on a different version.
    match zed::npm_package_installed_version(package)? {
        Some(installed) if installed == version => {
            // already correct version
        }
        _ => {
            zed::npm_install_package(package, version)?;
        }
    }

    let node = zed::node_binary_path()?;

    // Get the extension's working directory and construct path to bin
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Could not get current directory: {}", e))?;
    let bin_path = current_dir.join("node_modules/.bin/css-variable-lsp");

    if !bin_path.exists() {
        return Err(format!(
            "Language server bin does not exist: {:?} (current_dir: {:?})",
            bin_path, current_dir
        ));
    }

    let env = worktree.shell_env();
    Ok(zed::Command {
        command: node,
        args: vec![
            bin_path.to_string_lossy().to_string(),
            "--color-only-variables".to_string(),
            "--stdio".to_string(),
        ],
        env,
    })
}

zed::register_extension!(CssVariablesExtension);
