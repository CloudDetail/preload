use std::collections::HashMap;
use std::{ffi::CString, os::raw::c_char};

use crate::auto_svc_name::auto_discover_service_name;
use crate::inspector::{get_language_type_name, inspect, read_instrument_env_from_conf};

pub struct InstrumentResult {
    pub envp: *const *const c_char,
}

pub fn instrument(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
) -> Option<InstrumentResult> {
    let res = inspect(path, argv, envp)?;

    // 内部自定义变量
    let mut internal_vars: HashMap<&str, String> = HashMap::new();
    // 自动生成服务名并填充到自定义变量
    let service_name = auto_discover_service_name(&res);
    if let Some(svc_name) = service_name {
        internal_vars.insert("APO_AUTO_SERVICE_NAME", svc_name);
    }else{
        internal_vars.insert("APO_AUTO_SERVICE_NAME", get_language_type_name(res.language_type));
    }

    // 读取要注入的环境变量
    let new_env_vars = read_instrument_env_from_conf(res.language_type,internal_vars)?;

    // 将新环境变量拷贝到原始环境变量中
    let env_idx = res.original_envp.len();
    let instrumented_envp = copy_instrument_env(env_idx, envp, new_env_vars);
    Some(InstrumentResult{
        envp: instrumented_envp,
    })
}

// store_instrument_record 保存操作进程记录到文件
// fn store_instrument_record(res: &InstrumentResult) -> Result<i8,std::io::Error> {
//     let instrument_proc_dir = "/etc/apo/instrument/proc/";
//     if !std::path::Path::new(instrument_proc_dir).exists() {
//         std::fs::create_dir_all(instrument_proc_dir)?;
//     }

//     let pid = std::process::id();
//     let mut file_path = instrument_proc_dir.to_string();
//     file_path.push_str(&pid.to_string());
//     std::fs::write(file_path, res.language_type.to_string())?;
//     Ok(1)
// }

fn copy_instrument_env(
    env_idx: usize,
    envp: *const *const c_char,
    new_env_vars: Vec<String>,
) -> *mut *const i8 {
    let new_envp: *mut *const c_char;
    let array = match std::alloc::Layout::array::<*const c_char>(env_idx + new_env_vars.len() + 1) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe {
        // malloc space for new envp
        new_envp = std::alloc::alloc(array) as *mut *const c_char;
        // copy old env vars to new_envp
        let mut new_envp_idx = 0;
        if !envp.is_null() {
            while !(*envp.add(new_envp_idx)).is_null() {
                std::ptr::copy_nonoverlapping(
                    envp.add(new_envp_idx),
                    new_envp.add(new_envp_idx),
                    1,
                );
                new_envp_idx += 1;
            }
        }
        let last_envp_idx = new_envp_idx;
        // copy new env vars to new_envp
        for env_var in new_env_vars.iter() {
            let env_var_c_str = CString::new(env_var.as_str()).expect("unexpected config");
            let env_var_ptr: *const c_char = env_var_c_str.into_raw();
            std::ptr::copy_nonoverlapping(&env_var_ptr, new_envp.add(new_envp_idx), 1);
            new_envp_idx = new_envp_idx + 1;
        }
        // move _=$path to the end of environment
        std::ptr::swap(
            new_envp.add(new_envp_idx - 1),
            new_envp.add(last_envp_idx - 1),
        );
        // fill NULL pointer in the end of new_envp
        std::ptr::write(new_envp.add(new_envp_idx), std::ptr::null());
    }
    new_envp
}
