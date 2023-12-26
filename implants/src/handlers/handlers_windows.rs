use std::{
    ffi::c_void,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use windows::{
    core::{Error, HRESULT, HSTRING, PCWSTR},
    Win32::Networking::WinHttp::*
};

const WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS: u32 = WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE
    | WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_READ_COMPLETE
    | WINHTTP_CALLBACK_STATUS_WRITE_COMPLETE
    | WINHTTP_CALLBACK_STATUS_REQUEST_ERROR
    | WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE;

pub struct Handlers {
    pub h_session: *mut c_void,
    pub h_connect: *mut c_void,
    pub h_request: *mut c_void,
}

impl Handlers {
    pub fn new() -> Self {
        Self {
            h_session: std::ptr::null_mut(),
            h_connect: std::ptr::null_mut(),
            h_request: std::ptr::null_mut(),
        }
    }

    pub fn open_session(&mut self, agent: HSTRING) -> Result<(), Error> {
        let h_session = unsafe {
            WinHttpOpen(
                &agent,
                WINHTTP_ACCESS_TYPE_NO_PROXY,
                &HSTRING::new(),
                &HSTRING::new(),
                WINHTTP_FLAG_ASYNC)
        };
    
        if h_session.is_null() {
            return Err(Error::from_win32());
        }
    
        self.h_session = h_session;
        Ok(())
    }

    pub fn connect(&mut self, host: HSTRING, port: u16) -> Result<(), Error> {
        let h_connect = unsafe {
            WinHttpConnect(
                self.h_session,
                &host,
                port,
                0
            )
        };
    
        if h_connect.is_null() {
            return Err(Error::from_win32());
        }
    
        self.h_connect = h_connect;
    
        Ok(())
    }

    // fn set_option(state: &mut State) -> Result<(), Error> {
    //      WinHttpSetOption(
    //             h_session,
    //             WINHTTP_OPTION_HTTP2_KEEPALIVE,
    //             &15000u32 as *const _ as *const c_void,
    //         );
    // }

    pub fn open_request(&mut self, method: HSTRING, url_path: HSTRING) -> Result<(), Error> {
        let h_request = unsafe {
            WinHttpOpenRequest(
                self.h_connect,
                &method,
                &url_path,
                PCWSTR::null(),
                PCWSTR::null(),
                std::ptr::null_mut(),
                WINHTTP_OPEN_REQUEST_FLAGS(0),
            )
        };
    
        if h_request.is_null() {
            return Err(Error::from_win32());
        }
    
        self.h_request = h_request;
        Ok(())
    }

    pub fn add_headers(&mut self, method: &str) -> Result<(), Error> {
        let content_type = match method {
            "GET" => HSTRING::from("Content-Type: text/plain"),
            "POST" => HSTRING::from("Content-Type: application/json"),
            _ => HSTRING::from("Content-Type: text/plain"),
        };
    
        unsafe {
            WinHttpAddRequestHeaders(
                self.h_request,
                content_type.as_wide(),
                WINHTTP_ADDREQ_FLAG_ADD,
            );
        }
        Ok(())
    }

    pub fn send_req(&mut self, total_length: u32, ctx: usize) -> Result<(), Error> {
        let mut b_results = Ok(());
    
        b_results = unsafe {
            WinHttpSendRequest(
                self.h_request,
                Some(&[0]),
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

    pub fn write_data(&self, buf: &[u8], buf_written: *mut u32) -> Result<()> {
        unsafe {
            WinHttpWriteData(
                self.h_request,
                Some(buf.as_ptr() as *const c_void),
                buf.len() as u32,
                buf_written,
            );
        };
        Ok(())
    }

    pub fn recv_resp(&mut self) -> Result<(), Error> {
        unsafe {
            WinHttpReceiveResponse(self.h_request, std::ptr::null_mut())
        }
    }

    pub fn query_data_available(&mut self, num_of_bytes_available: *mut u32) -> Result<(), Error> {
        unsafe {
            WinHttpQueryDataAvailable(
                self.h_request,
                num_of_bytes_available,
            )
        }
    }

    pub fn read_data(
        &mut self,
        buffer: &mut [u8],
        dw_size: u32,
        num_of_bytes_read: *mut u32,
) -> Result<(), Error> {
        unsafe {
            WinHttpReadData(
                self.h_request,
                buffer.as_mut_ptr() as *mut std::ffi::c_void,
                dw_size,
                num_of_bytes_read,
            )
        }
    }

    pub fn process(&mut self) -> Result<String, Error> {
        let mut response = String::new();
    
        let result = self.recv_resp();
    
        if result.is_err() {
            return Err(Error::from_win32());
        }
    
        loop {
            let mut dw_size = 0;
    
            let num_of_bytes_available_op: *mut u32 = match Some(&mut dw_size) {
                Some(op) => op,
                None => std::ptr::null_mut(),
            };
    
            if let Err(e) = self.query_data_available(num_of_bytes_available_op) {
                println!("Error querying data available: {e}");
            }
    
            if dw_size == 0 {
                break;
            }
    
            let mut buffer: Vec<u8> = vec![0; dw_size as usize];
            let mut lpdw_num_of_bytes_read: u32 = 0;
    
            let num_of_bytes_read_op: *mut u32 = match Some(&mut lpdw_num_of_bytes_read) {
                Some(op) => op,
                None => std::ptr::null_mut(),
            };
    
            let r = self.read_data(&mut buffer, dw_size, num_of_bytes_read_op);
            
            match r {
                Ok(_) => {
                    response = response + String::from_utf8_lossy(&buffer).to_string().as_str();
                },
                Err(e) => println!("Error: {:?}", e),
            }                        
    
            if dw_size > 0 {
                break;
            }
        }
    
        Ok(response)
    }

    pub fn close_handles(&mut self) {
        unsafe {
            if !self.h_request.is_null() {
                WinHttpCloseHandle(self.h_request);
            }
            if !self.h_connect.is_null() {
                WinHttpCloseHandle(self.h_connect);
            }
            if !self.h_session.is_null() {
                WinHttpCloseHandle(self.h_session);
            }
        }
    }
}

struct AsyncHandler {
    h_request: *mut c_void,

    ctx: Mutex<AsyncContext>,
}

impl AsyncHandler {
    pub fn new(h: &Handlers) -> Self {
        let ah = AsyncHandler {
            h_request: h.h_request,
            ctx: Mutex::new(AsyncContext::new()),
        };

        let prev = unsafe {
            unsafe {
                WinHttpSetStatusCallback(
                    h_request,
                    Some(AsyncCallback),
                    WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS,
                    0,
                )
            }
        };

        if let Some(p) = prev {
            let raw: *mut c_void = p as *mut c_void;
            let invalid: *mut c_void = -1_i64 as *mut c_void;
            if raw == invalid {
                let e = Error::from_win32();
                assert!(e.code().is_ok(), "Fail to set callback: {}", e);
            }
        }

        ah
    }

    pub async fn async_send_req(
        &mut self,
        handlers: &Handlers,
        headers: HSTRING,
        optional: &[u8],
        total_length: u32,
    ) -> Result<(), Error> {
        let token: AwaitableToken;
        {
            {
                let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
                token = lctx.get_await_token();
                assert_eq!(lctx.state, 0);
                lctx.state = WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE;
            }
            let ctx_ptr: *const Mutex<AsyncContext> = &self.ctx;
            let raw_ctx: *mut c_void = ctx_ptr as *mut c_void;
            handlers.send_req(total_length, raw_ctx as usize)?;
        }

        token.await;

        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()
        }
    }

    pub async fn async_write_data(
        &mut self,
        handlers: &Handlers,
        buffer: &mut [u8],
        buffer_written: u32,
    ) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_FLAG_WRITE_COMPLETE;
        }
        handlers.write_data(buffer, buf_written)?;
        token.await;
        let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
        lctx.err.code().ok()?;
        Ok(lctx.len)
    }

    pub async fn async_recv_resp(&mut self, handlers: &Handlers) -> Result<(), Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE;
        }
        handlers.recv_resp()?;
        token.await;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()
        }
    }

    pub async fn async_query_data_available(&mut self, handlers: &Handlers) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE;
        }
        handlers.query_data_available(None)?;
        token.await;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()?;
            Ok(lctx.len)
        }
    }

    pub async fn async_read_data(
        &mut self,
        handlers: &Handlers,
        buffer: &mut [u8],
        dw_size: u32,
        num_of_bytes_read: *mut u32
    ) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_READ_COMPLETE;
        }
        handlers.read_data(buffer, dw_size, num_of_bytes_read)?;
        token.await;

        let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
        lctx.err.code().ok()?;
        Ok(lctx.len)
    }
}

