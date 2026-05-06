pub fn append_svg_header(svg: &mut String, width: u32, height: u32) {
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    ));
}

pub fn append_background_image(svg: &mut String, image_url: &str, width: u32, height: u32) {
    svg.push_str(&format!(
        r#"<image href="{}" x="0" y="0" width="{width}" height="{height}" preserveAspectRatio="none"/>"#,
        escape_xml_attr(image_url),
    ));
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);
        let value = ((first as u32) << 16) | ((second as u32) << 8) | third as u32;

        encoded.push(TABLE[((value >> 18) & 0x3f) as usize] as char);
        encoded.push(TABLE[((value >> 12) & 0x3f) as usize] as char);

        if chunk.len() > 1 {
            encoded.push(TABLE[((value >> 6) & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }

        if chunk.len() > 2 {
            encoded.push(TABLE[(value & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }

    encoded
}
