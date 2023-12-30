use std::ffi::c_void;
use windows::{
    core::{Error, HRESULT, HSTRING, PCWSTR},
    Win32::Networking::WinHttp::*
};

pub struct HInternet {
    handle: *mut c_void,
}

unsafe impl Send for HInternet {}

impl Drop for HInternet {
    // fn drop(&mut self) {
    //     if self.handle.is_null() {
    //         return;
    //     }
    //     let result = unsafe { WinHttpCloseHandle(self.handle) };
    //     if result.is_err() {
    //         let e = Error::from_win32();
    //         assert!(e.code().is_ok(), "Error: {}", e);
    //     }
    //     self.handle = std::ptr::null_mut();
    // }

    fn drop(&mut self) {
        self.close();
    }
}

impl HInternet {
    pub fn close(&mut self) {
        if self.handle.is_null() {
            return;
        }
        let result = unsafe { WinHttpCloseHandle(self.handle) };
        if result.is_err() {
            let e = Error::from_win32();
            assert!(e.code().is_ok(), "Error: {}", e);
        }
        self.handle = std::ptr::null_mut();
    }
}

pub struct HSession {
    pub h: HInternet,
}

impl HSession {
    pub fn new() -> Result<HSession, Error> {
        let hi = open_session()?;
        Ok(HSession { h: hi })
    }
}

pub struct HConnect {
    pub h: HInternet,
}

impl HConnect {
    pub fn new(hsession: &HSession, host: HSTRING, port: u16) -> Result<HConnect, Error> {
        let hi = connect(&hsession.h, host, port)?;
        Ok(HConnect { h: hi })
    }
}

pub struct HRequest {
    pub h: HInternet,
}

impl HRequest {
    pub fn new(
        hconnect: &HConnect,
        method: HSTRING,
        url_path: HSTRING,
        accept_types: Option<Vec<HSTRING>>
    ) -> Result<HRequest, Error> {
        let hi = open_request(&hconnect.h, method, url_path, accept_types)?;
        Ok(HRequest { h: hi })
    }

    pub fn set_status_callback(
        &self,
        lpfninternetcallback: WINHTTP_STATUS_CALLBACK,
        dwnotificationflags: u32,
        dwreserved: usize,
    ) -> WINHTTP_STATUS_CALLBACK {
        unsafe {
            WinHttpSetStatusCallback(
                self.h.handle,
                lpfninternetcallback,
                dwnotificationflags,
                dwreserved,
            )
        }
    }

    pub fn add_headers(&mut self, method: &str) -> Result<(), Error> {
        let content_type = match method {
            "GET" => HSTRING::from("Content-Type: text/plain"),
            "POST" => HSTRING::from("Content-Type: application/json"),
            _ => HSTRING::from("Content-Type: text/plain"),
        };
    
        unsafe {
            WinHttpAddRequestHeaders(
                self.h.handle,
                content_type.as_wide(),
                WINHTTP_ADDREQ_FLAG_ADD,
            );
        }
        Ok(())
    }

    pub fn send_req(
        &mut self,
        headers: HSTRING,
        total_length: u32,
        ctx: usize
    ) -> Result<(), Error> {
        let mut headers_op: Option<&[u16]> = None;
        if !headers.is_empty() {
            headers_op = Some(headers.as_wide());
        }

        let mut b_results = Ok(());
    
        b_results = unsafe {
            WinHttpSendRequest(
                self.h.handle,
                headers_op,
                Some(std::ptr::null()),
                0,
                total_length,
                ctx,
            )
        };
    
        if b_results.is_ok() {
            return Ok(());
        } else {
            return Err(Error::from_win32());
        }
    }

    pub fn write_data(
        &self,
        buf: &[u8],
        dwnumberofbytestowrite: u32,
        lpdwnumberofbyteswritten: Option<&mut u32>,
    ) -> Result<(), Error> {
        let len = buf.len();
        let lpdwnumberofbyteswritten_op: *mut u32 = match lpdwnumberofbyteswritten {
            Some(op) => op,
            None => std::ptr::null_mut(),
        };

        assert!(dwnumberofbytestowrite as usize <= len);

        unsafe {
            WinHttpWriteData(
                self.h.handle,
                Some(buf.as_ptr() as *const c_void),
                dwnumberofbytestowrite,
                lpdwnumberofbyteswritten_op,
            );
        };
        Ok(())
    }

    pub fn recv_resp(&mut self) -> Result<(), Error> {
        unsafe {
            WinHttpReceiveResponse(
                self.h.handle,
                std::ptr::null_mut()
            )
        }
    }

    pub fn query_data_available(&mut self, dw_size: Option<&mut u32>) -> Result<(), Error> {
        let num_of_bytes_available_op: *mut u32 = match dw_size {
            Some(op) => op,
            None => std::ptr::null_mut(),
        };

        unsafe {
            WinHttpQueryDataAvailable(
                self.h.handle,
                num_of_bytes_available_op,
            )
        }
    }

    pub fn read_data(
        &mut self,
        dw_size: u32,
        buffer: &mut [u8],
    ) -> Result<(), Error> {
        let mut lpdw_num_of_bytes_read: u32 = 0;

        let num_of_bytes_read_op: *mut u32 = match Some(&mut lpdw_num_of_bytes_read) {
            Some(op) => op,
            None => std::ptr::null_mut(),
        };

        unsafe {
            WinHttpReadData(
                self.h.handle,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                dw_size,
                num_of_bytes_read_op,
            )
        }
    }
}

fn open_session() -> Result<HInternet, Error> {
    let handle = unsafe {
        WinHttpOpen(
            &HSTRING::from("Hermit".to_string()),
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            &HSTRING::new(),
            &HSTRING::new(),
            WINHTTP_FLAG_ASYNC)
    };

    if handle.is_null() {
        return Err(Error::from_win32());
    }

    Ok(HInternet { handle })
}

fn connect(h: &HInternet, host: HSTRING, port: u16) -> Result<HInternet, Error> {
    let handle = unsafe {
        WinHttpConnect(
            h.handle,
            &host,
            port,
            0
        )
    };

    if handle.is_null() {
        return Err(Error::from_win32());
    }

    Ok(HInternet { handle })
}

fn open_request(
    h: &HInternet,
    method: HSTRING,
    url_path: HSTRING,
    accept_types: Option<Vec<HSTRING>>
) -> Result<HInternet, Error> {
    let mut at: Vec<PCWSTR> = match accept_types {
        Some(v) => {
            let mut out = v
                .into_iter()
                .map(|s| PCWSTR::from_raw(s.as_ptr()))
                .collect::<Vec<_>>();
            out.push(PCWSTR::from_raw(std::ptr::null()));
            out
        }
        None => Vec::new(),
    };

    let mut temp_ptr: *mut PCWSTR = std::ptr::null_mut();
    if !at.is_empty() {
        temp_ptr = at.as_mut_ptr();
    }
    
    let handle: *mut c_void = unsafe {
        WinHttpOpenRequest(
            h.handle,
            &method,
            &url_path,
            PCWSTR::null(),
            PCWSTR::null(),
            temp_ptr,
            WINHTTP_OPEN_REQUEST_FLAGS(0),
        )
    };

    if handle.is_null() {
        return Err(Error::from_win32());
    }

    Ok(HInternet { handle })
}