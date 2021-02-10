/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

 use core::ops::{Deref, DerefMut};

use bitfield::{bitfield_bitrange, bitfield_fields};
use byte::{check_len, BytesExt, TryRead, TryWrite};
use heapless::{
  consts::*,
  i,
  spsc::{Consumer, Producer, Queue},
  ArrayLength, String, Vec,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Flags(u8);
bitfield_bitrange! {struct Flags(u8)}

impl Flags {
  bitfield_fields! {
    u8;
    pub dup, set_dup: 7;
    pub qos, set_qos: 6, 5;
    pub retain, set_retain: 4;
    pub will, set_will: 3;
    pub clean_session, set_clean_session: 2;
    pub topic_id_type, set_topic_id_type: 1, 0;
  }
}

impl TryWrite for Flags {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, self.0)?;
    Ok(*offset)
  }
}

impl TryRead<'_> for Flags {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    Ok((Flags(bytes.read::<u8>(&mut 0)?), 1))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReturnCode {
  Accepted,
  Rejected(RejectedReason),
}

impl TryWrite for ReturnCode {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(
      offset,
      match self {
        ReturnCode::Accepted => 0u8,
        ReturnCode::Rejected(RejectedReason::Congestion) => 1u8,
        ReturnCode::Rejected(RejectedReason::InvalidTopicId) => 2u8,
        ReturnCode::Rejected(RejectedReason::NotSupported) => 3u8,
        ReturnCode::Rejected(RejectedReason::Reserved(n)) => n,
      },
    )?;
    Ok(*offset)
  }
}

impl TryRead<'_> for ReturnCode {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    Ok((
      match bytes.read::<u8>(offset)? {
        0 => ReturnCode::Accepted,
        1 => ReturnCode::Rejected(RejectedReason::Congestion),
        2 => ReturnCode::Rejected(RejectedReason::InvalidTopicId),
        3 => ReturnCode::Rejected(RejectedReason::NotSupported),
        n => ReturnCode::Rejected(RejectedReason::Reserved(n)),
      },
      *offset,
    ))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RejectedReason {
  Congestion,
  InvalidTopicId,
  NotSupported,
  Reserved(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForwardedMessage {
  pub ctrl: u8,
  pub wireless_node_id: WirelessNodeId,
  pub message: Message,
}

impl TryWrite for ForwardedMessage {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 3 + self.wireless_node_id.len() as u8); // len
    bytes.write(offset, 0xFEu8); // msg type
    bytes.write(offset, self.ctrl);
    bytes.write(offset, self.wireless_node_id.as_str());
    bytes.write(offset, self.message);
    Ok(*offset)
  }
}

impl TryRead<'_> for ForwardedMessage {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    bytes.read::<u8>(offset)?; // msg type
    Ok((
      ForwardedMessage {
        ctrl: bytes.read(offset)?,
        wireless_node_id: bytes.read_with(offset, len as usize - 3)?,
        message: bytes.read(offset)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WirelessNodeId(heapless::String<U16>);

impl WirelessNodeId {
  fn from(s: &str) -> Self { Self(String::from(s)) }
}

impl Deref for WirelessNodeId {
  type Target = heapless::String<U16>;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for WirelessNodeId {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TryWrite for WirelessNodeId {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, self.as_str())?;
    Ok(*offset)
  }
}

impl TryRead<'_, usize> for WirelessNodeId {
  fn try_read(bytes: &[u8], len: usize) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let mut s = String::new();
    s.push_str(bytes.read_with(offset, byte::ctx::Str::Len(len))?)
      .map_err(|e| byte::Error::BadInput {
        err: "wireless_node_id longer than 16 bytes",
      })?;
    Ok((WirelessNodeId(s), *offset))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Message {
  SearchGw(SearchGw),
  GwInfo(GwInfo),
  Connect(Connect),
  ConnAck(ConnAck),
  Register(Register),
  RegAck(RegAck),
  Publish(Publish),
  PingReq(PingReq),
  PingResp(PingResp),
}

impl TryWrite for Message {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    match self {
      Message::SearchGw(msg) => bytes.write(offset, msg),
      Message::GwInfo(msg) => bytes.write(offset, msg),
      Message::Connect(msg) => bytes.write(offset, msg),
      Message::ConnAck(msg) => bytes.write(offset, msg),
      Message::Register(msg) => bytes.write(offset, msg),
      Message::RegAck(msg) => bytes.write(offset, msg),
      Message::Publish(msg) => bytes.write(offset, msg),
      Message::PingReq(msg) => bytes.write(offset, msg),
      Message::PingResp(msg) => bytes.write(offset, msg),
    }?;
    Ok(*offset)
  }
}

impl TryRead<'_> for Message {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    // Not increasing offset because some messages needs access to len.
    Ok((
      match bytes.read::<u8>(&mut (*offset + 1))? {
        0x01 => Message::SearchGw(bytes.read(offset)?),
        0x02 => Message::GwInfo(bytes.read(offset)?),
        0x04 => Message::Connect(bytes.read(offset)?),
        0x05 => Message::ConnAck(bytes.read(offset)?),
        0x0a => Message::Register(bytes.read(offset)?),
        0x0b => Message::RegAck(bytes.read(offset)?),
        0x0c => Message::Publish(bytes.read(offset)?),
        0x16 => Message::PingReq(bytes.read(offset)?),
        0x17 => Message::PingResp(bytes.read(offset)?),
        t => {
          return Err(byte::Error::BadInput {
            err: "Recieved a message with unknown type",
          })
        }
      },
      *offset,
    ))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SearchGw {
  pub radius: u8,
}

impl TryWrite for SearchGw {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 3u8); // len
    bytes.write(offset, 0x01u8); // msg type
    bytes.write(offset, self.radius);
    Ok(*offset)
  }
}

