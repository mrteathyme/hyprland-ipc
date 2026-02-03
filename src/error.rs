use std::io::Error as IoError;
use tokio_util::codec::LinesCodecError;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Hyprland returned not-ok response: {0}")]
    NotOkResponse(String),

    #[error("Failed to parse Hyprland IPC response")]
    MalformedInput,

    #[error("HYPRLAND_INSTANCE_SIGNATURE was not found. Are you sure Hyprland is running?")]
    NoInstanceSignature,

    #[error("Unknown event {0}: {1}")]
    UnknownEvent(String, String),

    #[error("LinesCodec error: {0}")]
    LinesCodec(#[from] LinesCodecError),

    #[error("IO error: {0}")]
    Io(#[from] IoError),
}


pub type Result<T, E = Error> = std::result::Result<T, E>;
