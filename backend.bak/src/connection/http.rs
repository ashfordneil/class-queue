const SERVER: &'static str = "ClassQueue";

/// Return a HTTP 405 (invalid method) response
pub fn invalid_method() -> Vec<u8> {
    format!(
        "\
         HTTP/1.1 405 Method Not Allowed\r\n\
         Allow: GET\r\n\
         Content-Length: 0\r\n\
         Server: {}\r\n\
         \r\n\r\n",
        SERVER,
    ).into_bytes()
}
