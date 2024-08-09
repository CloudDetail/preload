use regex::Regex;

use crate::inspector::{InspectResult,JAVA_LANGUAGE};

pub fn auto_discover_service_name(res: &InspectResult) -> Option<String> {
    match res.language_type {
        JAVA_LANGUAGE => auto_discover_java_service_name(&res.original_argv),
        _ => None,
    }
}

// auto_discover_java_service_name
// java [options] -jar AAA/BBB-CCC-1.0.0.jar [args...]
//     -> bbb-ccc
// java [options] classname [args...]
//     -> classname
fn auto_discover_java_service_name(argvs: &Vec<String>) -> Option<String> {
    let mut pre_argv: &String = &String::new();
    for argv in argvs {
        if pre_argv.eq("-jar") {
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
        } else if pre_argv.eq("-m") {
            let class_index: usize;
            if let Some(pos) = argv.rfind('/') {
                class_index = pos + 1;
            } else {
                class_index = 0;
            }
            return Some(argv[class_index..].to_ascii_lowercase());
        } else if argv.starts_with("-D") || argv.starts_with("-X") {
            // 忽略以 -D 和 -X 开头的参数
            continue;
        }

        if !pre_argv.is_empty() // 不是第一个参数
            && !pre_argv.starts_with("-cp") // 前一个参数不是 -cp
            && !argv.starts_with('-')
        {
            // 本身不是配置项
            return Some(argv.to_ascii_lowercase());
        }

        pre_argv = argv
    }
    None
}
