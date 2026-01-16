use std::{
    fmt::Display,
    fs::File,
    io::{Read, Seek},
};

pub trait ParseFromStreamBE<T>
where
    T: Read + Seek,
{
    type Error;
    fn from_istream(f: &mut T) -> Result<Self, Self::Error>
    where
        Self: Sized,
        Self::Error: Display;
}

pub struct MidiParser {}
pub enum MidiType {
    ///One single track in one single chunk
    Clip(u16),
    ///Chunks represent parallel channels
    MultiTrack(u16),
    ///Tracks represent sequential chunks of one channel
    SequentialTrack(u16),
}

impl ParseFromStreamBE<File> for u16 {
    type Error = std::io::Error;
    fn from_istream(f: &mut File) -> Result<Self, Self::Error> {
        let mut ia = [0u8; 2];
        f.read(&mut ia)?;
        Ok(u16::from_be_bytes(ia))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseMidiTypeErr {
    #[error("IoError: {}", .0)]
    IoErr(#[from] std::io::Error),
    #[error("OutOfBoundsErr: expected range from {} to {} but got {}", .0, .1, .2)]
    OutOfBoundsErr(u16, u16, String),
    #[error("MidiFormatErr: {}", .0)]
    MidiFormatErr(String),
}
impl ParseFromStreamBE<File> for MidiType {
    type Error = ParseMidiTypeErr;
    fn from_istream(f: &mut File) -> Result<Self, ParseMidiTypeErr> {
        match u16::from_istream(f)? {
            0 => {
                let chunk_ct = u16::from_istream(f)?;
                if chunk_ct != 0x0001 {
                    Err(ParseMidiTypeErr::MidiFormatErr(format!(
                        "File claimed type 0 but contains more than one track chunk"
                    )))
                } else {
                    Ok(Self::Clip(0x0001))
                }
            }
            1 => Ok(Self::MultiTrack(u16::from_istream(f)?)),
            2 => Ok(Self::MultiTrack(u16::from_istream(f)?)),
            v => Err(ParseMidiTypeErr::OutOfBoundsErr(0, 2, v.to_string())),
        }
    }
}
