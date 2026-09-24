// SPDX-License-Identifier: Apache-2.0
//! The fetch against a server on `127.0.0.1`, so the suite reaches no other
//! host and runs offline.

use std::io::{Read, Write};
use std::net::TcpListener;

/// A zip with one file at the root and one in a directory.
fn archive() -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    writer.start_file("headwater-package.yml", options).unwrap();
    writer.write_all(b"name: acme/example\n").unwrap();
    writer.start_file("schema/kinds.yml", options).unwrap();
    writer.write_all(b"kinds: []\n").unwrap();
    writer.finish().unwrap().into_inner()
}

/// Serve `count` requests. `/moved` answers 302 to `/x.zip`, `/x.zip` answers
/// with the archive, and anything else answers 404.
fn serve(count: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let body = archive();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(count) {
            let mut stream = stream.unwrap();
            let mut request = [0_u8; 4096];
            let read = stream.read(&mut request).unwrap();
            let line = String::from_utf8_lossy(&request[..read]).to_string();
            let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
            let (head, payload): (String, &[u8]) = match path.as_str() {
                "/moved" => (
                    "HTTP/1.1 302 Found\r\nLocation: /x.zip\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                    &[],
                ),
                "/x.zip" => (
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    ),
                    &body,
                ),
                _ => (
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                    &[],
                ),
            };
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(payload).unwrap();
        }
    });
    base
}

#[test]
fn a_loopback_zip_unpacks_with_its_members_at_the_root() {
    let base = serve(1);
    let fetched = headwater_fetch::fetch(&format!("{base}/x.zip")).unwrap();
    let manifest =
        std::fs::read_to_string(fetched.path().join("headwater-package.yml")).unwrap();
    assert_eq!(manifest, "name: acme/example\n");
    assert!(fetched.path().join("schema/kinds.yml").is_file());

    let dir = fetched.path().to_path_buf();
    drop(fetched);
    assert!(!dir.exists(), "the temporary directory outlived its handle");
}

#[test]
fn a_redirect_is_followed() {
    let base = serve(2);
    let fetched = headwater_fetch::fetch(&format!("{base}/moved")).unwrap();
    assert!(fetched.path().join("headwater-package.yml").is_file());
}

#[test]
fn an_error_status_is_a_transport_error_that_names_the_location() {
    let base = serve(1);
    let location = format!("{base}/missing.zip");
    let error = headwater_fetch::fetch(&location).unwrap_err();
    assert!(
        matches!(error, headwater_fetch::Error::Transport(_)),
        "{error}"
    );
    assert!(error.to_string().contains(&location), "{error}");
}
