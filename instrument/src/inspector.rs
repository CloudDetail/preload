use ini::Ini;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::path::Path;

pub const UNKNOWN_LANGUAGE: i32 = 0;
pub const JAVA_LANGUAGE: i32 = 1;
// pub const GO_LANGUAGE: i32 = 2;
pub const PYTHON_LANGUAGE: i32 = 3;
pub const DOTNET_LANGUAGE: i32 = 4;
pub const NODEJS_LANGUAGE: i32 = 5;

const APO_INSTRUMENT_DISABLE_ENV: &str = "APO_INSTRUMENT_DISABLE";
const INSTRUMENT_CONF_PATH: &str = "/etc/apo/instrument/libapoinstrument.conf";
const BASENAME_BLACKLIST_SECTION: &str = "basename-blacklist";
const CMDLINE_BLACKLIST_SECTION: &str = "cmdline-blacklist";
const ENV_BLACKLIST_SECTION: &str = "env-blacklist";

pub struct InspectResult {
    pub language_type: i32,
    pub original_envp: Vec<String>,
    pub original_argv: Vec<String>,
}

pub fn get_language_type_name(language_type: i32) -> String {
    match language_type {
        JAVA_LANGUAGE => {
            return "java".to_string();
        }
        PYTHON_LANGUAGE => {
            return "python".to_string();
        }
        DOTNET_LANGUAGE => {
            return "dotnet".to_string();
        }
        NODEJS_LANGUAGE => {
            return "nodejs".to_string();
        }
        _ => {
            return "unknown".to_string();
        }
    }
}

pub fn read_instrument_env_from_conf(
    language_type: i32,
    internal_vars: HashMap<&str, String>,
) -> Option<Vec<String>> {
    let conf = Ini::load_from_file(INSTRUMENT_CONF_PATH);
    match conf {
        Ok(ini) => {
            let section = ini.section(Some(language_section_name(language_type)?))?;

            let mut res = vec![];
            section.iter().for_each(|(k, v)| {
                let trimmed_value = v.trim();
                if trimmed_value.starts_with("{{") && trimmed_value.ends_with("}}") {
                    let mut dynamic_value = trimmed_value
                        .trim_matches(|c| c == '{' || c == '}')
                        .to_string();
                    for (internal_key, internal_value) in &internal_vars {
                        dynamic_value = dynamic_value.replace(internal_key, internal_value)
                    }
                    res.push(format!("{}={}", k, dynamic_value))
                } else {
                    res.push(format!("{}={}", k, v))
                }
            });
            Some(res)
        }
        Err(_) => return None,
    }
}

// inspect 在解析argv和envp后,返回程序类型和envp的长度(用于后续更新envp)
pub fn inspect(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> Option<InspectResult> {
    let mut language_type = UNKNOWN_LANGUAGE;
    let mut env_idx = 0;
    let mut envps = vec![];
    let mut argvs = vec![];

    let mut executable_path = None;

    unsafe {
        if !path.is_null() {
            let c_str = CStr::from_ptr(path);
            executable_path = match c_str.to_str() {
                Ok(s) => Some(s.to_string()),
                Err(_) => return None,
            };
        }

        if !argv.is_null() {
            let mut argv_idx = 0;
            while !(*argv.add(argv_idx)).is_null() {
                let c_str = CStr::from_ptr(*argv.add(argv_idx));
                let argv_str = match c_str.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return None;
                    }
                };
                envps.push(argv_str.to_string());
                if language_type == UNKNOWN_LANGUAGE {
                    if argv_str.contains("java") {
                        language_type = JAVA_LANGUAGE;
                    } else if argv_str.contains("python") {
                        language_type = PYTHON_LANGUAGE;
                    } else if argv_str.contains("node") {
                        language_type = NODEJS_LANGUAGE;
                    }
                }
                argv_idx += 1;
            }
        }

        // check from envp
        if !envp.is_null() {
            while !(*envp.add(env_idx)).is_null() {
                let c_str = CStr::from_ptr(*envp.add(env_idx));
                let env_str = match c_str.to_str() {
                    Ok(s) => s,
                    Err(_) => {
                        return None;
                    }
                };
                argvs.push(env_str.to_string());
                if env_str.contains(APO_INSTRUMENT_DISABLE_ENV) {
                    return None;
                }
                if language_type == UNKNOWN_LANGUAGE && env_str.contains("ASPNET")
                    || env_str.contains("DOTNET")
                {
                    language_type = DOTNET_LANGUAGE;
                }
                env_idx += 1;
            }
        }
    }

    if language_type == UNKNOWN_LANGUAGE {
        return None;
    }

    let conf = load_instrument_conf()?;
    if !has_language_instrument_conf(&conf, language_type) {
        return None;
    }

    if is_blacklisted_in_conf(&conf, executable_path.as_deref(), &envps, &argvs) {
        return None;
    }

    Some(InspectResult {
        language_type: language_type,
        original_envp: argvs,
        original_argv: envps,
    })
}

