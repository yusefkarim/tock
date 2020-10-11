//! Tock syscall number definitions and arch-agnostic interface trait.

use core::fmt::Write;

use crate::process;
use crate::returncode::ReturnCode;

use crate::mem::{AppSlice, Shared};

/// The syscall number assignments.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Syscall {
    /// Return to the kernel to allow other processes to execute or to wait for
    /// interrupts and callbacks.
    ///
    /// SVC_NUM = 0
    YIELD,

    /// Pass a callback function to the kernel.
    ///
    /// SVC_NUM = 1
    SUBSCRIBE {
        driver_number: usize,
        subdriver_number: usize,
        callback_ptr: *mut (),
        appdata: usize,
    },

    /// Instruct the kernel or a capsule to perform an operation.
    ///
    /// SVC_NUM = 2
    COMMAND {
        driver_number: usize,
        subdriver_number: usize,
        arg0: usize,
        arg1: usize,
    },

    /// Share a memory buffer with the kernel.
    ///
    /// SVC_NUM = 3
    ALLOW {
        driver_number: usize,
        subdriver_number: usize,
        allow_address: *mut u8,
        allow_size: usize,
    },

    /// Various memory operations.
    ///
    /// SVC_NUM = 4
    MEMOP { operand: usize, arg0: usize },
}

/// Why the process stopped executing and execution returned to the kernel.
#[derive(PartialEq)]
pub enum ContextSwitchReason {
    /// Process called a syscall. Also returns the syscall and relevant values.
    SyscallFired { syscall: Syscall },
    /// Process triggered the hardfault handler.
    Fault,
    /// Process exceeded its timeslice.
    TimesliceExpired,
    /// Process interrupted (e.g. by a hardware event)
    Interrupted,
}


// To allow type refinement on the enum (e.g., factory constructor
// methods), we need to embed structs within the enum values.
// https://github.com/rust-lang/rfcs/issues/754
// https://github.com/rust-lang/rust/issues/1679
// https://www.reddit.com/r/rust/comments/2rdoxx/enum_variants_as_types/
// -pal 9/22/20
#[derive(Debug)]
pub struct SyscallFailure {error: ReturnCode}
#[derive(Debug)]
pub struct SyscallFailureU32 {error: ReturnCode, rval0: u32}
#[derive(Debug)]
pub struct SyscallFailureU32U32 {error: ReturnCode, rval0: u32, rval1: u32}
// Frustrating question: we want the u64 to be 8-byte aligned;
// how do we ensure that? This layout assumes the enum encoding is
// the first word. -pal
#[derive(Debug)]
pub struct SyscallFailureU64 {error: ReturnCode, rval0: u64}
#[derive(Debug)]
pub struct SyscallSuccessU32 {rval0: u32}
#[derive(Debug)]
pub struct SyscallSuccessU32U32 {rval0: u32, rval1: u32}
#[derive(Debug)]
pub struct SyscallSuccessU64 {rval0: u64}
#[derive(Debug)]
pub struct SyscallSuccessU32U32U32 {rval0: u32, rval1: u32, rval2: u32}
// Frustrating question: we want the u64 to be 8-byte aligned;
// how do we ensure that? This layout assumes the enum encoding is
// the first word. -pal
#[derive(Debug)]
pub struct SyscallSuccessU32U64 {rval0: u32, rval1: u64}

pub struct SyscallSuccessAllow {buf: AppSlice<Shared, u8>}
pub struct SyscallFailureAllow {error: ReturnCode, buf: AppSlice<Shared, u8>}
pub struct SyscallFailureAllowRaw {error: ReturnCode, addr: u32, len: u32}

/// Enumeration of the possible return values from a system call.
#[derive(Debug)]
pub enum CommandResult {
    Failure (SyscallFailure),
    FailureU32 (SyscallFailureU32), 
    FailureU32U32 (SyscallFailureU32U32),
    FailureU64 (SyscallFailureU64),
    Success,
    SuccessU32 (SyscallSuccessU32),
    SuccessU32U32 (SyscallSuccessU32U32),
    SuccessU64 (SyscallSuccessU64),
    SuccessU32U32U32 (SyscallSuccessU32U32U32),
    // Frustrating question: we want the u64 to be 8-byte aligned;
    // how do we ensure that? This layout assumes the enum encoding is
    // the first word. -pal
    SuccessU32U64 (SyscallSuccessU32U64),
}

