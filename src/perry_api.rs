//! Perry TypeScript-to-native FFI surface (`perry-ffi`).
//!
//! Each `js_nsm_*` export is listed in `package.json` under
//! `perry.nativeLibrary.functions[]`. TypeScript wrappers in
//! `src/index.ts` call these symbols by name.

use aws_nitro_enclaves_nsm_api::api::{Request, Response};
use aws_nitro_enclaves_nsm_api::driver::{
    nsm_exit as api_nsm_exit, nsm_init as api_nsm_init, nsm_process_request,
};
use perry_ffi::{
    alloc_buffer, alloc_string, build_object_shape, js_array_alloc, js_array_push,
    js_object_alloc_with_shape, js_object_set_field, throw_with_code, ErrorKind, JsValue,
};
use serde_bytes::ByteBuf;

fn throw_nsm(err: impl std::fmt::Debug) -> ! {
    throw_with_code(
        &format!("NSM error: {:?}", err),
        "ERR_NSM",
        ErrorKind::Error,
    );
}

fn throw_unexpected() -> ! {
    throw_with_code(
        "Unexpected response from NSM",
        "ERR_NSM_UNEXPECTED",
        ErrorKind::Error,
    );
}

fn unwrap_response(res: Response) -> Response {
    match res {
        Response::Error(err) => throw_nsm(err),
        other => other,
    }
}

fn buffer_to_js(bytes: &[u8]) -> JsValue {
    let buf = alloc_buffer(bytes);
    if buf.is_null() {
        throw_with_code(
            "Failed to allocate Buffer for NSM response",
            "ERR_NSM_ALLOC",
            ErrorKind::Error,
        );
    }
    JsValue::from_object_ptr(buf)
}

/// Read a `buffer+len` parameter. Null pointer means `None`.
///
/// # Safety
///
/// When non-null, `ptr` must be valid for `len` bytes.
unsafe fn optional_bytes(ptr: *const u8, len: usize) -> Option<Vec<u8>> {
    if ptr.is_null() {
        None
    } else {
        Some(std::slice::from_raw_parts(ptr, len).to_vec())
    }
}

/// `nsmInit() -> number` — open the NSM device, return a file descriptor.
#[no_mangle]
pub extern "C" fn js_nsm_init() -> i32 {
    api_nsm_init()
}

/// `nsmExit(fd)` — close the NSM device.
#[no_mangle]
pub extern "C" fn js_nsm_exit(fd: i32) {
    api_nsm_exit(fd);
}

/// `nsmGetRandom(fd) -> Buffer` — request entropy from the NSM.
#[no_mangle]
pub extern "C" fn js_nsm_get_random(fd: i32) -> JsValue {
    let res = unwrap_response(nsm_process_request(fd, Request::GetRandom));
    if let Response::GetRandom { random } = res {
        buffer_to_js(&random)
    } else {
        throw_unexpected()
    }
}

/// `nsmExtendPcr(fd, index, data) -> Buffer` — extend a PCR and return the new value.
///
/// # Safety
///
/// `data_ptr` must be null or valid for `data_len` bytes. Perry passes this
/// pair from a `buffer+len` manifest parameter.
#[no_mangle]
pub unsafe extern "C" fn js_nsm_extend_pcr(
    fd: i32,
    index: u32,
    data_ptr: *const u8,
    data_len: usize,
) -> JsValue {
    let data = if data_ptr.is_null() {
        Vec::new()
    } else {
        std::slice::from_raw_parts(data_ptr, data_len).to_vec()
    };
    let res = unwrap_response(nsm_process_request(
        fd,
        Request::ExtendPCR {
            index: index as u16,
            data,
        },
    ));
    if let Response::ExtendPCR { data } = res {
        buffer_to_js(&data)
    } else {
        throw_unexpected()
    }
}

/// `nsmDescribePcr(fd, index) -> { lock, data }` — inspect a single PCR.
#[no_mangle]
pub extern "C" fn js_nsm_describe_pcr(fd: i32, index: u32) -> JsValue {
    let res = unwrap_response(nsm_process_request(
        fd,
        Request::DescribePCR {
            index: index as u16,
        },
    ));
    if let Response::DescribePCR { lock, data } = res {
        let keys = ["lock", "data"];
        let (packed, shape_id) = build_object_shape(&keys);
        // SAFETY: shape metadata is freshly built; field writes stay in bounds.
        unsafe {
            let obj = js_object_alloc_with_shape(
                shape_id,
                keys.len() as u32,
                packed.as_ptr(),
                packed.len() as u32,
            );
            js_object_set_field(obj, 0, JsValue::from_bool(lock));
            js_object_set_field(obj, 1, buffer_to_js(&data));
            JsValue::from_object_ptr(obj)
        }
    } else {
        throw_unexpected()
    }
}

