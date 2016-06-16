use std::result;
use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidMosqClient,
    ConnectionError(i32),
    AlreadyConnected,
    UnsupportedFeature,
    UnrecognizedPacket,
    ConnectionAbort,
    HandshakeFailed,
    ProtocolViolation,
    Disconnected,
    Timeout,
    ConnectionRefused(i32),
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}
