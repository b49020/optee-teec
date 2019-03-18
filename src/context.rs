use libc;
use optee_teec_sys as raw;
use std::ptr;

use crate::{ConnectionMethods, Error, Result, Session, Uuid};

pub struct Context {
    raw: raw::TEEC_Context,
}

impl Context {
    pub fn new() -> Result<Context> {
        Context::new_raw(0, true)
    }

    pub fn new_raw(fd: libc::c_int, reg_mem: bool) -> Result<Context> {
        let mut raw_ctx = raw::TEEC_Context { fd, reg_mem };
        unsafe {
            match raw::TEEC_InitializeContext(ptr::null_mut() as *mut libc::c_char, &mut raw_ctx) {
                raw::TEEC_SUCCESS => Ok(Context { raw: raw_ctx }),
                code => Err(Error::from_raw_error(code)),
            }
        }
    }

    pub fn as_mut_raw_ptr(&mut self) -> *mut raw::TEEC_Context {
        &mut self.raw
    }

    pub fn open_session(&mut self, uuid: Uuid) -> Result<Session> {
        let mut raw_session = raw::TEEC_Session {
            ctx: self.as_mut_raw_ptr(),
            session_id: 0,
        };
        let mut err_origin: libc::uint32_t = 0;
        unsafe {
            match raw::TEEC_OpenSession(
                self.as_mut_raw_ptr(),
                &mut raw_session,
                uuid.as_raw_ptr(),
                ConnectionMethods::LoginPublic as u32,
                ptr::null() as *const libc::c_void,
                ptr::null_mut() as *mut raw::TEEC_Operation,
                &mut err_origin,
            ) {
                raw::TEEC_SUCCESS => Ok(Session::from_raw(raw_session)),
                code => Err(Error::from_raw_error(code)),
            }
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            raw::TEEC_FinalizeContext(&mut self.raw);
        }
    }
}