#[derive(Debug)]
pub enum SubscribeResult {
    Failure (SyscallFailure),
    Success
}

pub enum AllowResult {
    Failure (SyscallFailureAllow),
    FailureRaw (SyscallFailureAllowRaw),
    Success (SyscallSuccessAllow),
}

pub trait SyscallResult {
    fn return_code_to_error_code(rcode: ReturnCode) -> u32 {
        match rcode {
            ReturnCode::SuccessWithValue { value: _ } => 0,
            ReturnCode::SUCCESS => 0,
            ReturnCode::FAIL => 1,
            ReturnCode::EBUSY => 2,
            ReturnCode::EALREADY => 3,
            ReturnCode::EOFF => 4,
            ReturnCode::ERESERVE => 5,
            ReturnCode::EINVAL => 6,
            ReturnCode::ESIZE => 7,
            ReturnCode::ECANCEL => 8,
            ReturnCode::ENOMEM => 9,
            ReturnCode::ENOSUPPORT => 10,
            ReturnCode::ENODEVICE => 11,
            ReturnCode::EUNINSTALLED => 12,
            ReturnCode::ENOACK => 13,
        }
    }
    fn into_registers(&self, r0: &mut u32, r1: &mut u32, r2: &mut u32, r3: &mut u32);
}

impl SyscallResult for SubscribeResult {
    fn into_registers(&self, r0: &mut u32, r1: &mut u32, _r2: &mut u32, _r3: &mut u32) {
        match self {
            SubscribeResult::Failure(fail) => {
                *r0 = 0;
                *r1 = Self::return_code_to_error_code(fail.error);
            },
            SubscribeResult::Success => {
                *r0 = 128;
            },
        }
    }
}
impl SyscallResult for AllowResult {
    fn into_registers(&self, r0: &mut u32, r1: &mut u32, r2: &mut u32, r3: &mut u32) {
        match self {
            AllowResult::Failure(fail) => {
                *r0 = 2;
                *r1 = Self::return_code_to_error_code(fail.error);
                *r2 = fail.buf.ptr() as u32;
                *r3 = fail.buf.len() as u32;
            },
            AllowResult::FailureRaw(fail) => {
                *r0 = 2;
                *r1 = Self::return_code_to_error_code(fail.error);
                *r2 = fail.addr;
                *r3 = fail.len;
            },
            AllowResult::Success(success) => {
                *r0 = 130;
                *r1 = success.buf.ptr() as u32;
                *r2 = success.buf.len() as u32;	
            },
        }
    }
}
impl SyscallResult for CommandResult {
    fn into_registers(&self, r0: &mut u32, r1: &mut u32, r2: &mut u32, r3: &mut u32) {
        match self {
            CommandResult::Failure(fail) => {
                *r0 = 0;
                *r1 = Self::return_code_to_error_code(fail.error);
            },
            CommandResult::FailureU32(fail) => {
                *r0 = 1;
                *r1 = Self::return_code_to_error_code(fail.error);
                *r2 = fail.rval0;
            },
            CommandResult::FailureU32U32(fail) => {
                *r0 = 2;
                *r1 = Self::return_code_to_error_code(fail.error);
                *r2 = fail.rval0;
                *r3 = fail.rval1;
            },
            CommandResult::FailureU64(fail) => {
                *r0 = 3;
                *r1 = Self::return_code_to_error_code(fail.error);
                *r2 = (fail.rval0 & 0xffff_ffffff) as u32;
                *r3 = (fail.rval0 >> 32) as u32;
            },
            CommandResult::Success => {
                *r0 = 128;
            },
            CommandResult::SuccessU32(success) => {
                *r0 = 129;
                *r1 = success.rval0;
            },
            CommandResult::SuccessU32U32(success) => {
                *r0 = 130;
                *r1 = success.rval0;
                *r2 = success.rval1;
            },
            CommandResult::SuccessU64(success) => {
                *r0 = 131;
                *r1 = (success.rval0 & 0xffff_ffff) as u32;
                *r2 = (success.rval0 >> 32) as u32;
            },
            CommandResult::SuccessU32U32U32(success) => {
                *r0 = 132;
                *r1 = success.rval0;
                *r2 = success.rval1;
                *r3 = success.rval2;
            },
            CommandResult::SuccessU32U64(success) => {
                *r0 = 133;
                *r1 = success.rval0;
                *r2 = (success.rval1 & 0xffff_ffff) as u32;
                *r3 = (success.rval1 >> 32) as u32;
            },
        }
    }
}