#[derive(Debug)]
struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            completed: false,
            waker: None,
        }
    }
}

struct AsyncWaitObject {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Default for AsyncWaitObject {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncWaitObject {
    fn new() -> Self {
        Self {
            shared_state: Arc::new(Mutex::new(SharedState::new()))
        }
    }

    fn wake(&self) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.completed = true;
        if let Some(waker) = shared_state.waker.take() {
            waker.wake()
        }
    }

    fn reset(&mut self) {
        self.shared_state = Arc::new(Mutex::new(SharedState::new()));
    }

    fn get_await_token(&self) -> AwaitableToken {
        AwaitableToken {
            shared_state: self.shared_state.clone(),
        }
    }
}

struct AwaitableToken {
    shared_state: Arc<Mutex<SharedState>>,
}

impl Future for AwaitableToken {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct AsyncContext {
    state: u32,
    as_obj: AsyncWaitObject,
    err: Error,
    len: u32,
}

impl AsyncContext {
    fn new() -> AsyncContext {
        AsyncContext {
            state: 0,
            as_obj: AsyncWaitObject::new(),
            err: Error::from(HRESULT(0)),
            len: 0,
        }
    }

    fn wake(&self) {
        self.as_obj.wake();
    }

    fn reset(&mut self) {
        self.as_obj.reset();
        self.state = 0;
        self.len = 0;
        self.err = Error::OK;
    }

