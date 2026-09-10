pub mod key {
    pub const QUIT: u8 = b'q';
    pub const NAVIGATE_UP: u8 = b'k';
    pub const NAVIGATE_DOWN: u8 = b'j';
    pub const BACKSPACE: u8 = 127;
    pub const BACKSPACE_ALTERNATIVE: u8 = 8;
    pub const ENTER: u8 = b'\n';
    pub const ENTER_ALTERNATIVE: u8 = b'\r';
    pub const EDIT_QUERY: u8 = b's';
    pub const SPACE: u8 = b' ';
}

pub mod layout {
    pub const MARGIN: u16 = 2;
    pub const PADDING: u16 = 1;
    pub const SPACING: u16 = 1;
    pub const MAX_RESULTS: usize = 10;
}
