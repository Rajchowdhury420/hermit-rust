// References for Windows API:
//  - https://learn.microsoft.com/en-us/windows/win32/sysinfo/system-information-functions
//  - https://microsoft.github.io/windows-docs-rs/doc/windows/
use std::io::{Error, ErrorKind};
use windows::{
    core::{HSTRING, PWSTR},
    Win32::{
        // Networking::WinSock::{gethostname, WSAGetLastError},
        // Security::Authentication::Identity::{
        //     GetComputerObjectNameW,
        //     GetUserNameExW,
        // },
        System::SystemInformation::{
            ComputerNameDnsHostname,
            GetComputerNameExW,
            GetSystemInfo,
            //     GetSystemWindowsDirectoryW,
            //     GetSystemWow64DirectoryW,
            //     GetSystemWow64Directory2W,
            //     GetVersionExW,
            //     GetWindowsDirectoryW
            SYSTEM_INFO,
        },
    },
};

// pub fn get_hostname() {
//     unsafe {
//         let mut hostname: Vec<u8> = Vec::new();
//         let result = gethostname(&mut hostname);
//         if result == 0 {
//             println!("hostname: {}", from_utf8(&hostname).unwrap());
//         } else {
//             println!("result: {}: {:#?}", result, WSAGetLastError());
//         }
//     }
// }

pub fn get_computer_name() -> Result<String, Error> {
    unsafe {
        let mut buffer_size = 0;

        // Get buffer size
        GetComputerNameExW(
            ComputerNameDnsHostname,
            PWSTR::null(),
            &mut buffer_size,
        );

        let mut buffer = Vec::with_capacity((buffer_size as usize) + 8);
        if let Err(e) = GetComputerNameExW(
            ComputerNameDnsHostname,
            PWSTR::from_raw(buffer.as_mut_ptr()),
            &mut buffer_size,
        ) {
            println!("Error: {e}");
            return Err(Error::new(ErrorKind::Other, format!("{e}")));
        }

        buffer.set_len(buffer_size as usize);
        match String::from_utf16(&buffer) {
            Ok(name) => Ok(name),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("{e}"))),
        }
    }
}

pub fn get_systeminfo() {
    unsafe {
        let mut buffer = SYSTEM_INFO::default();
        GetSystemInfo(&mut buffer);
        println!("Number of Processors: {:?}", buffer.dwNumberOfProcessors);
    }
}