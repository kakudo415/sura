pub enum Events {
  BackSpace,
  CarriageReturn,
  LineFeed,
  DeviceControl1, // ^Q
  DeviceControl3, // ^S
  Delete,
  CursorUp,
  CursorDown,
  CursorForward,
  CursorBack,
  Character(char),
}

pub struct Input {
  inner: [u8; 3],
  len: usize,
}

impl Input {
  pub fn new() -> Self {
    Input {
      inner: [0, 0, 0],
      len: 0,
    }
  }

  fn push(&mut self, ch: u8) {
    self.inner[self.len] = ch;
    self.len += 1;
  }

  pub fn read(&mut self) {
    let mut buf = [0; 1];
    loop {
      if unsafe { libc::read(0, (&mut buf).as_ptr() as *mut libc::c_void, 1) } > 0 {
        self.push((&mut buf)[0]);
        return;
      }
    }
  }

  pub fn event(&mut self) -> Events {
    loop {
      self.read();
      match self.inner {
        [08, 00, 00] => return Events::BackSpace,
        [13, 00, 00] => return Events::CarriageReturn,
        [10, 00, 00] => return Events::LineFeed,
        [17, 00, 00] => return Events::DeviceControl1,
        [19, 00, 00] => return Events::DeviceControl3,
        [127, 00, 00] => return Events::Delete,
        [27, 91, 65] => return Events::CursorUp,
        [27, 91, 66] => return Events::CursorDown,
        [27, 91, 67] => return Events::CursorForward,
        [27, 91, 68] => return Events::CursorBack,
        [27, _, _] => continue,
        ch => {
          return Events::Character(char::from_u32(ch[0] as u32).unwrap());
        }
      }
    }
  }
}
