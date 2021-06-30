use std::process::exit;

pub trait VecExtension {
    fn log(self);
    fn log_and_die_if_exists(self);
    fn get_string(self) -> String;
    fn get_string_or_die(self) -> String;
    fn log_and_get_string(self) -> String;
    fn log_and_get_string_or_die(self) -> String;
}

impl VecExtension for Vec<u8> {
    fn log(self) {
        String::from_utf8(self)
            .map(|s| {
                println!("{}", s);
            })
            .map_err(|e| println!("Parse error: {}", e.to_string()))
            .unwrap();
    }

    fn log_and_die_if_exists(self) {
        String::from_utf8(self)
            .map(|s| {
                println!("{}", s);
                if s.len() > 0 {
                    exit(1);
                }
            })
            .map_err(|e| println!("Parse error: {}", e.to_string()))
            .unwrap();
    }

    fn get_string(self) -> String {
        return String::from_utf8(self)
            .map(|s| {
                return s;
            })
            .map_err(|_e| {
                return "";
            })
            .unwrap();
    }

    fn get_string_or_die(self) -> String {
        return String::from_utf8(self).expect("Parse error!");
    }

    fn log_and_get_string(self) -> String {
        return String::from_utf8(self)
            .map(|s| {
                println!("{}", s);
                return s;
            })
            .map_err(|e| {
                println!("Parse error: {}", e.to_string());
                return "";
            })
            .unwrap();
    }

    fn log_and_get_string_or_die(self) -> String {
        return String::from_utf8(self)
            .map(|s| {
                println!("{}", s);
                return s;
            })
            .expect("Parse error!");
    }
}
