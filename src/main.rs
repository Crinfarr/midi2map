use std::io::{Read, Seek};
mod midi_parser;
use crate::midi_parser::{MidiType, ParseFromStreamBE};

#[derive(thiserror::Error, Debug)]
enum AppErr {
    #[error("Argument error: {}", .0)]
    ArgumentErr(String),
    #[error("Filesystem error: {}", .0)]
    FileSystemError(String),
    #[error("Input file error: {}", .0)]
    FileError(String),
    #[error("File parsing error: {}", .0)]
    ParseError(String),
}
type AppResult = Result<(), AppErr>;

fn main() -> AppResult {
    let fp_str = std::env::args()
        .nth(0)
        .ok_or(AppErr::ArgumentErr("No file path".to_owned()))?;
    let mut istream = std::fs::File::open(fp_str)
        .map_err(|e| AppErr::FileSystemError(format!("Error opening input: {e}")))?;
    //header chunk
    //verify header
    {
        let mut f4b = [0u8; 4];
        istream
            .read_exact(&mut f4b)
            .map_err(|e| AppErr::FileError(format!("Failed to read header: {e}")))?;
        //magic number
        if f4b != "MThd".as_bytes() {
            return Err(AppErr::FileError(format!("No midi header")));
        }
    }
    //Verify midi type
    let miditype = {
        //skip header length field since it's always 0x00000006 or this isn't a midi file
        istream
            .seek_relative(4)
            .map_err(|e| AppErr::FileError(format!("While seeking to header: {e}")))?;
        let midi_type = MidiType::from_istream(&mut istream)
            .map_err(|e| AppErr::ParseError(format!("While reading MIDI format: {e}")))?;
        match midi_type {
            MidiType::Clip(_) => println!("Detected single track file format"),
            MidiType::MultiTrack(ntracks) => {
                println!("Detected {ntracks} parallel tracks, defaulting to track 1")
            }
            MidiType::SequentialTrack(n) => println!("Detected {n} sequential tracks"),
        }
        midi_type
    };

    Ok(())
}
