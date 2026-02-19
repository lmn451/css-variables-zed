use std::fs;
use std::path::{Path, PathBuf};

use zed::serde_json::Value;
use zed::settings::{CommandSettings, LspSettings};
use zed_extension_api as zed;

const CSS_VARIABLES_BINARY_NAME: &str = "css-variable-lsp";
const CSS_VARIABLES_NPM_PACKAGE: &str = "css-variable-lsp";
const CSS_VARIABLES_RELEASE_REPO: &str = "lmn451/css-lsp-rust";
const CSS_VARIABLES_CACHE_PREFIX: &str = "css-variable-lsp-";

#[derive(Clone, Debug, PartialEq, Eq)]
enum PathBinaryTarget {
    MatchVersion(String),
    // npm latest lookup failed (offline/network issue) - allow PATH if it has a readable version.
    AllowWhenVersionReadable,
    // Explicit dist-tags (e.g. beta) should use npm fallback instead of PATH.
    Reject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ParsedSemver {
    major: u64,
    minor: u64,
    patch: u64,
    pre_release: Option<String>,
}

#[derive(Clone, Debug)]
struct CachedDirCandidate {
    name: String,
    path: PathBuf,
    semver: Option<ParsedSemver>,
}

struct CssVariablesExtension {
    cached_binary_path: Option<String>,
}

impl CssVariablesExtension {
    fn resolve_css_variables_binary(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        _worktree: &zed::Worktree,
        _user_settings: Option<&Value>,
        binary_settings: Option<&CommandSettings>,
    ) -> zed::Result<String> {
        if let Some(path) = binary_settings.and_then(|settings| settings.path.as_ref()) {
            return Ok(path.clone());
        }

        let (platform, arch) = zed::current_platform();
        let binary_name = binary_name_for_platform(platform);
        match self.resolve_latest_rust_binary(language_server_id, platform, arch, binary_name) {
            Ok(path) => {
                self.cached_binary_path = Some(path.clone());
                Ok(path)
            }
            Err(latest_err) => {
                if let Some(path) = self.valid_cached_binary_path() {
                    self.cached_binary_path = Some(path.clone());
                    return Ok(path);
                }

                if let Some(path) = find_any_cached_binary(CSS_VARIABLES_CACHE_PREFIX, binary_name)?
                {
                    self.cached_binary_path = Some(path.clone());
                    return Ok(path);
                }

                Err(latest_err)
            }
        }
    }

