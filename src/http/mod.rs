use alloc::vec::Vec;

pub struct Request<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub body: Option<&'a [u8]>,
    pub headers: Option<&'a [(&'a str, &'a str)]>,
}

pub struct Response {
    pub status_code: u16,
    pub body: Option<Vec<u8>>,
}
