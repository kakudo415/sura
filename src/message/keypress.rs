#[derive(PartialEq, Eq, Debug)]
pub enum KeyPress {
    Control(char),
    Character(char),

    Delete,

    CursorUp,
    CursorDown,
    CursorForward,
    CursorBack,
}

impl KeyPress {
    pub const BS: KeyPress = KeyPress::Control('H');
    pub const CR: KeyPress = KeyPress::Control('M');
    pub const LF: KeyPress = KeyPress::Control('J');
}
