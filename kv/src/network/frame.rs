use std::io::{Read, Write};

use bytes::{Buf, BufMut, BytesMut};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use prost::Message;

use crate::{CommandRequest, CommandResponse, error::KvError};
//长度占用4个字节
pub const LEN_LEN: usize = 4;
//长度占31bit，最大frame为2GB
const MAX_FRAME: usize = 2 * 1024 * 1024 * 1024;
//payload超过1436字节时，进行压缩
const COMPRESSION_LIMIT: usize = 1436;

const COMPRESSION_BIT: usize = 1 << 31;

#[allow(unused)]
pub trait FrameCoder
where
    Self: Message + Sized + Default,
{
    fn encode_frame(&self, buf: &mut BytesMut) -> Result<(), KvError> {
        let size: usize = self.encoded_len();
        if size >= MAX_FRAME {
            return Err(KvError::FrameError);
        }
        buf.put_u32(size as _);
        if size > COMPRESSION_LIMIT {
            let mut buf1 = Vec::with_capacity(size);
            self.encode(&mut buf1)?;
            let palyload = buf.split_off(LEN_LEN);
            buf.clear();
            let mut encoder = GzEncoder::new(palyload.writer(), Compression::default());
            encoder.write_all(&buf1[..])?;
            let payload = encoder.finish()?.into_inner();
            buf.put_u32((payload.len() | COMPRESSION_BIT) as _);
            buf.unsplit(payload);
        } else {
            self.encode(buf)?;
        }
        Ok(())
    }

    fn decode_frame(buf: &mut BytesMut) -> Result<Self, KvError> {
        let header = buf.get_u32() as usize;
        let (len, compressed) = decode_header(header);
        if compressed {
            let mut decoder = GzDecoder::new(&buf[..len]);
            let mut buf1 = Vec::with_capacity(len * 2);
            decoder.read_to_end(&mut buf1)?;
            buf.advance(len);
            Ok(Self::decode(&buf1[..buf1.len()])?)
        } else {
            let msg = Self::decode(&buf[..len])?;
            buf.advance(len);
            Ok(msg)
        }
    }
}

#[allow(unused)]
fn decode_header(header: usize) -> (usize, bool) {
    let len = header & !COMPRESSION_BIT;
    let compressed = header & COMPRESSION_BIT == COMPRESSION_BIT;
    (len, compressed)
}

impl FrameCoder for CommandRequest {}
impl FrameCoder for CommandResponse {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Value, value};
    use bytes::Bytes;

    #[test]
    fn command_request_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let cmd = CommandRequest::new_hget("t1", "k1");
        cmd.encode_frame(&mut buf).unwrap();

        // 最高位没设置
        assert_eq!(is_compressed(&buf), false);

        let cmd1 = CommandRequest::decode_frame(&mut buf).unwrap();
        assert_eq!(cmd, cmd1);
    }

    #[test]
    fn command_response_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let values: Vec<Value> = vec![1.into(), "hello".into(), b"data".into()];
        let res: CommandResponse = values.into();
        res.encode_frame(&mut buf).unwrap();

        // 最高位没设置
        assert_eq!(is_compressed(&buf), false);

        let res1 = CommandResponse::decode_frame(&mut buf).unwrap();
        assert_eq!(res, res1);
    }

    #[test]
    fn command_response_compressed_encode_decode_should_work() {
        let mut buf = BytesMut::new();

        let value: Value = Bytes::from(vec![0u8; COMPRESSION_LIMIT + 1]).into();
        let res: CommandResponse = value.into();
        res.encode_frame(&mut buf).unwrap();

        // 最高位设置了
        assert_eq!(is_compressed(&buf), true);

        let res1 = CommandResponse::decode_frame(&mut buf).unwrap();
        assert_eq!(res, res1);
    }

    fn is_compressed(data: &[u8]) -> bool {
        if let &[v] = &data[..1] {
            v >> 7 == 1
        } else {
            false
        }
    }
    impl<const N: usize> From<&[u8; N]> for Value {
        fn from(value: &[u8; N]) -> Self {
            Bytes::copy_from_slice(&value[..]).into()
        }
    }

    impl From<Bytes> for Value {
        fn from(buf: Bytes) -> Self {
            Self {
                value: Some(value::Value::BytesValue(buf.to_vec())),
            }
        }
    }
}
