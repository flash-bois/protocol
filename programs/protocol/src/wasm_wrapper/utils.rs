use js_sys::Uint8Array;

pub fn to_buffer(v: &[u8]) -> Uint8Array {
    let b = Uint8Array::new_with_length(32);
    b.copy_from(v);
    b
}
