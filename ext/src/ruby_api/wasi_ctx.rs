use super::{
    root,
    wasi_ctx_builder::{file_r, file_w, wasi_file},
    WasiCtxBuilder,
};
use crate::error;
use crate::helpers::OutputLimitedBuffer;
use deterministic_wasi_ctx::build_wasi_ctx as wasi_deterministic_ctx;
use magnus::{
    class, function, gc::Marker, method, prelude::*, typed_data::Obj, Error, Object, RString,
    RTypedData, Ruby, TypedData, Value,
};
use std::{borrow::Borrow, cell::RefCell, fs::File, path::PathBuf};
use wasi_common::pipe::{ReadPipe, WritePipe};
use wasi_common::WasiCtx as WasiCtxImpl;

/// @yard
/// WASI context to be sent as {Store#new}’s +wasi_ctx+ keyword argument.
///
/// Instance methods mutate the current object and return +self+.
///
/// @see https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html
///   Wasmtime's Rust doc
#[magnus::wrap(class = "Wasmtime::WasiCtx", size, free_immediately)]
pub struct WasiCtx {
    inner: RefCell<WasiCtxImpl>,
}

type RbSelf = Obj<WasiCtx>;

impl WasiCtx {
    /// @yard
    /// Create a new deterministic {WasiCtx}. See https://github.com/Shopify/deterministic-wasi-ctx for more details
    /// @return [WasiCtx]
    pub fn deterministic() -> Self {
        Self {
            inner: RefCell::new(wasi_deterministic_ctx()),
        }
    }

    /// @yard
    /// Set stdin to read from the specified file.
    /// @def set_stdin_file(path)
    /// @param path [String] The path of the file to read from.
    /// @return [WasiCtxBuilder] +self+
    fn set_stdin_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_r(path).map(wasi_file).unwrap();
        inner.set_stdin(cs);
        rb_self
    }

    /// @yard
    /// Set stdin to the specified String.
    /// @def set_stdin_string(content)
    /// @param content [String]
    /// @return [WasiCtx] +self+
    fn set_stdin_string(rb_self: RbSelf, content: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let str = unsafe { content.as_slice() };
        let pipe = ReadPipe::from(str);
        inner.set_stdin(Box::new(pipe));
        rb_self
    }

    /// @yard
    /// Set stdout to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @def set_stdout_file(path)
    /// @param path [String] The path of the file to write to.
    /// @return [WasiCtx] +self+
    fn set_stdout_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stdout(cs);
        rb_self
    }

    /// @yard
    /// Set stdout to write to a string buffer.
    /// If the string buffer is frozen, Wasm execution will raise a Wasmtime::Error error.
    /// No encoding checks are done on the resulting string, it is the caller's responsibility to ensure the string contains a valid encoding
    /// @def set_stdout_buffer(buffer, capacity)
    /// @param buffer [String] The string buffer to write to.
    /// @param capacity [Integer] The maximum number of bytes that can be written to the output buffer.
    /// @return [WasiCtx] +self+
    fn set_stdout_buffer(rb_self: RbSelf, buffer: RString, capacity: usize) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let pipe = WritePipe::new(OutputLimitedBuffer::new(buffer.into(), capacity));
        inner.set_stdout(Box::new(pipe));
        rb_self
    }

    /// @yard
    /// Set stderr to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @def set_stderr_file(path)
    /// @param path [String] The path of the file to write to.
    /// @return [WasiCtx] +self+
    fn set_stderr_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stderr(cs);
        rb_self
    }

    /// @yard
    /// Set stderr to write to a string buffer.
    /// If the string buffer is frozen, Wasm execution will raise a Wasmtime::Error error.
    /// No encoding checks are done on the resulting string, it is the caller's responsibility to ensure the string contains a valid encoding
    /// @def set_stderr_buffer(buffer, capacity)
    /// @param buffer [String] The string buffer to write to.
    /// @param capacity [Integer] The maximum number of bytes that can be written to the output buffer.
    /// @return [WasiCtx] +self+
    fn set_stderr_buffer(rb_self: RbSelf, buffer: RString, capacity: usize) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let pipe = WritePipe::new(OutputLimitedBuffer::new(buffer.into(), capacity));
        inner.set_stderr(Box::new(pipe));
        rb_self
    }

    pub fn from_inner(inner: WasiCtxImpl) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }

    pub fn get_inner(&self) -> WasiCtxImpl {
        return self.inner.borrow().clone();
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("WasiCtx", class::object())?;
    class.define_singleton_method("deterministic", function!(WasiCtx::deterministic, 0))?;
    class.define_method("set_stdin_file", method!(WasiCtx::set_stdin_file, 1))?;
    class.define_method("set_stdin_string", method!(WasiCtx::set_stdin_string, 1))?;
    class.define_method("set_stdout_file", method!(WasiCtx::set_stdout_file, 1))?;
    class.define_method("set_stdout_buffer", method!(WasiCtx::set_stdout_buffer, 2))?;
    class.define_method("set_stderr_file", method!(WasiCtx::set_stderr_file, 1))?;
    class.define_method("set_stderr_buffer", method!(WasiCtx::set_stderr_buffer, 2))?;
    Ok(())
}
