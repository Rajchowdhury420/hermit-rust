use std::ffi::c_void;
use std::mem;
use windows::{
    core::*,
    Win32::{
        Networking::WinHttp::*,
        System::{Com::*, Memory::*, Threading::*},
    }
};

pub fn run(proto: &str, host: &str, port: u16) -> Result<()> {
    unsafe {
        // Reference:
        // https://github.com/Steve-xmh/alhc/blob/main/src/windows/mod.rs

        println!("{proto}://{host}:{port}/");

        let mut lpsz_host = HSTRING::from(host);
        // let mut pswz_host = PCWS

        let method = "GET";
        let mut lpsz_method = HSTRING::from(method);

        let mut h_session: *mut c_void = std::ptr::null_mut();
        let mut h_connect: *mut c_void = std::ptr::null_mut();
        let mut h_request: *mut c_void = std::ptr::null_mut();
        let mut b_results = Ok(());

        let mut dw_size: usize = 0;
        let mut dw_downloaded: usize = 0;
        let mut out_buffer = Vec::new();
        let mut psz_out_buffer: *mut c_void = &mut out_buffer as *mut _ as *mut c_void;

        let h_session = WinHttpOpen(
            PCWSTR::null(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            PCWSTR::null(),
            PCWSTR::null(),
            WINHTTP_FLAG_ASYNC);

        if !h_session.is_null() {
            h_connect = WinHttpConnect(h_session, &lpsz_host, port, 0);
        }

        if !h_connect.is_null() {
            h_request = WinHttpOpenRequest(
                h_connect,
                &lpsz_method,
                PCWSTR::null(),
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null_mut(),
                WINHTTP_FLAG_SECURE);
        }

        if !h_request.is_null() {
            b_results = WinHttpSendRequest(
                h_request,
                Some(&[0]),
                Some(std::ptr::null()),
                0,
                0,
                0);
        }

        if b_results.is_ok() {
            b_results = WinHttpReceiveResponse(h_request, std::ptr::null_mut());
        }

        match b_results {
            Ok(_) => {
                loop {
                    dw_size = 0;
    
                    if let Err(e) = WinHttpQueryDataAvailable(h_request, std::ptr::null_mut()) {
                        println!("Error: {e}");
                    }
    
                    out_buffer = vec![0; dw_size + 1];
                    if psz_out_buffer.is_null() {
                        println!("Out of memory.");
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
                println!("Error: {e}");
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
    
        println!("Done");
    }


    Ok(())
}
