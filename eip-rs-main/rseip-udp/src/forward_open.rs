use bytes::{Buf, BytesMut};

#[derive(Debug)]
pub struct ForwardOpenRequest {
    pub connection_id: u32,
    pub originator_serial_number: u32,
    pub timeout_ticks: u8,
    pub requested_packet_interval: u32,
    pub transport_trigger: u8,
}

impl ForwardOpenRequest {
    pub fn parse(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 16 {
            return None;
        }

        Some(Self {
            connection_id: buf.get_u32_le(),
            originator_serial_number: buf.get_u32_le(),
            timeout_ticks: buf.get_u8(),
            requested_packet_interval: buf.get_u32_le(),
            transport_trigger: buf.get_u8(),
        })
    }
}