fn load_instrument_conf() -> Option<Ini> {
    Ini::load_from_file(INSTRUMENT_CONF_PATH).ok()
}

fn has_language_instrument_conf(ini: &Ini, language_type: i32) -> bool {
    language_section_name(language_type)
        .and_then(|section| ini.section(Some(section)))
        .is_some()
}

fn language_section_name(language_type: i32) -> Option<&'static str> {
    match language_type {
        JAVA_LANGUAGE => Some("java"),
        PYTHON_LANGUAGE => Some("python"),
        DOTNET_LANGUAGE => Some("dotnet"),
        NODEJS_LANGUAGE => Some("nodejs"),
        _ => None,
    }
}

fn is_blacklisted_in_conf(
    ini: &Ini,
    executable_path: Option<&str>,
    argvs: &[String],
    envps: &[String],
) -> bool {
    basename_is_blacklisted(ini, executable_path, argvs)
        || cmdline_is_blacklisted(ini, argvs)
        || env_is_blacklisted(ini, envps)
}

fn basename_is_blacklisted(ini: &Ini, executable_path: Option<&str>, argvs: &[String]) -> bool {
    let section = match ini.section(Some(BASENAME_BLACKLIST_SECTION)) {
        Some(section) => section,
        None => return false,
    };
    let full = match section.get("full") {
        Some(full) => full,
        None => return false,
    };

    let command = normalized_full_command(executable_path, argvs);
    parse_list_value(full)
        .iter()
        .any(|entry| command.as_deref() == Some(entry.as_str()))
}

fn cmdline_is_blacklisted(ini: &Ini, argvs: &[String]) -> bool {
    let section = match ini.section(Some(CMDLINE_BLACKLIST_SECTION)) {
        Some(section) => section,
        None => return false,
    };
    let include = match section.get("include") {
        Some(include) => include,
        None => return false,
    };
    let cmdline = argvs.join(" ");
    parse_list_value(include)
        .iter()
        .any(|pattern| cmdline.contains(pattern))
}

fn env_is_blacklisted(ini: &Ini, envps: &[String]) -> bool {
    let section = match ini.section(Some(ENV_BLACKLIST_SECTION)) {
        Some(section) => section,
        None => return false,
    };

    section.iter().any(|(key, value)| {
        let key = key.trim();
        let value = value.trim();
        if value.is_empty() {
            envps.iter().any(|env| {
                env.split_once('=')
                    .map(|(env_key, _)| env_key == key)
                    .unwrap_or(false)
            })
        } else {
            let expected = format!("{}={}", key, value);
            envps.iter().any(|env| env == &expected)
        }
    })
}