pub struct SrvFactory;

impl SrvFactory {
    pub fn failure(error: ReturnCode) -> SyscallFailure {
        SyscallFailure {error}
    }

    pub fn failure_u32(error: ReturnCode, rval0: u32) -> SyscallFailureU32 {
        SyscallFailureU32 {error, rval0}
    }

    pub fn failure_u32_u32(error: ReturnCode, rval0: u32, rval1: u32) -> SyscallFailureU32U32 {
        SyscallFailureU32U32 {error, rval0, rval1}
    }

    pub fn failure_u64(error: ReturnCode, rval0: u64) -> SyscallFailureU64 {
        SyscallFailureU64 {error, rval0}
    }

    pub fn success_u32(rval0: u32) -> SyscallSuccessU32 {
        SyscallSuccessU32 {rval0}
    }

    pub fn success_u32_u32(rval0: u32, rval1: u32) -> SyscallSuccessU32U32 {
        SyscallSuccessU32U32 {rval0, rval1}
    }

    pub fn success_u64(rval0: u64) -> SyscallSuccessU64 {
        SyscallSuccessU64 {rval0}
    }

    pub fn success_u32_u32_u32(rval0: u32, rval1: u32, rval2: u32) -> SyscallSuccessU32U32U32 {
        SyscallSuccessU32U32U32 {rval0, rval1, rval2}
    }

    pub fn success_u32_u64(rval0: u32, rval1: u64) -> SyscallSuccessU32U64 {
        SyscallSuccessU32U64 {rval0, rval1}
    }

    pub fn success_allow(buf: AppSlice<Shared, u8>) -> SyscallSuccessAllow {
        SyscallSuccessAllow {buf}
    }

    pub fn failure_allow(error: ReturnCode, buf: AppSlice<Shared, u8>) -> SyscallFailureAllow {
        SyscallFailureAllow {error, buf}
    } 

    pub fn failure_allow_raw(error: ReturnCode, addr: u32, len: u32) -> SyscallFailureAllowRaw {
        SyscallFailureAllowRaw {error, addr, len}
    } 
}

/// This trait must be implemented by the architecture of the chip Tock is
/// running on. It allows the kernel to manage switching to and from processes
/// in an architecture-agnostic manner.
pub trait UserspaceKernelBoundary {
    /// Some architecture-specific struct containing per-process state that must
    /// be kept while the process is not running. For example, for keeping CPU
    /// registers that aren't stored on the stack.
    ///
    /// Implementations should **not** rely on the `Default` constructor (custom
    /// or derived) for any initialization of a process's stored state. The
    /// initialization must happen in the `initialize_process()` function.
    type StoredState: Default;

    /// Called by the kernel after it has memory allocated to it but before it
    /// is allowed to begin executing. Allows for architecture-specific process
    /// setup, e.g. allocating a syscall stack frame.
    ///
    /// This function must also initialize the stored state (if needed).
    ///
    /// This function may be called multiple times on the same process. For
    /// example, if a process crashes and is to be restarted, this must be
    /// called. Or if the process is moved this may need to be called.
    unsafe fn initialize_process(
        &self,
        stack_pointer: *const usize,
        stack_size: usize,
        state: &mut Self::StoredState,
    ) -> Result<*const usize, ()>;

    /// Set the return value the process should see when it begins executing
    /// again after a command syscall. This will only be called after a
    /// process has called a command.
    ///
    /// To help implementations, both the current stack pointer of the process
    /// and the saved state for the process are provided. The `return_value` is
    /// the value that should be passed to the process so that when it resumes
    /// executing it knows the return value of the syscall it called.
    unsafe fn set_syscall_return_command(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: &CommandResult,
    );


