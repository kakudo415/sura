use super::*;

#[cfg(target_os = "windows")]
pub struct Terminal {}

#[cfg(target_os = "windows")]
impl Terminal {
    pub fn open() -> Self {
        todo!()
    }

    pub fn close(&mut self) {
        todo!()
    }
}
