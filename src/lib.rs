#![deny(clippy::all)]

use aws_nitro_enclaves_nsm_api::api::{Request, Response};
use aws_nitro_enclaves_nsm_api::driver::{
    nsm_exit as api_nsm_exit, nsm_init as api_nsm_init, nsm_process_request,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_bytes::ByteBuf;

#[napi]
pub fn nsm_init() -> i32 {
    api_nsm_init()
}

#[napi]
pub fn nsm_exit(fd: i32) {
    api_nsm_exit(fd)
}

fn handle_response(res: Response) -> Result<Response> {
    match res {
        Response::Error(err) => Err(Error::new(Status::GenericFailure, format!("{:?}", err))),
        _ => Ok(res),
    }
}

#[napi]
pub fn nsm_get_random(fd: i32) -> Result<Buffer> {
    let res = handle_response(nsm_process_request(fd, Request::GetRandom))?;
    if let Response::GetRandom { random } = res {
        Ok(random.into())
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Unexpected response from NSM",
        ))
    }
}

#[napi]
pub fn nsm_extend_pcr(fd: i32, index: u16, data: Buffer) -> Result<Buffer> {
    let res = handle_response(nsm_process_request(
        fd,
        Request::ExtendPCR {
            index,
            data: data.to_vec(),
        },
    ))?;
    if let Response::ExtendPCR { data } = res {
        Ok(data.into())
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Unexpected response from NSM",
        ))
    }
}

#[napi]
pub fn nsm_describe_pcr(fd: i32, index: u16) -> Result<DescribePcrResponse> {
    let res = handle_response(nsm_process_request(fd, Request::DescribePCR { index }))?;
    if let Response::DescribePCR { lock, data } = res {
        Ok(DescribePcrResponse {
            lock,
            data: data.into(),
        })
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Unexpected response from NSM",
        ))
    }
}

#[napi(object)]
pub struct DescribePcrResponse {
    pub lock: bool,
    pub data: Buffer,
}

#[napi]
pub fn nsm_lock_pcr(fd: i32, index: u16) -> Result<()> {
    handle_response(nsm_process_request(fd, Request::LockPCR { index }))?;
    Ok(())
}

#[napi]
pub fn nsm_lock_pcrs(fd: i32, range: u16) -> Result<()> {
    handle_response(nsm_process_request(fd, Request::LockPCRs { range }))?;
    Ok(())
}

#[napi(object)]
pub struct DescribeNsmResponse {
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub module_id: String,
    pub max_pcrs: u16,
    pub locked_pcrs: Vec<u16>,
    pub digest: String,
}

#[napi]
pub fn nsm_describe_nsm(fd: i32) -> Result<DescribeNsmResponse> {
    let res = handle_response(nsm_process_request(fd, Request::DescribeNSM))?;
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
        Ok(DescribeNsmResponse {
            version_major,
            version_minor,
            version_patch,
            module_id,
            max_pcrs,
            locked_pcrs: locked_pcrs.into_iter().collect(),
            digest: format!("{:?}", digest),
        })
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Unexpected response from NSM",
        ))
    }
}

#[napi]
pub fn nsm_get_attestation_doc(
    fd: i32,
    user_data: Option<Buffer>,
    nonce: Option<Buffer>,
    public_key: Option<Buffer>,
) -> Result<Buffer> {
    let req = Request::Attestation {
        user_data: user_data.map(|b| ByteBuf::from(b.to_vec())),
        nonce: nonce.map(|b| ByteBuf::from(b.to_vec())),
        public_key: public_key.map(|b| ByteBuf::from(b.to_vec())),
    };
    let res = handle_response(nsm_process_request(fd, req))?;
    if let Response::Attestation { document } = res {
        Ok(document.into())
    } else {
        Err(Error::new(
            Status::GenericFailure,
            "Unexpected response from NSM",
        ))
    }
}
