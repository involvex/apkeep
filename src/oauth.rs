use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use chromiumoxide::browser::{Browser, BrowserConfig};
use configparser::ini::Ini;
use futures_util::StreamExt;
use simple_error::SimpleError;

use crate::config;

pub const EMBEDDED_SETUP_URL: &str = "https://accounts.google.com/EmbeddedSetup";
pub const OAUTH_COOKIE_NAME: &str = "oauth_token";

/// The OAuth token used for the AAS token exchange starts with `oauth2_`.
pub fn is_valid_oauth_token(token: &str) -> bool {
    token.starts_with("oauth2_")
}

/// Extract an `oauth_token` value from the contents of a Netscape `cookies.txt` file.
pub fn oauth_token_from_cookies_text(text: &str) -> Option<String> {
    for line in text.lines() {
        // yt-dlp style exports prefix HttpOnly cookies with this marker.
        let line = line.strip_prefix("#HttpOnly_").unwrap_or(line);
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 7 || fields[5] != OAUTH_COOKIE_NAME {
            continue;
        }
        let value = fields[6].trim();
        if is_valid_oauth_token(value) {
            return Some(value.to_string());
        }
    }
    None
}

/// Read a Netscape `cookies.txt` file and return its `oauth_token` value.
pub fn oauth_token_from_cookies_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string(path)?;
    oauth_token_from_cookies_text(&text).ok_or_else(|| {
        Box::new(SimpleError::new(format!(
            "No OAuth token cookie found in {}. Export the cookies for accounts.google.com after logging in at {}",
            path.display(),
            EMBEDDED_SETUP_URL
        ))) as Box<dyn Error>
    })
}

fn find_in_path(names: &[&str]) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let candidate = dir.join(format!("{}.exe", name));
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

