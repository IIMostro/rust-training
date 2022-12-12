use anyhow::Result;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use snow::{HandshakeState, TransportState};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder, Framed, LengthDelimitedCodec};

pub const NOISE_CODEC: &str = "Noise_XX_25519_ChaChaPoly_SHA256";
pub const HEADER_LEN: usize = 2;
pub const MAX_FRAME_LEN: usize = 65535;

pub struct Builder {
    params: &'static str,
    initiator: bool,
}

impl Builder {
    pub fn new(params: &'static str, initiator: bool) -> Self {
        Builder {
            params,
            initiator,
        }
    }

    pub fn new_codec(self) -> Result<NoiseCodec> {
        let builder = snow::Builder::new(self.params.parse()?);
        let keypair = builder.generate_keypair()?;
        let mut builder = builder.local_private_key(&keypair.private);
        let noise = match self.initiator {
            true => builder.build_initiator()?,
            false => builder.build_responder()?
        };
        Ok(NoiseCodec {
            builder: self,
            state: NoiseState::Handshake(noise),
        })
    }

    pub fn new_framed<T>(self, inner: T) -> Result<Framed<T, NoiseCodec>>
        where T: AsyncRead + AsyncWrite
    {
        let codec = self.new_codec()?;
        Ok(Framed::new(inner, codec))
    }
}

enum NoiseState {
    Handshake(HandshakeState),
    Transport(TransportState),
}

impl NoiseState {
    fn write_message(&mut self, message: &[u8], output: &mut [u8]) -> Result<usize> {
        match self {
            NoiseState::Handshake(state) => {
                let (len, _) = state.write_message(message, output)?;
                Ok(len)
            }
            NoiseState::Transport(state) => {
                let (len, _) = state.write_message(message, output)?;
                Ok(len)
            }
        }
    }

    fn read_message(&mut self, message: &[u8], output: &mut [u8]) -> Result<usize> {
        match self {
            NoiseState::Handshake(state) => {
                let (len, _) = state.read_message(message, output)?;
                Ok(len)
            }
            NoiseState::Transport(state) => {
                let (len, _) = state.read_message(message, output)?;
                Ok(len)
            }
        }
    }
}

pub struct NoiseCodec {
    builder: Builder,
    state: NoiseState,
}

impl Encoder<Bytes> for NoiseCodec{
    type Error = anyhow::Error;

    fn encode(&mut self, item: Bytes, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        if &item.len() > &MAX_FRAME_LEN {
            return Err(anyhow::anyhow!("frame too large"));
        }
        dst.reserve(HEADER_LEN + &item.len() * 2);
        let mut body = dst.split_off(2);
        let n = self.state.write_message(&item, &mut body)?;
        dst.put_uint(n as u64, HEADER_LEN);
        dst.unsplit(body);
        Ok(())
    }
}

impl Decoder for NoiseCodec{

    type Item = BytesMut;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> std::result::Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_LEN {
            return Ok(None);
        }
        let len = src.get_uint(HEADER_LEN) as usize;
        if src.len() < (HEADER_LEN + &len) {
            return Ok(None);
        }
        let mut payload = src.split_to(len);
        let n = self.state.read_message(&payload, src)?;
        let decode = src.split_to(n);
        Ok(Some(decode))
    }
}

