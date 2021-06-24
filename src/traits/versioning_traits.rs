pub trait Versioning {
    fn is_more_recent(self, than: &str) -> bool;
}

impl Versioning for &str {
    fn is_more_recent(self, than: &str) -> bool {
        let base_version = self
            .to_string()
            .split('.')
            .map(|v| v.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        let than_version = than
            .to_string()
            .split('.')
            .map(|v| v.parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
        for i in 0..base_version.len() {
            return &base_version[i] > &than_version[i];
        }
        return false;
    }
}
