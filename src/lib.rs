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

        let user_settings = LspSettings::for_worktree("css_variables", worktree)
            .ok()
            .and_then(|lsp_settings| lsp_settings.settings);

        build_css_variables_command(worktree, user_settings)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> zed::Result<Option<zed::serde_json::Value>> {
        if let Ok(lsp_settings) = LspSettings::for_worktree("css_variables", worktree) {
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
    let version = "1.0.8";

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
    let mut args = vec![bin_path.to_string_lossy().to_string()];
    args.extend(build_css_variables_args(user_settings));

    Ok(zed::Command {
        command: node,
        args,
        env,
    })
}

fn build_css_variables_args(user_settings: Option<Value>) -> Vec<String> {
    let mut args = vec![
        "--color-only-variables".to_string(),
        "--stdio".to_string(),
    ];

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
}