    fn resolve_latest_rust_binary(
        &self,
        language_server_id: &zed::LanguageServerId,
        platform: zed::Os,
        arch: zed::Architecture,
        binary_name: &str,
    ) -> zed::Result<String> {
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            CSS_VARIABLES_RELEASE_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let asset_name = asset_name_for_platform(platform, arch)?;
        let version_dir = cache_dir_for_release_version(&release.version);

        if let Some(path) = find_binary_in_dir(&version_dir, binary_name)? {
            return Ok(path);
        }

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "latest release '{}' missing expected asset '{}'",
                    release.version, asset_name
                )
            })?;

        let (download_type, is_archive) = download_type_for_asset(asset_name);
        fs::create_dir_all(&version_dir)
            .map_err(|err| format!("failed to create directory '{version_dir}': {err}"))?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Downloading,
        );

        let binary_path = if is_archive {
            zed::download_file(&asset.download_url, &version_dir, download_type)
                .map_err(|err| format!("failed to download {asset_name}: {err}"))?;

            find_binary_in_dir(&version_dir, binary_name)?.ok_or_else(|| {
                format!("downloaded archive did not contain expected binary '{binary_name}'")
            })?
        } else {
            let binary_path = format!("{version_dir}/{binary_name}");
            if !Path::new(&binary_path).exists() {
                zed::download_file(&asset.download_url, &binary_path, download_type)
                    .map_err(|err| format!("failed to download {asset_name}: {err}"))?;
            }
            binary_path
        };

        if platform != zed::Os::Windows {
            zed::make_file_executable(&binary_path)?;
        }

        prune_cached_versions(CSS_VARIABLES_CACHE_PREFIX, &version_dir);
        Ok(binary_path)
    }

    fn valid_cached_binary_path(&self) -> Option<String> {
        self.cached_binary_path.as_ref().and_then(|path| {
            fs::metadata(path)
                .ok()
                .filter(|stat| stat.is_file())
                .map(|_| path.clone())
        })
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

        // Try Rust binary first, then PATH if it's current, then fall back to npm
        let command = match self.resolve_css_variables_binary(
            language_server_id,
            worktree,
            user_settings.as_ref(),
            binary_settings,
        ) {
            Ok(path) => path,
            Err(_rust_err) => {
                // 4) Check PATH before npm fallback (user's own install)
                if let Some(path) = worktree.which(CSS_VARIABLES_BINARY_NAME) {
                    if should_use_path_binary(&path, user_settings.as_ref()) {
                        path
                    } else {
                        // 5) npm fallback (and update)
                        return build_npm_fallback_command(worktree, user_settings);
                    }
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
    let package = CSS_VARIABLES_NPM_PACKAGE;
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

fn should_use_path_binary(path: &str, user_settings: Option<&Value>) -> bool {
    let Some(path_version) = read_binary_version(path) else {
        return false;
    };
    let target = expected_path_binary_target(user_settings);
    should_use_path_binary_for_target(&path_version, &target)
}

fn should_use_path_binary_for_target(path_version: &str, target: &PathBinaryTarget) -> bool {
    match target {
        PathBinaryTarget::MatchVersion(target_version) => {
            normalize_semver_token(path_version) == normalize_semver_token(target_version)
        }
        PathBinaryTarget::AllowWhenVersionReadable => true,
        PathBinaryTarget::Reject => false,
    }
}

fn expected_path_binary_target(user_settings: Option<&Value>) -> PathBinaryTarget {
    let npm_version =
        npm_version_from_settings(user_settings).unwrap_or_else(|| "latest".to_string());
    if npm_version == "latest" {
        match zed::npm_package_latest_version(CSS_VARIABLES_NPM_PACKAGE) {
            Ok(version) => PathBinaryTarget::MatchVersion(version),
            Err(_) => PathBinaryTarget::AllowWhenVersionReadable,
        }
    } else if is_npm_version(&npm_version) {
        PathBinaryTarget::MatchVersion(npm_version)
    } else {
        // Dist tags (e.g. beta) are treated as stale for PATH checks.
        PathBinaryTarget::Reject
    }
}

fn read_binary_version(path: &str) -> Option<String> {
    let candidates = ["--version", "-V"];
    for arg in candidates {
        if let Some(version) = read_binary_version_with_arg(path, arg) {
            return Some(version);
        }
    }
    None
}

fn read_binary_version_with_arg(path: &str, arg: &str) -> Option<String> {
    let mut command = zed::process::Command::new(path.to_string()).arg(arg);
    let output = command.output().ok()?;
    if output.status != Some(0) {
        return None;
    }

    let mut combined = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if combined.is_empty() {
        combined = stderr;
    } else if !stderr.is_empty() {
        combined.push('\n');
        combined.push_str(&stderr);
    }

    extract_semver_token(&combined)
}

fn extract_semver_token(output: &str) -> Option<String> {
    let bytes = output.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }

        let start = i;
        i = consume_digits(bytes, i);
        if i >= bytes.len() || bytes[i] != b'.' {
            continue;
        }

        i += 1;
        let minor_start = i;
        i = consume_digits(bytes, i);
        if i == minor_start || i >= bytes.len() || bytes[i] != b'.' {
            continue;
        }

        i += 1;
        let patch_start = i;
        i = consume_digits(bytes, i);
        if i == patch_start {
            continue;
        }

        while i < bytes.len() && is_semver_tail_byte(bytes[i]) {
            i += 1;
        }

        let token = output[start..i].to_string();
        return Some(token);
    }

    None
}

fn consume_digits(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    index
}

fn is_semver_tail_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+')
}

fn normalize_semver_token(version: &str) -> String {
    version
        .trim()
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_string()
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

fn cache_dir_for_release_version(version: &str) -> String {
    let normalized = normalize_semver_token(version);
    let sanitized = normalized
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    if sanitized.is_empty() {
        format!("{CSS_VARIABLES_CACHE_PREFIX}unknown")
    } else {
        format!("{CSS_VARIABLES_CACHE_PREFIX}{sanitized}")
    }
}

fn parse_semver_from_cache_dir_name(prefix: &str, name: &str) -> Option<ParsedSemver> {
    let suffix = name.strip_prefix(prefix)?;
    parse_semver_token(suffix)
}

fn parse_semver_token(version: &str) -> Option<ParsedSemver> {
    let normalized = normalize_semver_token(version);
    let without_build = normalized
        .split_once('+')
        .map(|(core, _)| core)
        .unwrap_or(normalized.as_str());
    let (core, pre_release) = without_build
        .split_once('-')
        .map(|(core, pre)| (core, Some(pre.to_string())))
        .unwrap_or((without_build, None));

    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    let patch = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    Some(ParsedSemver {
        major,
        minor,
        patch,
        pre_release,
    })
}

fn compare_semver(a: &ParsedSemver, b: &ParsedSemver) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match a.major.cmp(&b.major) {
        Ordering::Equal => {}
        non_eq => return non_eq,
    }
    match a.minor.cmp(&b.minor) {
        Ordering::Equal => {}
        non_eq => return non_eq,
    }
    match a.patch.cmp(&b.patch) {
        Ordering::Equal => {}
        non_eq => return non_eq,
    }

    match (&a.pre_release, &b.pre_release) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a_pre), Some(b_pre)) => compare_pre_release(a_pre, b_pre),
    }
}

