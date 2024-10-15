

#[derive(Debug)]
#[repr(C)]
pub struct MyPiecePayload<T: ?Sized = [u8]> {
    pub index: [u8; 4],
    pub begin: [u8; 4],
    pub block: T,
}
impl MyPiecePayload {
    pub fn to_bytes(&self) -> Vec<u8> {
        let a = self as *const Self as *const [u8; Self::PIECE_SIZE];
        let a = unsafe { &*a };
        let v: Vec<_> = a.iter().chain(self.block.iter()).cloned().collect();
        v
    }
    const PIECE_SIZE: usize = std::mem::size_of::<MyPiecePayload<()>>();
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::PIECE_SIZE {
            return None;
        }
        let correct_len = data.len() - Self::PIECE_SIZE;
        let fat_pointer_with_correct_len = &data[..correct_len] as *const [u8] as *const Self;
        let a = unsafe { &*fat_pointer_with_correct_len };
        Some(a)
    }
}
