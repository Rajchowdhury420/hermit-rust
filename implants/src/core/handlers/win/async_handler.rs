use std::{
    ffi::c_void,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};
use windows::{
    core::{Error, HRESULT, HSTRING},
    Win32::Networking::WinHttp::*
};

use super::handler::HRequest;

const WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS: u32 = WINHTTP_CALLBACK_STATUS_SENDREQUEST_COMPLETE
    | WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE
    | WINHTTP_CALLBACK_STATUS_READ_COMPLETE
    | WINHTTP_CALLBACK_STATUS_WRITE_COMPLETE
    | WINHTTP_CALLBACK_STATUS_REQUEST_ERROR
    | WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE;

pub struct HRequestAsync {
    pub h: HRequest,

    ctx: Mutex<AsyncContext>,
}

impl HRequestAsync {
    pub fn new(h: HRequest) -> Self {
        let hra = HRequestAsync {
            h,
            ctx: Mutex::new(AsyncContext::new()),
        };

        let prev = hra.h.set_status_callback(
            Some(AsyncCallback),
            WINHTTP_CALLBACK_FLAG_ALL_COMPLETIONS,
            0,
        );

        if let Some(p) = prev {
            let raw: *mut c_void = p as *mut c_void;
            let invalid: *mut c_void = -1_i64 as *mut c_void;
            if raw == invalid {
                let e = Error::from_win32();
                assert!(e.code().is_ok(), "Fail to set callback: {}", e);
            }
        }

        hra
    }

    pub async fn async_send_req(
        &mut self,
        headers: HSTRING,
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
            self.h.send_req(headers, total_length, raw_ctx as usize)?;
        }

        token.await;

        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()
        }
    }

    pub async fn async_write_data(
        &mut self,
        buffer: &[u8],
        dwnumberofbytestowrite: u32,
    ) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_FLAG_WRITE_COMPLETE;
        }

        self.h.write_data(buffer, dwnumberofbytestowrite, None)?;

        token.await;

        let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
        lctx.err.code().ok()?;
        Ok(lctx.len)
    }

    pub async fn async_recv_resp(
        &mut self,
    ) -> Result<(), Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_HEADERS_AVAILABLE;
        }

        self.h.recv_resp()?;

        token.await;

        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()
        }
    }

    pub async fn async_query_data_available(
        &mut self,
    ) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_DATA_AVAILABLE;
        }
        self.h.query_data_available(None)?;

        token.await;

        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            lctx.err.code().ok()?;
            Ok(lctx.len)
        }
    }

    pub async fn async_read_data(
        &mut self,
        dw_size: u32,
        buffer: &mut [u8],
    ) -> Result<u32, Error> {
        let token: AwaitableToken;
        {
            let lctx: &mut AsyncContext = &mut self.ctx.lock().unwrap();
            assert_eq!(lctx.state, 0);
            lctx.reset();
            token = lctx.get_await_token();
            lctx.state = WINHTTP_CALLBACK_STATUS_READ_COMPLETE;
        }
        self.h.read_data(dw_size, buffer)?;

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
    dwinternetstatus: u32,
    lpvstatusinformation: *mut c_void,
    dwstatusinformationlength: u32,
) {
    assert_ne!(dwcontext, 0);

    let ctx_mtx_raw: *mut Mutex<AsyncContext> = dwcontext as *mut Mutex<AsyncContext>;
    let ctx_mtx: &mut Mutex<AsyncContext> = unsafe { &mut *ctx_mtx_raw };
    let ctx: &mut AsyncContext = ctx_mtx.get_mut().unwrap();
    assert_ne!(ctx.state, 0);

    match dwinternetstatus {
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
            panic!("Unknown callback case {}", dwinternetstatus);
        }
    }
}