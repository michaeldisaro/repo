pub trait StringExtension {
    fn strip(self) -> String;
}

impl StringExtension for &String {
    fn strip(self) -> String {
        self.to_string().trim().replace("\"", "")
    }
}

impl StringExtension for &serde_json::Value {
    fn strip(self) -> String {
        self.to_string().trim().replace("\"", "")
    }
}
