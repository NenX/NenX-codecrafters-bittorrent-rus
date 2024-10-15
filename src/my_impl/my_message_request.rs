#[derive(Debug)]
#[repr(C)]
pub struct MyRequestPayload {
    pub index: [u8; 4],
    pub begin: [u8; 4],
    pub length: [u8; 4],
}
impl MyRequestPayload {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        let index = index.to_be_bytes();
        let begin = begin.to_be_bytes();
        let length = length.to_be_bytes();
        

        Self {
            index,
            begin,
            length,
        }
    }
    pub fn to_bytes(&self) -> &[u8] {
        let a = self as *const Self as *const [u8; std::mem::size_of::<Self>()];
        
        (unsafe { &*a }) as _
    }
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < std::mem::size_of::<Self>() {
            return None;
        }
        let a = data as *const [u8] as *const Self;
        let a = unsafe { &*a };
        Some(a)
    }
}


#[derive(Debug)]
#[repr(C)]
pub struct QQ {
    pub index: [u8; 4],
    pub begin: [u8; 4],
    pub length: [u8; 4],
}
impl QQ {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        let index = index.to_be_bytes();
        let begin = begin.to_be_bytes();
        let length = length.to_be_bytes();
        

        Self {
            index,
            begin,
            length,
        }
    }
    pub fn to_bytes(&self) -> &[u8] {
        let a = self as *const Self as *const [u8; std::mem::size_of::<Self>()];
        
        (unsafe { &*a }) as _
    }
    pub fn ref_from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < std::mem::size_of::<Self>() {
            return None;
        }
        let a = data as *const [u8] as *const Self;
        let a = unsafe { &*a };
        Some(a)
    }
}
