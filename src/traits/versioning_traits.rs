pub trait Versioning {
    fn is_more_recent(self, than: &str) -> bool;
    fn split_dot_version(self) -> Vec<u8>;
    fn strip_dash_variant(self) -> String;
}

impl Versioning for &str {
    fn is_more_recent(self, than: &str) -> bool {
        let base_version = self.split_dot_version();
        let than_version = than.split_dot_version();
        for i in 0..base_version.len() {
            return &base_version[i] > &than_version[i];
        }
        return false;
    }

    fn split_dot_version(self) -> Vec<u8> {
        return self
            .to_string()
            .split('.')
            .map(|v| v.strip_dash_variant().parse::<u8>().unwrap())
            .collect::<Vec<u8>>();
    }

    fn strip_dash_variant(self) -> String {
        return self
            .to_string()
            .split('-')
            .next()
            .unwrap_or("0")
            .to_string();
    }
}
