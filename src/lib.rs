use zed_extension_api as zed;

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
}

fn build_css_variables_command(worktree: &zed::Worktree) -> zed::Result<zed::Command> {
    // Prefer a globally installed server: npm i -g css-variables-language-server
    let server_path = worktree
        .which("css-variables-language-server")
        .ok_or_else(|| {
            "css-variables-language-server not found in PATH. Install it with: npm i -g css-variables-language-server"
                .to_string()
        })?;

    // Start with the worktree's shell environment so PATH and other vars are inherited.
    let mut env = worktree.shell_env();

    // Ensure common Homebrew/npm locations are included when Zed is launched from the GUI.
    // Env vars are Vec<(String, String)>; tweak PATH if present, otherwise add it.
    let mut has_path = false;
    for (key, value) in env.iter_mut() {
        if key == "PATH" {
            *value = format!("/opt/homebrew/bin:/usr/local/bin:{value}");
            has_path = true;
            break;
        }
    }
    if !has_path {
        env.push((
            "PATH".to_string(),
            "/opt/homebrew/bin:/usr/local/bin".to_string(),
        ));
    }

    Ok(zed::Command {
        command: server_path,
        args: vec!["--stdio".to_string()],
        env,
    })
}

zed::register_extension!(CssVariablesExtension);
