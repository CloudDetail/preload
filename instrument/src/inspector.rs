use ini::Ini;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::collections::HashMap;

pub const UNKNOWN_LANGUAGE: i32 = 0;
pub const JAVA_LANGUAGE: i32 = 1;
// pub const GO_LANGUAGE: i32 = 2;
pub const PYTHON_LANGUAGE: i32 = 3;
pub const DOTNET_LANGUAGE: i32 = 4;
pub const NODEJS_LANGUAGE: i32 = 5;

const APO_INSTRUMENT_DISABLE_ENV: &str = "APO_INSTRUMENT_DISABLE";

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
    let conf = Ini::load_from_file("/etc/apo/instrument/libapoinstrument.conf");
    match conf {
        Ok(ini) => {
            let section;
            match language_type {
                JAVA_LANGUAGE => {
                    section = ini.section(Some("java"))?;
                }
                PYTHON_LANGUAGE => {
                    section = ini.section(Some("python"))?;
                }
                DOTNET_LANGUAGE => {
                    section = ini.section(Some("dotnet"))?;
                }
                NODEJS_LANGUAGE => {
                    section = ini.section(Some("nodejs"))?;
                }
                _ => return None,
            }

            let mut res = vec![];
            section.iter().for_each(|(k, v)| {
                let trimmed_value = v.trim();
                if trimmed_value.starts_with("{{") && trimmed_value.ends_with("}}") {
                    let mut dynamic_value = trimmed_value.trim_matches(|c| c == '{' || c == '}').to_string();
                    for (internal_key,internal_value) in &internal_vars  {
                        dynamic_value = dynamic_value.replace(internal_key, internal_value)
                    }
                    res.push(format!("{}={}", k, v))
                }else{
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
    _path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> Option<InspectResult> {
    let mut language_type = UNKNOWN_LANGUAGE;
    let mut env_idx = 0;
    let mut envps = vec![];
    let mut argvs = vec![];

    unsafe {
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

    Some(InspectResult {
        language_type: language_type,
        original_envp: argvs,
        original_argv: envps,
    })
}