    /// Set the return value the process should see when it begins executing
    /// again after a subscribe syscall. This will only be called after a
    /// process has called a subscribe.
    ///
    /// To help implementations, both the current stack pointer of the process
    /// and the saved state for the process are provided. The `return_value` is
    /// the value that should be passed to the process so that when it resumes
    /// executing it knows the return value of the syscall it called.
    unsafe fn set_syscall_return_subscribe(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: &SubscribeResult,
    );


    /// Set the return value the process should see when it begins executing
    /// again after an allow syscall. This will only be called after a
    /// process has called an allow.
    ///
    /// To help implementations, both the current stack pointer of the process
    /// and the saved state for the process are provided. The `return_value` is
    /// the value that should be passed to the process so that when it resumes
    /// executing it knows the return value of the syscall it called.
    unsafe fn set_syscall_return_allow(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
        return_value: &AllowResult,
    );

    /// Set the function that the process should execute when it is resumed.
    /// This has two major uses: 1) sets up the initial function call to
    /// `_start` when the process is started for the very first time; 2) tells
    /// the process to execute a callback function after calling `yield()`.
    ///
    /// **Note:** This method cannot be called in conjunction with
    /// `set_syscall_return_value`, as the injected function will clobber the
    /// return value.
    ///
    /// ### Arguments
    ///
    /// - `stack_pointer` is the address of the stack pointer for the current
    ///   app.
    /// - `remaining_stack_memory` is the number of bytes below the
    ///   `stack_pointer` that is allocated for the process. This value is
    ///   checked by the implementer to ensure that there is room for this stack
    ///   frame without overflowing the stack.
    /// - `state` is the stored state for this process.
    /// - `callback` is the function that should be executed when the process
    ///   resumes.
    ///
    /// ### Return
    ///
    /// Returns `Ok` or `Err` with the current address of the stack pointer for
    /// the process. One reason for returning `Err` is that adding the function
    /// call requires adding to the stack, and there is insufficient room on the
    /// stack to add the function call.
    unsafe fn set_process_function(
        &self,
        stack_pointer: *const usize,
        remaining_stack_memory: usize,
        state: &mut Self::StoredState,
        callback: process::FunctionCall,
    ) -> Result<*mut usize, *mut usize>;

    /// Context switch to a specific process.
    ///
    /// This returns a tuple:
    /// - The new stack pointer address of the process.
    /// - Why the process stopped executing and switched back to the kernel.
    unsafe fn switch_to_process(
        &self,
        stack_pointer: *const usize,
        state: &mut Self::StoredState,
    ) -> (*mut usize, ContextSwitchReason);

    /// Display architecture specific (e.g. CPU registers or status flags) data
    /// for a process identified by its stack pointer.
    unsafe fn print_context(
        &self,
        stack_pointer: *const usize,
        state: &Self::StoredState,
        writer: &mut dyn Write,
    );
}

/// Helper function for converting raw values passed back from an application
/// into a `Syscall` type in Tock.
///
/// Different architectures may have different mechanisms for passing
/// information about what syscall an app called, but they will have have to
/// convert the series of raw values into a more useful Rust type. While
/// implementations are free to do this themselves, this provides a generic
/// helper function which should help reduce duplicated code.
///
/// The mappings between raw `syscall_number` values and the associated syscall
/// type are specified and fixed by Tock. After that, this function only
/// converts raw values to more meaningful types based on the syscall.
pub fn arguments_to_syscall(
    syscall_number: u8,
    r0: usize,
    r1: usize,
    r2: usize,
    r3: usize,
) -> Option<Syscall> {
    match syscall_number {
        0 => Some(Syscall::YIELD),
        1 => Some(Syscall::SUBSCRIBE {
            driver_number: r0,
            subdriver_number: r1,
            callback_ptr: r2 as *mut (),
            appdata: r3,
        }),
        2 => Some(Syscall::COMMAND {
            driver_number: r0,
            subdriver_number: r1,
            arg0: r2,
            arg1: r3,
        }),
        3 => Some(Syscall::ALLOW {
            driver_number: r0,
            subdriver_number: r1,
            allow_address: r2 as *mut u8,
            allow_size: r3,
        }),
        4 => Some(Syscall::MEMOP {
            operand: r0,
            arg0: r1,
        }),
        _ => None,
    }
}
