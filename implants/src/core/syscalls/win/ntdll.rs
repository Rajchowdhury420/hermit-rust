use core::arch::global_asm;
use ntapi::ntpsapi::PPS_ATTRIBUTE_LIST;
use std::ffi::c_void;
use windows::{
    Wdk::Foundation::OBJECT_ATTRIBUTES,
    Win32::Foundation::{HANDLE, NTSTATUS},
};

// import NT functions from ntdll
// Reference:
//  https://github.com/felix-rs/ntcall-rs/blob/290745855f561a106c53e25abee1e3d7a9065874/build.rs
#[link(name = "ntdll")]
extern "system" {
    // It exists on the `windows` crate.
    // ----------------------------------
    // pub fn NtAllocateVirtualMemory(
    //     ProcessHandle: HANDLE,
    //     BaseAddress: *mut PVOID,
    //     ZeroBits: ULONG_PTR,
    //     RegionSize: PSIZE_T,
    //     AllocationType: ULONG,
    //     Protect: ULONG,
    // ) -> NTSTATUS;

    // It exists on the `windows` crate.
    // -----------------------------------
    // pub fn NtClose(Handle: HANDLE) -> NTSTATUS;

    pub fn NtCreateThreadEx(
        ThreadHandle: *mut HANDLE,
        DesiredAccess: u32,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        ProcessHandle: HANDLE,
        StartRoutine: *mut *mut c_void,
        Argument: *mut *mut c_void,
        CreateFlags: u64,
        ZeroBits: usize,
        StackSize: usize,
        MaximumStackSize: usize,
        AttributeList: PPS_ATTRIBUTE_LIST,
    ) -> NTSTATUS;

    // It exists on the `windows` crate.
    // ----------------------------------
    // pub fn NtOpenProcess(
    //     ProcessHandle: PHANDLE,
    //     DesiredAccess: ACCESS_MASK,
    //     ObjectAttributes: POBJECT_ATTRIBUTES,
    //     ClientId: PCLIENT_ID,
    // ) -> NTSTATUS;

    // It exists on the `windows` crate.
    // ----------------------------------
    // pub fn NtWaitForSingleObject(
    //     Handle: HANDLE,
    //     Alertable: BOOLEAN,
    //     Timeout: PLARGE_INTEGER,
    // ) -> NTSTATUS;

    pub fn NtWriteVirtualMemory(
        ProcessHandle: *mut HANDLE,
        BaseAddress: u64, // *mut *mut c_void,
        Buffer: u64, // *mut *mut c_void,
        BufferSize: usize,
        NumberOfBytesWritten: u64,
    ) -> NTSTATUS;
}

global_asm!(
    r#"
.macro define_syscall name, id
.global \name
\name:
    mov r10, rcx
    mov eax, \id
    syscall
    ret
.endm
"#
);

global_asm!(include_str!("../../../../resources/syscall_ids"));