impl TryRead<'_> for SearchGw {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    *offset += 1; // msg type
    Ok((
      SearchGw {
        radius: bytes.read(offset)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GwInfo {
  pub gw_id: u8,
}

impl TryWrite for GwInfo {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 3u8); // len
    bytes.write(offset, 0x02u8); // msg type
    bytes.write(offset, self.gw_id);
    Ok(*offset)
  }
}

impl TryRead<'_> for GwInfo {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    *offset += 1; // msg type
    Ok((
      GwInfo {
        gw_id: bytes.read(offset)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Connect {
  pub flags: Flags,
  pub duration: u16,
  pub client_id: ClientId,
}

impl TryWrite for Connect {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    let len = 6 + self.client_id.len() as u8;
    bytes.write(offset, len);
    bytes.write(offset, 0x04u8); // msg type
    bytes.write(offset, self.flags);
    bytes.write(offset, 0x01u8); // protocol id
    bytes.write_with(offset, self.duration, byte::ctx::BE);
    bytes.write(offset, self.client_id.as_str());
    Ok(*offset)
  }
}

impl TryRead<'_> for Connect {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    if len < 6 {
      return Err(byte::Error::BadInput {
        err: "Connect len must be >= 6 bytes",
      });
    }
    *offset += 1; // msg type
    let flags = bytes.read(offset)?;
    bytes.read::<u8>(offset)?; // protocol id
    Ok((
      Connect {
        flags,
        duration: bytes.read_with(offset, byte::ctx::BE)?,
        client_id: bytes.read_with(offset, len as usize - 6)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClientId(heapless::String<U16>);

impl ClientId {
  fn from(s: &str) -> Self { Self(String::from(s)) }
}

impl Deref for ClientId {
  type Target = heapless::String<U16>;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for ClientId {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TryWrite for ClientId {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, self.as_str())?;
    Ok(*offset)
  }
}

impl TryRead<'_, usize> for ClientId {
  fn try_read(bytes: &[u8], len: usize) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let mut s = String::new();
    s.push_str(bytes.read_with(offset, byte::ctx::Str::Len(len))?)
      .map_err(|e| byte::Error::BadInput {
        err: "client_id longer than 16 bytes",
      })?;
    Ok((ClientId(s), *offset))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConnAck {
  pub code: ReturnCode,
}

impl TryWrite for ConnAck {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 3u8); // len
    bytes.write(offset, 0x05u8); // msg type
    bytes.write(offset, self.code);
    Ok(*offset)
  }
}

impl TryRead<'_> for ConnAck {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    *offset += 1; // msg type
    Ok((
      ConnAck {
        code: bytes.read(offset)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Register {
  pub topic_id: u16,
  pub msg_id: u16,
  pub topic_name: TopicName,
}

impl TryWrite for Register {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    let len = 6 + self.topic_name.len() as u8;
    bytes.write(offset, len);
    bytes.write(offset, 0x0Au8); // msg type
    bytes.write_with(offset, self.topic_id, byte::ctx::BE);
    bytes.write_with(offset, self.msg_id, byte::ctx::BE);
    bytes.write(offset, self.topic_name.as_str());
    Ok(*offset)
  }
}

impl TryRead<'_> for Register {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    if len < 6 {
      return Err(byte::Error::BadInput {
        err: "Register len must be >= 6 bytes",
      });
    }
    *offset += 1; // msg type
    Ok((
      Register {
        topic_id: bytes.read_with(offset, byte::ctx::BE)?,
        msg_id: bytes.read_with(offset, byte::ctx::BE)?,
        topic_name: bytes.read_with(offset, len as usize - 6)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TopicName(heapless::String<U256>);

impl TopicName {
  fn from(s: &str) -> Self { Self(String::from(s)) }
}

impl Deref for TopicName {
  type Target = heapless::String<U256>;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for TopicName {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TryWrite for TopicName {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, self.as_str())?;
    Ok(*offset)
  }
}

impl TryRead<'_, usize> for TopicName {
  fn try_read(bytes: &[u8], len: usize) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let mut s = String::new();
    s.push_str(bytes.read_with(offset, byte::ctx::Str::Len(len))?)
      .map_err(|e| byte::Error::BadInput {
        err: "topic_name longer than 256 bytes",
      })?;
    Ok((TopicName(s), *offset))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RegAck {
  pub topic_id: u16,
  pub msg_id: u16,
  pub code: ReturnCode,
}

impl TryWrite for RegAck {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 7u8); // len
    bytes.write(offset, 0xBu8); // msg type
    bytes.write_with(offset, self.topic_id, byte::ctx::BE);
    bytes.write_with(offset, self.msg_id, byte::ctx::BE);
    bytes.write(offset, self.code);
    Ok(*offset)
  }
}

impl TryRead<'_> for RegAck {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    *offset += 1; // msg type
    Ok((
      RegAck {
        topic_id: bytes.read_with(offset, byte::ctx::BE)?,
        msg_id: bytes.read_with(offset, byte::ctx::BE)?,
        code: bytes.read(offset)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Publish {
  pub flags: Flags,
  pub topic_id: u16,
  pub msg_id: u16,
  pub data: PublishData,
}

impl TryWrite for Publish {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    let len = 7 + self.data.len() as u8;
    bytes.write(offset, len);
    bytes.write(offset, 0x0Cu8); // msg type
    bytes.write(offset, self.flags);
    bytes.write_with(offset, self.topic_id, byte::ctx::BE);
    bytes.write_with(offset, self.msg_id, byte::ctx::BE);
    bytes.write(offset, self.data.as_str());
    Ok(*offset)
  }
}

impl TryRead<'_> for Publish {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    if len < 7 {
      return Err(byte::Error::BadInput {
        err: "Publish len must be >= 6 bytes",
      });
    }
    *offset += 1; // msg type
    Ok((
      Publish {
        flags: bytes.read(offset)?,
        topic_id: bytes.read_with(offset, byte::ctx::BE)?,
        msg_id: bytes.read_with(offset, byte::ctx::BE)?,
        data: bytes.read_with(offset, len as usize - 7)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PublishData(heapless::String<U256>);

impl PublishData {
  fn from(s: &str) -> Self { Self(String::from(s)) }
}

impl Deref for PublishData {
  type Target = heapless::String<U256>;

  fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for PublishData {
  fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl TryWrite for PublishData {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, self.as_str())?;
    Ok(*offset)
  }
}

impl TryRead<'_, usize> for PublishData {
  fn try_read(bytes: &[u8], len: usize) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let mut s = String::new();
    s.push_str(bytes.read_with(offset, byte::ctx::Str::Len(len))?)
      .map_err(|e| byte::Error::BadInput {
        err: "data longer than 256 bytes",
      })?;
    Ok((PublishData(s), *offset))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PingReq {
  pub client_id: ClientId,
}

impl TryWrite for PingReq {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    let len = 2 + self.client_id.len() as u8;
    bytes.write(offset, len);
    bytes.write(offset, 0x16u8); // msg type
    bytes.write(offset, self.client_id.as_str());
    Ok(*offset)
  }
}

impl TryRead<'_> for PingReq {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    if len < 2 {
      return Err(byte::Error::BadInput {
        err: "Len must be at least 2 bytes",
      });
    }
    *offset += 1; // msg type
    Ok((
      PingReq {
        client_id: bytes.read_with(offset, len as usize - 2)?,
      },
      *offset,
    ))
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PingResp {}

impl TryWrite for PingResp {
  fn try_write(self, bytes: &mut [u8], _ctx: ()) -> byte::Result<usize> {
    let offset = &mut 0;
    bytes.write(offset, 2u8); // len
    bytes.write(offset, 0x17u8); // msg type
    Ok(*offset)
  }
}

impl TryRead<'_> for PingResp {
  fn try_read(bytes: &[u8], _ctx: ()) -> byte::Result<(Self, usize)> {
    let offset = &mut 0;
    let len: u8 = bytes.read(offset)?;
    check_len(bytes, len as usize)?;
    *offset += 1; // msg type
    Ok((PingResp {}, *offset))
  }
}

#[cfg(test)]
mod tests {
  use assert_hex::*;

  use super::*;

  #[test]
  fn forwarded_message_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = ForwardedMessage {
      ctrl: 0,
      wireless_node_id: WirelessNodeId::from("test-node"),
      message: Message::PingResp(PingResp {}),
    };
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(
      &bytes[..len],
      &[12u8, 0xfe, 0x00, b't', b'e', b's', b't', b'-', b'n', b'o', b'd', b'e', 2, 0x17]
    );
    let actual: ForwardedMessage = bytes.read(&mut 0).unwrap();
    assert_eq_hex!(actual, expected);
  }

  #[test]
  fn return_code_encode() {
    let mut buf = [0u8; 5];
    let mut offset = 0usize;
    buf.write(&mut offset, ReturnCode::Accepted);
    buf.write(
      &mut offset,
      ReturnCode::Rejected(RejectedReason::Congestion),
    );
    buf.write(
      &mut offset,
      ReturnCode::Rejected(RejectedReason::InvalidTopicId),
    );
    buf.write(
      &mut offset,
      ReturnCode::Rejected(RejectedReason::NotSupported),
    );
    buf.write(
      &mut offset,
      ReturnCode::Rejected(RejectedReason::Reserved(0x12)),
    );
    assert_eq_hex!(&buf, &[0x00u8, 0x01u8, 0x02u8, 0x03u8, 0x12u8]);
  }

  #[test]
  fn return_code_parse() {
    let buf = &[0x00u8, 0x01u8, 0x02u8, 0x03u8, 0x12u8];
    let mut actual = [ReturnCode::Accepted; 5];
    let mut offset = 0usize;
    for i in 0..5 {
      actual[i] = buf.read(&mut offset).unwrap();
    }
    assert_eq!(
      &actual,
      &[
        ReturnCode::Accepted,
        ReturnCode::Rejected(RejectedReason::Congestion),
        ReturnCode::Rejected(RejectedReason::InvalidTopicId),
        ReturnCode::Rejected(RejectedReason::NotSupported),
        ReturnCode::Rejected(RejectedReason::Reserved(0x12)),
      ]
    );
  }

  #[test]
  fn searchgw_encode_parse() {
    let bytes = &mut [0u8; 10];
    let mut len = 0usize;
    let expected = Message::SearchGw(SearchGw { radius: 5 });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(&bytes[..len], [0x03u8, 0x01, 0x05]);
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn gwinfo_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::GwInfo(GwInfo { gw_id: 0x12 });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(&bytes[..len], [0x03u8, 0x02, 0x12]);
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn connect_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::Connect(Connect {
      flags: Flags(0x12),
      duration: 0x3456,
      client_id: ClientId::from("test-client"),
    });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(
      &bytes[..len],
      [
        0x11u8, 0x04, 0x12, 0x01, 0x34, 0x56, b't', b'e', b's', b't', b'-', b'c', b'l', b'i', b'e',
        b'n', b't'
      ]
    );
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn register_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::Register(Register {
      topic_id: 0x1234,
      msg_id: 0x5678,
      topic_name: TopicName::from("test"),
    });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(
      &bytes[..len],
      [0x0au8, 0x0a, 0x12, 0x34, 0x56, 0x78, b't', b'e', b's', b't']
    );
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn regack_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::RegAck(RegAck {
      topic_id: 0x1234,
      msg_id: 0x5678,
      code: ReturnCode::Rejected(RejectedReason::Congestion),
    });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(&bytes[..len], [0x07u8, 0x0b, 0x12, 0x34, 0x56, 0x78, 0x1]);
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn publish_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::Publish(Publish {
      flags: Flags(0x12),
      topic_id: 0x1234,
      msg_id: 0x5678,
      data: PublishData::from("test"),
    });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(
      &bytes[..len],
      [0x0bu8, 0x0c, 0x12, 0x12, 0x34, 0x56, 0x78, b't', b'e', b's', b't']
    );
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn pingreq_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::PingReq(PingReq {
      client_id: ClientId::from("test-client"),
    });
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(
      &bytes[..len],
      [0xdu8, 0x16, b't', b'e', b's', b't', b'-', b'c', b'l', b'i', b'e', b'n', b't']
    );
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }

  #[test]
  fn pingresp_encode_parse() {
    let mut bytes = [0u8; 20];
    let mut len = 0usize;
    let expected = Message::PingResp(PingResp {});
    let mut len = 0usize;
    bytes.write(&mut len, expected.clone()).unwrap();
    assert_eq_hex!(&bytes[..len], &[0x02u8, 0x17]);
    let actual: Message = bytes.read(&mut 0).unwrap();
    assert_eq!(actual, expected);
  }
}
