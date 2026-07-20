use ini::Ini;
use regex::Regex;

use crate::inspector::{InspectResult, JAVA_LANGUAGE};

const INSTRUMENT_CONF_PATH: &str = "/etc/apo/instrument/libapoinstrument.conf";
const JAVA_SERVICE_NAME_SECTION: &str = "java-service-name";
const SERVICE_NAME_PATTERN_KEY: &str = "from";
const SERVICE_NAME_PLACEHOLDER: &str = "${service_name}";

pub fn auto_discover_service_name(res: &InspectResult) -> Option<String> {
    match res.language_type {
        JAVA_LANGUAGE => auto_discover_java_service_name(&res.original_argv),
        _ => None,
    }
}

// auto_discover_java_service_name
// java [options] -jar AAA/BBB-CCC-1.0.0.jar [args...]
// !!! java [options] -jar [options] xxx.jar [args...]
//     -> bbb-ccc
// java [options] classname [args...]
//     -> classname
fn auto_discover_java_service_name(argvs: &Vec<String>) -> Option<String> {
    if let Some(service_name) = discover_java_service_name_from_conf(argvs) {
        return Some(service_name);
    }

    let mut pre_argv: &String = &String::new();
    let mut java_cmd_start = false;
    let mut jar_option_find = false;

    for argv in argvs {
        if !java_cmd_start {
            if argv.eq("java") || argv.ends_with("/java") {
                java_cmd_start = true;
                pre_argv = argv
            }
            continue;
        }

        // 分析当前参数,对分析毫无影响的情况
        if argv.starts_with("-D") || argv.starts_with("-X") || argv.starts_with("@") {
            // 忽略以 -D 和 -X 开头的参数
            continue;
        }

        // 通过前一个参数,就能确定当前参数的内容的情况
        if jar_option_find && argv.ends_with(".jar") {
            // AAA/BBB-CCC-1.0.0.jar -> bbb-ccc
            let version_pattern = r"-\d+(\.\d+)+(-SNAPSHOT)?";
            let re = Regex::new(version_pattern).unwrap();
            let filename_index: usize;
            if let Some(pos) = argv.rfind('/') {
                filename_index = pos + 1;
            } else {
                filename_index = 0;
            }
            let jar_index = argv.find(".jar").unwrap_or(argv.len());
            return Some(
                re.replace(&argv[filename_index..jar_index], "")
                    .to_ascii_lowercase(),
            );
        } else if pre_argv.eq("-m") || pre_argv.eq("--module") {
            let class_index: usize;
            if let Some(pos) = argv.rfind('/') {
                class_index = pos + 1;
            } else {
                class_index = 0;
            }
            return Some(argv[class_index..].to_ascii_lowercase());
        } else if pre_argv.eq("-cp")
            || pre_argv.eq("-classpath")
            || pre_argv.eq("--class-path")
            || pre_argv.eq("-p")
            || pre_argv.eq("--module-path")
            || pre_argv.eq("--upgrade-module-path")
            || pre_argv.eq("--add-modules")
            || pre_argv.eq("--enable-native-access")
            || pre_argv.eq("--describe-module")
        {
            // 其他常规带参数options
            pre_argv = argv;
            continue;
        }

        if argv.eq("-jar") {
            jar_option_find = true;
            pre_argv = argv;
            continue;
        }

        // 对于Java Command;
        // java [options] -jar XXXX.jar [args]
        // java [options] classname [args]
        // options阶段
        if !argv.starts_with('-')
        // 本身不是新配置项
        {
            return Some(argv.to_ascii_lowercase());
        }

        pre_argv = argv
    }
    None
}

fn discover_java_service_name_from_conf(argvs: &[String]) -> Option<String> {
    let conf = Ini::load_from_file(INSTRUMENT_CONF_PATH).ok()?;
    let section = conf.section(Some(JAVA_SERVICE_NAME_SECTION))?;
    let pattern = section.get(SERVICE_NAME_PATTERN_KEY)?;
    discover_java_service_name_from_patterns(argvs, &[pattern])
}

fn discover_java_service_name_from_patterns(argvs: &[String], patterns: &[&str]) -> Option<String> {
    patterns
        .iter()
        .flat_map(|pattern| split_pattern_entries(pattern))
        .find_map(|pattern| discover_java_service_name_from_pattern(argvs, pattern))
}

fn split_pattern_entries(pattern: &str) -> Vec<&str> {
    let pattern = pattern.trim();
    let pattern = pattern
        .strip_prefix('[')
        .and_then(|item| item.strip_suffix(']'))
        .unwrap_or(pattern);

    pattern
        .split(|c| c == ',' || c == ';' || c == '\n')
        .map(|item| item.trim().trim_matches(|c| c == '\'' || c == '"'))
        .filter(|item| !item.is_empty())
        .collect()
}

fn discover_java_service_name_from_pattern(argvs: &[String], pattern: &str) -> Option<String> {
    let placeholder_index = pattern.find(SERVICE_NAME_PLACEHOLDER)?;
    let prefix = &pattern[..placeholder_index];
    let suffix = &pattern[placeholder_index + SERVICE_NAME_PLACEHOLDER.len()..];

    argvs.iter().find_map(|argv| {
        if !argv.starts_with(prefix) || !argv.ends_with(suffix) {
            return None;
        }
        let value_end = argv.len().saturating_sub(suffix.len());
        if prefix.len() > value_end {
            return None;
        }
        let value = &argv[prefix.len()..value_end];
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argvs(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn configured_service_name_pattern_has_highest_priority() {
        let args = argvs(&[
            "java",
            "-Dapp=service1",
            "-jar",
            "/opt/requesttemplate-demo-0.0.1-SNAPSHOT.jar",
        ]);

        assert_eq!(
            discover_java_service_name_from_patterns(&args, &["-Dapp=${service_name}"]),
            Some("service1".to_string())
        );
    }

    #[test]
    fn configured_service_name_pattern_supports_suffix() {
        let args = argvs(&[
            "java",
            "-Dapp.name=service1-prod",
            "com.example.Application",
        ]);

        assert_eq!(
            discover_java_service_name_from_patterns(&args, &["-Dapp.name=${service_name}-prod"]),
            Some("service1".to_string())
        );
    }

    #[test]
    fn configured_service_name_pattern_supports_ini_list() {
        let args = argvs(&[
            "java",
            "-Dspring.application.name=svc",
            "com.example.Application",
        ]);

        assert_eq!(
            discover_java_service_name_from_patterns(
                &args,
                &["['-Dapp=${service_name}','-Dspring.application.name=${service_name}']"]
            ),
            Some("svc".to_string())
        );
    }

    #[test]
    fn configured_service_name_pattern_falls_back_when_missing() {
        let args = argvs(&[
            "java",
            "-jar",
            "/opt/requesttemplate-demo-0.0.1-SNAPSHOT.jar",
        ]);

        assert_eq!(
            discover_java_service_name_from_patterns(&args, &["-Dapp=${service_name}"]),
            None
        );
        assert_eq!(
            auto_discover_java_service_name(&args),
            Some("requesttemplate-demo".to_string())
        );
    }
}
