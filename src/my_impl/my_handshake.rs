#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MyHandShakeData {
    pub length: u8,
    pub bittorrent: [u8; 19],
    pub reserved: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl MyHandShakeData {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            bittorrent: *b"BitTorrent protocol",
            reserved: [0; 8],
            info_hash,
            peer_id,
        }
    }
    pub fn set_ext_reserved_bit(&mut self) -> &Self {
        let item = self.reserved.get_mut(5).unwrap();
        *item = 0x10;
        self
    }
    pub fn has_ext_reserved_bit(&self) -> bool {
        let reserved = self.reserved;

        let item = reserved.get(5).unwrap();

        println!("ffw {:?}", reserved);

        *item == 0x10
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        // Safety: Handshake is a POD with repr(c)
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
}
