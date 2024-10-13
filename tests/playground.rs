#[derive(Debug)]
struct M<T: ?Sized>(T);

impl M<[u8]> {
    pub fn new(data: &[u8]) -> Option<&Self> {
          // the back!
          let piece = &data[..3] as *const [u8] as *const Self;
          // Safety: Piece is a POD with repr(c), _and_ the fat pointer data length is the length of
          // the trailing DST field (thanks to the PIECE_LEAD offset).
          Some(unsafe { &*piece })
    }
}

#[test]
fn test() {
    let a: u8 = b'a';
    let b: char = 'a';
    let c: &[u8; 2] = b"aa";
    let d: &[u8] = &c[..];
    let e: &str = "我";
    let f: &[u8] = e.as_bytes();
    let g = M::new(f).unwrap();
    println!(
        r#"value => u8 {:?}、 char {}、 &[u8; 2] {:?}、 &[u8] {}、 &str {:?}、 &[u8] {:?} "#,
        g, b, c, e, f, d,
    );
    println!(
        r#"size =>  u8 {}、 char {}、 &[u8; 2] {}、 &[u8] {}、 &str {}、 &[u8] {} "#,
        size_of_val(&g),
        size_of_val(&b),
        size_of_val(&c),
        size_of_val(&d),
        size_of_val(&e),
        size_of_val(&f),
    );
    println!(
        r#"size =>  u8 {}、 char {}、 &[u8; 2] {}、 &[u8] {}、 &str {}、 f&[u8] {} "#,
        size_of_val(&g),
        size_of_val(&b),
        size_of_val(c),
        size_of_val(d),
        size_of_val(e),
        size_of_val(f),
    );
}