fn split_entries(value: &str) -> Vec<&str> {
    value
        .split(|c| c == ',' || c == ';' || c == '\n')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_list_value(value: &str) -> Vec<String> {
    let value = value.trim();
    let value = value
        .strip_prefix('[')
        .and_then(|item| item.strip_suffix(']'))
        .unwrap_or(value);

    split_entries(value)
        .into_iter()
        .map(|item| item.trim_matches(|c| c == '\'' || c == '"').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn normalized_full_command(executable_path: Option<&str>, argvs: &[String]) -> Option<String> {
    let argv0 = argvs.first()?;
    let command = executable_path.unwrap_or(argv0);
    let mut normalized = vec![basename(command).to_string()];
    normalized.extend(argvs.iter().skip(1).cloned());
    Some(normalized.join(" "))
}

fn basename(path: &str) -> &str {
    Path::new(path)
        .file_name()
        .and_then(|filename| filename.to_str())
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn basename_blacklist_conf(entry: &str) -> String {
        format!("[basename-blacklist]\nfull=['{}']\n", entry)
    }

    fn ini_from_conf(conf: &str) -> Ini {
        Ini::load_from_str(conf).unwrap()
    }

    #[test]
    fn basename_blacklist_matches_normalized_full_command_from_argv0() {
        let conf = basename_blacklist_conf("java -version");
        let argvs = vec!["/usr/bin/java".to_string(), "-version".to_string()];
        let envps = vec![];

        let ini = ini_from_conf(&conf);

        assert!(is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn basename_blacklist_matches_normalized_full_command_from_execve_path() {
        let conf = basename_blacklist_conf("python -m app");
        let argvs = vec!["python".to_string(), "-m".to_string(), "app".to_string()];
        let envps = vec![];

        let ini = ini_from_conf(&conf);

        assert!(is_blacklisted_in_conf(
            &ini,
            Some("/opt/app/bin/python"),
            &argvs,
            &envps
        ));
    }

    #[test]
    fn basename_blacklist_does_not_match_partial_command() {
        let conf = basename_blacklist_conf("java");
        let argvs = vec![
            "/usr/bin/java".to_string(),
            "-jar".to_string(),
            "app.jar".to_string(),
        ];
        let envps = vec![];

        let ini = ini_from_conf(&conf);

        assert!(!is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn basename_blacklist_does_not_match_unlisted_program() {
        let conf = basename_blacklist_conf("java -version");
        let argvs = vec!["/usr/bin/python".to_string()];
        let envps = vec![];

        let ini = ini_from_conf(&conf);

        assert!(!is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn cmdline_blacklist_matches_include_pattern() {
        let conf = "[cmdline-blacklist]\ninclude=['-jar admin.jar','--skip-instrument']\n";
        let argvs = vec![
            "/usr/bin/java".to_string(),
            "-jar".to_string(),
            "admin.jar".to_string(),
        ];
        let envps = vec![];

        let ini = ini_from_conf(conf);

        assert!(is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn language_instrument_conf_requires_matching_language_section() {
        let conf = "\
[basename-blacklist]
full=['java -version']

[python]
PYTHONPATH=/tmp
";
        let ini = ini_from_conf(conf);

        assert!(!has_language_instrument_conf(&ini, JAVA_LANGUAGE));
        assert!(has_language_instrument_conf(&ini, PYTHON_LANGUAGE));
    }

    #[test]
    fn env_blacklist_matches_env_key() {
        let conf = "[env-blacklist]\nAPO_SKIP_SERVICE=\n";
        let argvs = vec![];
        let envps = vec!["APO_SKIP_SERVICE=1".to_string()];

        let ini = ini_from_conf(conf);

        assert!(is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn env_blacklist_matches_exact_env_value() {
        let conf = "[env-blacklist]\nAPP_NAME=admin-api\n";
        let argvs = vec![];
        let envps = vec!["APP_NAME=admin-api".to_string()];

        let ini = ini_from_conf(conf);

        assert!(is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn env_blacklist_does_not_match_different_env_value() {
        let conf = "[env-blacklist]\nAPP_NAME=admin-api\n";
        let argvs = vec![];
        let envps = vec!["APP_NAME=public-api".to_string()];

        let ini = ini_from_conf(conf);

        assert!(!is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }

    #[test]
    fn blacklist_config_supports_ini_list_entries() {
        let conf = "\
[basename-blacklist]
full=['python']

[env-blacklist]
APO_SKIP_SERVICE=
APP_NAME=admin-api
";
        let argvs = vec!["/usr/bin/python".to_string()];
        let envps = vec!["APO_SKIP_SERVICE=1".to_string()];

        let ini = ini_from_conf(conf);

        assert!(is_blacklisted_in_conf(&ini, None, &argvs, &envps));
    }
}
