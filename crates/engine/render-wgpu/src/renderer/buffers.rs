pub fn vertices_as_bytes(vertices: &[super::ColorVertex]) -> &[u8] {
    let byte_len = std::mem::size_of_val(vertices);
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast::<u8>(), byte_len) }
}

pub fn texture_vertices_as_bytes(vertices: &[super::TextureVertex]) -> &[u8] {
    let byte_len = std::mem::size_of_val(vertices);
    unsafe { std::slice::from_raw_parts(vertices.as_ptr().cast::<u8>(), byte_len) }
}