fn compare_pre_release(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();
    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        match (a_parts.get(i), b_parts.get(i)) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(a_part), Some(b_part)) => {
                let a_num = a_part.parse::<u64>().ok();
                let b_num = b_part.parse::<u64>().ok();

                let part_cmp = match (a_num, b_num) {
                    (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => a_part.cmp(b_part),
                };

                if part_cmp != Ordering::Equal {
                    return part_cmp;
                }
            }
        }
    }

    Ordering::Equal
}

fn compare_cached_dir_candidates(
    a: &CachedDirCandidate,
    b: &CachedDirCandidate,
) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (&a.semver, &b.semver) {
        (Some(a_semver), Some(b_semver)) => {
            let semver_cmp = compare_semver(b_semver, a_semver);
            if semver_cmp == Ordering::Equal {
                b.name.cmp(&a.name)
            } else {
                semver_cmp
            }
        }
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => b.name.cmp(&a.name),
    }
}

fn find_any_cached_binary(prefix: &str, binary_name: &str) -> zed::Result<Option<String>> {
    let entries = match fs::read_dir(".") {
        Ok(entries) => entries,
        Err(_) => return Ok(None),
    };

    let mut cached_dirs: Vec<CachedDirCandidate> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {err}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !name.starts_with(prefix) {
            continue;
        }
        let semver = parse_semver_from_cache_dir_name(prefix, &name);
        cached_dirs.push(CachedDirCandidate { name, path, semver });
    }

    cached_dirs.sort_by(compare_cached_dir_candidates);

    for candidate in cached_dirs {
        if let Some(found) = find_binary_in_tree(&candidate.path, binary_name)? {
            return Ok(Some(found));
        }
    }

    Ok(None)
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
        } else if file_type.is_file() && entry.file_name().to_str() == Some(binary_name) {
            return Ok(Some(path.to_string_lossy().to_string()));
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

    #[test]
    fn extracts_semver_from_version_output() {
        assert_eq!(
            extract_semver_token("css-variable-lsp 0.1.9").as_deref(),
            Some("0.1.9")
        );
        assert_eq!(
            extract_semver_token("v0.2.0-alpha.1").as_deref(),
            Some("0.2.0-alpha.1")
        );
    }

    #[test]
    fn rejects_invalid_semver_output() {
        assert!(extract_semver_token("css-variable-lsp unknown").is_none());
        assert!(extract_semver_token("version: 0.1").is_none());
    }

    #[test]
    fn normalizes_semver_tokens() {
        assert_eq!(normalize_semver_token("v1.2.3"), "1.2.3");
        assert_eq!(normalize_semver_token("V1.2.3-beta.1"), "1.2.3-beta.1");
    }

    #[test]
    fn compares_path_version_to_target_version() {
        assert_eq!(
            normalize_semver_token("v0.1.6"),
            normalize_semver_token("0.1.6")
        );
        assert_ne!(
            normalize_semver_token("0.1.5"),
            normalize_semver_token("0.1.6")
        );
    }

    #[test]
    fn path_target_allows_when_latest_lookup_fails() {
        let target = PathBinaryTarget::AllowWhenVersionReadable;
        assert!(should_use_path_binary_for_target("0.1.6", &target));
    }

    #[test]
    fn path_target_rejects_for_dist_tags() {
        let target = PathBinaryTarget::Reject;
        assert!(!should_use_path_binary_for_target("0.1.6", &target));
    }

    #[test]
    fn path_target_requires_exact_match_when_known() {
        let target = PathBinaryTarget::MatchVersion("0.1.6".to_string());
        assert!(should_use_path_binary_for_target("v0.1.6", &target));
        assert!(!should_use_path_binary_for_target("0.1.5", &target));
    }

    #[test]
    fn semver_compare_uses_numeric_ordering() {
        let older = parse_semver_token("0.9.0").unwrap();
        let newer = parse_semver_token("0.10.0").unwrap();
        assert_eq!(compare_semver(&newer, &older), std::cmp::Ordering::Greater);
    }

    #[test]
    fn semver_compare_prefers_stable_over_prerelease() {
        let prerelease = parse_semver_token("1.0.0-beta.1").unwrap();
        let stable = parse_semver_token("1.0.0").unwrap();
        assert_eq!(
            compare_semver(&stable, &prerelease),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn semver_compare_orders_prerelease_identifiers() {
        let alpha1 = parse_semver_token("1.0.0-alpha.1").unwrap();
        let alpha2 = parse_semver_token("1.0.0-alpha.2").unwrap();
        assert_eq!(
            compare_semver(&alpha2, &alpha1),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn cached_dir_sort_prefers_newest_semver_over_lexical_order() {
        let mut candidates = vec![
            CachedDirCandidate {
                name: "css-variable-lsp-0.9.0".to_string(),
                path: PathBuf::from("old"),
                semver: parse_semver_from_cache_dir_name(
                    CSS_VARIABLES_CACHE_PREFIX,
                    "css-variable-lsp-0.9.0",
                ),
            },
            CachedDirCandidate {
                name: "css-variable-lsp-0.10.0".to_string(),
                path: PathBuf::from("new"),
                semver: parse_semver_from_cache_dir_name(
                    CSS_VARIABLES_CACHE_PREFIX,
                    "css-variable-lsp-0.10.0",
                ),
            },
            CachedDirCandidate {
                name: "css-variable-lsp-latest".to_string(),
                path: PathBuf::from("fallback"),
                semver: parse_semver_from_cache_dir_name(
                    CSS_VARIABLES_CACHE_PREFIX,
                    "css-variable-lsp-latest",
                ),
            },
        ];

        candidates.sort_by(compare_cached_dir_candidates);

        assert_eq!(candidates[0].name, "css-variable-lsp-0.10.0");
        assert_eq!(candidates[1].name, "css-variable-lsp-0.9.0");
        assert_eq!(candidates[2].name, "css-variable-lsp-latest");
    }

    #[test]
    fn builds_cache_dir_from_release_version() {
        assert_eq!(
            cache_dir_for_release_version("v0.1.9"),
            "css-variable-lsp-0.1.9"
        );
        assert_eq!(
            cache_dir_for_release_version("release/0.2.0"),
            "css-variable-lsp-release_0.2.0"
        );
    }
}
