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
// !!! java [options] -jar [options] xxx.jar [args...]
//     -> bbb-ccc
// java [options] classname [args...]
//     -> classname
fn auto_discover_java_service_name(argvs: &Vec<String>) -> Option<String> {
    let mut pre_argv: &String = &String::new();
    let mut java_cmd_start = false;
    let mut jar_option_find = false;

    for argv in argvs {
        if !java_cmd_start {
            if argv.eq("java") {
                java_cmd_start = true;
                pre_argv = argv
            }
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
        } else if pre_argv.eq("-m") || pre_argv.eq("--module")  {
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
            || pre_argv.eq("--describe-module"){
            // 其他常规带参数options
            pre_argv = argv;
            continue;
        }

        // 分析当前参数,对分析毫无影响的情况
        if argv.starts_with("-D") || argv.starts_with("-X") || argv.starts_with("@") {
            // 忽略以 -D 和 -X 开头的参数
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
        if !argv.starts_with('-') // 本身不是新配置项
        {
            return Some(argv.to_ascii_lowercase());
        }

        pre_argv = argv
    }
    None
}

