use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug)]
pub enum ExplicitMessageType {
    ReadTag,
    WriteTag,
    Unknown,
}

#[derive(Debug)]
pub struct ExplicitMessage {
    pub msg_type: ExplicitMessageType,
    pub connection_id: u32,
    pub data: Vec<u8>,
}

impl ExplicitMessage {
    pub fn parse(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 6 {
            return None;
        }

        let command = buf.get_u8();
        let connection_id = buf.get_u32_le();
        let data = buf.split().to_vec();

        let msg_type = match command {
            0x01 => ExplicitMessageType::ReadTag,
            0x02 => ExplicitMessageType::WriteTag,
            _ => ExplicitMessageType::Unknown,
        };

        Some(Self {
            msg_type,
            connection_id,
            data,
        })
    }

    pub fn to_response(&self, success: bool, response_data: Vec<u8>) -> Vec<u8> {
        let mut response = BytesMut::with_capacity(8 + response_data.len());
        response.put_u8(if success { 0x10 } else { 0x20 }); // Success or Failure
        response.put_u32_le(self.connection_id);
        response.extend_from_slice(&response_data);
        response.to_vec()
    }
}