/// Locate a browser executable for the requested kind. Returns `None` to let
/// chromiumoxide fall back to its own auto-detection for the default browser.
fn browser_executable(kind: &str) -> Option<PathBuf> {
    let is_windows = cfg!(windows);
    let is_macos = cfg!(target_os = "macos");

    let mut candidates: Vec<PathBuf> = Vec::new();

    if is_windows {
        let local = env::var("LOCALAPPDATA").unwrap_or_default();
        let pf = env::var("PROGRAMFILES").unwrap_or_default();
        let pf86 = env::var("PROGRAMFILES(X86)").unwrap_or_default();
        match kind {
            "edge" => {
                if !pf86.is_empty() {
                    candidates
                        .push(Path::new(&pf86).join(r"Microsoft\Edge\Application\msedge.exe"));
                }
                if !pf.is_empty() {
                    candidates.push(Path::new(&pf).join(r"Microsoft\Edge\Application\msedge.exe"));
                }
            }
            "chromium" => {
                if !local.is_empty() {
                    candidates.push(Path::new(&local).join(r"Chromium\Application\chrome.exe"));
                }
            }
            _ => {
                if !local.is_empty() {
                    candidates
                        .push(Path::new(&local).join(r"Google\Chrome\Application\chrome.exe"));
                }
                if !pf.is_empty() {
                    candidates.push(Path::new(&pf).join(r"Google\Chrome\Application\chrome.exe"));
                }
                if !pf86.is_empty() {
                    candidates.push(Path::new(&pf86).join(r"Google\Chrome\Application\chrome.exe"));
                }
            }
        }
    } else if is_macos {
        match kind {
            "edge" => candidates.push(PathBuf::from(
                "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
            )),
            "chromium" => candidates.push(PathBuf::from(
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            )),
            _ => candidates.push(PathBuf::from(
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            )),
        }
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let names: &[&str] = match kind {
        "edge" => &["microsoft-edge", "msedge"],
        "chromium" => &["chromium", "chromium-browser"],
        _ => &["google-chrome", "google-chrome-stable", "chrome"],
    };
    find_in_path(names)
}

/// Open a browser window at Google's embedded setup page, wait for the user to
/// log in, and return the freshly issued `oauth_token` cookie.
pub async fn obtain_oauth_token_from_browser(
    kind: &str,
    timeout_secs: u64,
) -> Result<String, Box<dyn Error>> {
    eprintln!(
        "A browser window will open at {}. Log in to the Google account you want to use; apkeep will read the OAuth token from the browser session.",
        EMBEDDED_SETUP_URL
    );
    eprintln!("If a Google Terms of Service prompt appears, accept it in the browser window.");

    // A dedicated temporary profile avoids touching the user's normal browser
    // session and is removed on every exit path.
    let profile = tempfile::TempDir::new()?;

    let executable = browser_executable(kind);
    let mut config_builder = BrowserConfig::builder()
        .with_head()
        .user_data_dir(profile.path())
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .launch_timeout(Duration::from_secs(30));
    if let Some(executable) = executable {
        config_builder = config_builder.chrome_executable(executable);
    }
    let config = config_builder
        .build()
        .map_err(|err| SimpleError::new(format!("Could not configure browser: {}", err)))?;

    let (mut browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|err| SimpleError::new(format!("Could not launch browser: {}", err)))?;

    // The CDP handler must be continuously polled for commands to get responses.
    let handler_task = tokio::spawn(async move { while handler.next().await.is_some() {} });

    let result = async {
        // Keep the page alive for the duration of the login; dropping it would
        // close the tab and stop the login flow.
        let _page = browser.new_page(EMBEDDED_SETUP_URL).await?;

        let deadline = Instant::now() + Duration::from_secs(timeout_secs);
        loop {
            let cookies = browser.get_cookies().await.unwrap_or_default();
            if let Some(cookie) = cookies.iter().find(|cookie| {
                cookie.name == OAUTH_COOKIE_NAME && is_valid_oauth_token(&cookie.value)
            }) {
                break Ok(cookie.value.clone());
            }
            if Instant::now() >= deadline {
                break Err(Box::new(SimpleError::new(format!(
                    "Timed out after {} seconds waiting for the OAuth token. Make sure you completed the login in the browser window (and accepted the Terms of Service if prompted), then try again.",
                    timeout_secs
                ))) as Box<dyn Error>);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
    .await;

    let _ = browser.close().await;
    handler_task.abort();

    result
}

/// Persist an email and AAS token into `apkeep.ini`, preserving other settings.
pub fn save_aas_token(
    ini_file: Option<&Path>,
    email: &str,
    aas_token: &str,
) -> Result<(), Box<dyn Error>> {
    let path = match ini_file {
        Some(path) => path.to_path_buf(),
        None => {
            let mut path = config::config_dir()?;
            path.push("apkeep.ini");
            path
        }
    };

    let mut conf = Ini::new();
    if path.is_file() {
        let contents = fs::read_to_string(&path)?;
        conf.read(contents).map_err(|err| {
            Box::new(SimpleError::new(format!(
                "Could not parse config file {}: {}",
                path.display(),
                err
            ))) as Box<dyn Error>
        })?;
    }
    conf.setstr("google", "email", Some(email));
    conf.setstr("google", "aas_token", Some(aas_token));
    conf.write(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_oauth_token_from_cookies_text() {
        let text = concat!(
            "# Netscape HTTP Cookie File\n",
            "#HttpOnly_.accounts.google.com\tTRUE\t/\tTRUE\t0\tsid\tother-value\n",
            ".accounts.google.com\tTRUE\t/\tTRUE\t0\toauth_token\toauth2_4/example\n",
        );
        assert_eq!(
            oauth_token_from_cookies_text(text),
            Some("oauth2_4/example".to_string())
        );
    }

    #[test]
    fn ignores_unrelated_or_invalid_cookies() {
        let text = concat!(
            "# comment\n",
            ".accounts.google.com\tTRUE\t/\tTRUE\t0\tsid\tvalue\n",
            ".accounts.google.com\tTRUE\t/\tTRUE\t0\toauth_token\tnot-a-token\n",
            "malformed\tline\n",
        );
        assert_eq!(oauth_token_from_cookies_text(text), None);
        assert_eq!(oauth_token_from_cookies_text(""), None);
    }

    #[test]
    fn validates_oauth_token_prefix() {
        assert!(is_valid_oauth_token("oauth2_4/abc"));
        assert!(!is_valid_oauth_token("ya29.abc"));
        assert!(!is_valid_oauth_token(""));
    }

    #[test]
    fn saves_and_preserves_config() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("apkeep.ini");
        fs::write(
            &path,
            "[google]\nemail = old@example.com\naas_token = old\n\n[other]\nkey = value\n",
        )
        .unwrap();

        save_aas_token(Some(&path), "new@example.com", "new-token").unwrap();

        let contents = fs::read_to_string(&path).unwrap();
        let mut conf = Ini::new();
        conf.read(contents).unwrap();
        assert_eq!(
            conf.get("google", "email"),
            Some("new@example.com".to_string())
        );
        assert_eq!(
            conf.get("google", "aas_token"),
            Some("new-token".to_string())
        );
        assert_eq!(conf.get("other", "key"), Some("value".to_string()));
    }
}