/// `nsmLockPcr(fd, index)` — lock a single PCR against further extension.
#[no_mangle]
pub extern "C" fn js_nsm_lock_pcr(fd: i32, index: u32) {
    let _ = unwrap_response(nsm_process_request(
        fd,
        Request::LockPCR {
            index: index as u16,
        },
    ));
}

/// `nsmLockPcrs(fd, range)` — lock PCRs in `[0, range)`.
#[no_mangle]
pub extern "C" fn js_nsm_lock_pcrs(fd: i32, range: u32) {
    let _ = unwrap_response(nsm_process_request(
        fd,
        Request::LockPCRs {
            range: range as u16,
        },
    ));
}

/// `nsmDescribeNsm(fd) -> DescribeNsmResponse` — describe the NSM module.
#[no_mangle]
pub extern "C" fn js_nsm_describe_nsm(fd: i32) -> JsValue {
    let res = unwrap_response(nsm_process_request(fd, Request::DescribeNSM));
    if let Response::DescribeNSM {
        version_major,
        version_minor,
        version_patch,
        module_id,
        max_pcrs,
        locked_pcrs,
        digest,
    } = res
    {
        let keys = [
            "versionMajor",
            "versionMinor",
            "versionPatch",
            "moduleId",
            "maxPcrs",
            "lockedPcrs",
            "digest",
        ];
        let (packed, shape_id) = build_object_shape(&keys);
        // SAFETY: shape metadata is freshly built; field writes stay in bounds.
        unsafe {
            let obj = js_object_alloc_with_shape(
                shape_id,
                keys.len() as u32,
                packed.as_ptr(),
                packed.len() as u32,
            );
            js_object_set_field(obj, 0, JsValue::from_int32(version_major as i32));
            js_object_set_field(obj, 1, JsValue::from_int32(version_minor as i32));
            js_object_set_field(obj, 2, JsValue::from_int32(version_patch as i32));
            js_object_set_field(
                obj,
                3,
                JsValue::from_string_ptr(alloc_string(&module_id).as_raw()),
            );
            js_object_set_field(obj, 4, JsValue::from_int32(max_pcrs as i32));

            let locked: Vec<u16> = locked_pcrs.into_iter().collect();
            let mut arr = js_array_alloc(locked.len() as u32);
            for pcr in locked {
                arr = js_array_push(arr, JsValue::from_int32(pcr as i32));
            }
            js_object_set_field(obj, 5, JsValue::from_object_ptr(arr));
            js_object_set_field(
                obj,
                6,
                JsValue::from_string_ptr(alloc_string(&format!("{:?}", digest)).as_raw()),
            );
            JsValue::from_object_ptr(obj)
        }
    } else {
        throw_unexpected()
    }
}

/// `nsmGetAttestationDoc(fd, userData?, nonce?, publicKey?) -> Buffer`.
///
/// Optional buffers are passed as `buffer+len` pairs; a null pointer means
/// the option is omitted.
///
/// # Safety
///
/// Each non-null `*_ptr` must be valid for the matching `*_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn js_nsm_get_attestation_doc(
    fd: i32,
    user_data_ptr: *const u8,
    user_data_len: usize,
    nonce_ptr: *const u8,
    nonce_len: usize,
    public_key_ptr: *const u8,
    public_key_len: usize,
) -> JsValue {
    let req = Request::Attestation {
        user_data: optional_bytes(user_data_ptr, user_data_len).map(ByteBuf::from),
        nonce: optional_bytes(nonce_ptr, nonce_len).map(ByteBuf::from),
        public_key: optional_bytes(public_key_ptr, public_key_len).map(ByteBuf::from),
    };
    let res = unwrap_response(nsm_process_request(fd, req));
    if let Response::Attestation { document } = res {
        buffer_to_js(&document)
    } else {
        throw_unexpected()
    }
}