    fn get_await_token(&self) -> AwaitableToken {
        self.as_obj.get_await_token()
    }
}

#[no_mangle]
extern "system" fn AsyncCallback(
    _handler: *mut c_void,
    dwcontext: usize,
    dwstatus: u32,
    lpvstatusinformation: *mut c_void,
    dwstatusinformationlength: u32,
) {
    assert_ne!(dwcontext, 0);

    let ctx_mtx_raw: *mut Mutex<AsyncContext> = dwcontext as *mut Mutex<AsyncContext>;
    let ctx_mtx: &mut Mutex<AsyncContext> = unsafe { &mut *ctx_mtx_raw };
    let ctx: &mut AsyncContext = ctx_mtx.get_mut().unwrap();
    assert_eq!(ctx.state, 0);

    match dwstatus {
        WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE => {
            assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE);
            ctx.state = 0;
            ctx.wake();
        }
        WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE => {
            assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE);
            ctx.state = 0;
            ctx.wake();
        }
        WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE => {
            assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE);
            ctx.state = 0;
            assert_eq!(
                dwstatusinformationlength as usize,
                std::mem::size_of::<u32>()
            );
            let temp_info: *mut u32 = lpvstatusinformation as *mut u32;
            let data_len: u32 = unsafe { *temp_info };
            ctx.len = data_len;
            ctx.wake();
        }
        WINHTTP_CALLBACK_FLAG_WRITE_COMPLETE => {
            assert_eq!(ctx.state, WINHTTP_CALLBACK_FLAG_WRITE_COMPLETE);
            ctx.state = 0;
            assert_eq!(
                dwstatusinformationlength as usize,
                std::mem::size_of::<u32>()
            );
            let temp_info: *mut u32 = lpvstatusinformation as *mut u32;
            let data_len: u32 = unsafe { *temp_info };
            ctx.len = data_len;
            ctx.wake();
        }
        WINHTTP_CALLBACK_STATUS_READ_COMPLETE => {
            assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_READ_COMPLETE);
            ctx.state = 0;
            ctx.len = dwstatusinformationlength;
            ctx.wake();
        }
        WINHTTP_CALLBACK_STATUS_REQUEST_ERROR => {
            let temp_res = lpvstatusinformation as *mut &WINHTTP_ASYNC_RESULT;
            let res = unsafe { *temp_res };
            let err = Error::from(HRESULT(res.dwError as i32));

            assert!(err.code().is_err());

            match res.dwResult as u32 {
                API_QUERY_DATA_AVAILABLE => {
                    assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE);
                }
                API_RECEIVE_RESPONSE => {
                    assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE);
                }
                API_SEND_REQUEST => {
                    assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE);
                }
                API_READ_DATA => {
                    assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_READ_COMPLETE);
                }
                API_WRITE_DATA => {
                    assert_eq!(ctx.state, WINHTTP_CALLBACK_STATUS_WRITE_COMPLETE);
                }
                _ => {
                    panic!("Unknown dwResult {}", res.dwResult);
                }
            }

            ctx.state = 0;
            ctx.err = err;
            ctx.wake();
        }
        _ => {
            panic!("Unknown callback case {}", dwstatus);
        }
    }
}