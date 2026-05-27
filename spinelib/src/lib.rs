mod document;
mod ffi;
mod value;

pub use document::Document;
pub use value::{Value, ValueType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_traverse() {
        let src = "server\n| host = localhost\n| port = 8080\n";
        let doc = Document::parse(src).expect("parse failed");
        let root = doc.root().expect("no root");

        let server = root.get("server").expect("no server");
        let host = server.get("host").expect("no host");
        let port = server.get("port").expect("no port");

        assert_eq!(host.as_str().unwrap(), "localhost");
        assert_eq!(port.as_f64().unwrap(), 8080.0);
    }

    #[test]
    fn test_error_handling() {
        let src = "host = localhost\nhost = example.com\n";
        let result = Document::parse(src);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        println!("{}", errors[0]);
    }
}
