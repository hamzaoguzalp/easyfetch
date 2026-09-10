use std::fmt::Display;

macro_rules! style_fn {
    ($name:ident, $code:expr) => {
        fn $name(&self) -> String {
            format!("\x1b[{}m{}\x1b[0m", $code, self)
        }
    };
}

pub trait Colorize: Display {
    // Foreground Colors
    style_fn!(black, "30");
    style_fn!(red, "31");
    style_fn!(green, "32");
    style_fn!(yellow, "33");
    style_fn!(blue, "34");
    style_fn!(magenta, "35");
    style_fn!(cyan, "36");
    style_fn!(white, "37");
    style_fn!(dark_grey, "90");

    // Background Colors
    style_fn!(on_black, "40");
    style_fn!(on_red, "41");
    style_fn!(on_green, "42");
    style_fn!(on_yellow, "43");
    style_fn!(on_blue, "44");
    style_fn!(on_magenta, "45");
    style_fn!(on_cyan, "46");
    style_fn!(on_white, "47");
    style_fn!(on_dark_grey, "100");

    // Text Styles
    style_fn!(bold, "1");
    style_fn!(dim, "2");
    style_fn!(italic, "3");
    style_fn!(underline, "4");

    // Custom RGB / Hex
    fn color_fg_rgb(&self, r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{r};{g};{b}m{}\x1b[0m", self)
    }

    fn color_fg_hex(&self, hex_code: &str) -> String {
        let hex_str = hex_code.trim()
                                    .trim_start_matches('#')
                                    .trim_start_matches("0x");
        if hex_code.len() != 6 {
            return self.to_string();
        }
        let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap_or(255);
        
        self.color_fg_rgb(r, g, b)
    }

    fn color_bg_rgb(&self, r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{r};{g};{b}m{}\x1b[0m", self)
    }

    fn color_bg_hex(&self, hex_code: &str) -> String {
        let hex_str = hex_code.trim()
                                    .trim_start_matches('#')
                                    .trim_start_matches("0x");
        
        if hex_code.len() != 6 { return self.to_string(); }
        
        let r = u8::from_str_radix(&hex_str[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex_str[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex_str[4..6], 16).unwrap_or(255);
        self.color_bg_rgb(r, g, b)
    }
}

impl<T: Display + ?Sized> Colorize for T { }
