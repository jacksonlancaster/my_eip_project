use bytes::{Buf, BytesMut};

#[derive(Debug)]
pub struct ForwardCloseRequest {
    pub connection_id: u32,
    pub originator_serial_number: u32,
}

impl ForwardCloseRequest {
    pub fn parse(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }

        Some(Self {
            connection_id: buf.get_u32_le(),
            originator_serial_number: buf.get_u32_le(),
        })
    }
}
