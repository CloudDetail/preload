use std::os::raw::{c_char, c_int};

mod instrument;
use instrument::instrument;

mod inspector;
mod auto_svc_name;

#[no_mangle]
pub extern "C" fn apo_execve(
    path: *const c_char,
    argv: *const *const c_char,
    envp: *const *const c_char,
    execve_shim: extern "C" fn(
        path: *const c_char,
        argv: *const *const c_char,
        envp: *const *const c_char,
    ) -> c_int,
) -> c_int {
    let res;
    if let Some(instrument_res) = instrument(path, argv, envp){
        res = execve_shim(path, argv, instrument_res.envp);
    }else{
        res = execve_shim(path, argv, envp);
    }

    return res;
}
