mod document;
mod ffi;
mod value;

pub use document::Document;
pub use value::{Value, ValueType};

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn test_parse_and_traverse() {
        let src = "server\n| host = localhost\n| port = 8080\n";
        let doc = Document::from_str_or_panic(src);
        let root = doc.root().expect("no root");

        let server = root.get("server").expect("no server");
        let host = server.get("host").expect("no host");
        let port = server.get("port").expect("no port");

        assert_eq!(host.as_str().unwrap(), "localhost");
        assert_eq!(port.as_f64().unwrap(), 8080.0);
    }

    #[test]
    fn test_from_file() {
        let path = PathBuf::from_str("../../example.spn");
        assert!(path.is_ok());
        let path = path.unwrap();
        let value = Document::from_path(path);
        assert!(value.is_ok());
        let value = value.unwrap();
        println!("{value:?}");
    }

    #[test]
    fn test_error_handling() {
        let src = "host = localhost\n\nyay = paru\n\n\nhost = example.com\n";
        let result = Document::from_str(src);
        assert!(result.is_err());
        let document::DocError::Parse(errors) = result.unwrap_err() else {
            return;
        };
        assert!(!errors.is_empty());
        println!("{}", errors[0]);
    }

    #[test]
    fn test_multi_error_handling() {
        let src = "age = 18\nemail = alice@tuta.io\n\nage = 12\nemail = alice@example.com";
        let result = Document::from_str(src);
        assert!(result.is_err());
        let document::DocError::Parse(errors) = result.unwrap_err() else {
            return;
        };
        assert!(!errors.is_empty());
        println!("{}", errors[0]);
    }
}
