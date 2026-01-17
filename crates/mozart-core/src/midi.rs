//! MIDI export functionality
//!
//! Exports songs to Standard MIDI File (SMF) Format 0

use crate::error::{MozartError, Result};
use crate::song::Song;
use crate::TICKS_PER_QUARTER;
use std::path::Path;

/// MIDI file writer
pub struct MidiExporter {
    /// Ticks per quarter note in the output file
    pub ticks_per_quarter: u16,
}

impl Default for MidiExporter {
    fn default() -> Self {
        MidiExporter {
            ticks_per_quarter: TICKS_PER_QUARTER as u16,
        }
    }
}

impl MidiExporter {
    /// Create a new MIDI exporter
    pub fn new() -> Self {
        Self::default()
    }

    /// Export a song to MIDI bytes
    pub fn export(&self, song: &Song) -> Result<Vec<u8>> {
        tracing::info!("Exporting song '{}' to MIDI", song.metadata.title);

        let mut data = Vec::new();

        // Write MIDI header
        self.write_header(&mut data)?;

        // Write single track
        let track_data = self.build_track(song)?;
        self.write_track(&mut data, &track_data)?;

        tracing::info!("MIDI export complete: {} bytes", data.len());
        Ok(data)
    }

    /// Export a song to a file
    pub fn export_to_file(&self, song: &Song, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        tracing::info!("Exporting song to {:?}", path);

        let data = self.export(song)?;
        std::fs::write(path, &data)
            .map_err(|e| MozartError::MidiError(format!("Failed to write file: {}", e)))?;

        tracing::info!("MIDI file saved: {:?}", path);
        Ok(())
    }

    fn write_header(&self, data: &mut Vec<u8>) -> Result<()> {
        // MThd chunk
        data.extend_from_slice(b"MThd");
        // Header length (always 6)
        data.extend_from_slice(&6u32.to_be_bytes());
        // Format 0 (single track)
        data.extend_from_slice(&0u16.to_be_bytes());
        // Number of tracks (1)
        data.extend_from_slice(&1u16.to_be_bytes());
        // Ticks per quarter note
        data.extend_from_slice(&self.ticks_per_quarter.to_be_bytes());

        Ok(())
    }

    fn build_track(&self, song: &Song) -> Result<Vec<u8>> {
        let mut track = Vec::new();

        // Tempo meta event (at time 0)
        let tempo_us = 60_000_000 / song.settings.tempo as u32;
        self.write_var_length(&mut track, 0); // Delta time
        track.push(0xFF); // Meta event
        track.push(0x51); // Tempo
        track.push(0x03); // Length
        track.push((tempo_us >> 16) as u8);
        track.push((tempo_us >> 8) as u8);
        track.push(tempo_us as u8);

        // Time signature meta event
        let ts = &song.settings.time_signature;
        self.write_var_length(&mut track, 0); // Delta time
        track.push(0xFF); // Meta event
        track.push(0x58); // Time signature
        track.push(0x04); // Length
        track.push(ts.numerator);
        // Denominator as power of 2 (4 = 2^2, 8 = 2^3)
        let denom_power = match ts.denominator {
            2 => 1,
            4 => 2,
            8 => 3,
            16 => 4,
            _ => 2,
        };
        track.push(denom_power);
        track.push(24); // MIDI clocks per metronome click
        track.push(8);  // 32nd notes per quarter note

        // Key signature meta event
        self.write_var_length(&mut track, 0);
        track.push(0xFF);
        track.push(0x59); // Key signature
        track.push(0x02); // Length
        // Calculate sharps/flats from key
        let key_byte = self.key_to_midi_key(&song.settings.key);
        track.push(key_byte as u8);
        // Major/minor (0 = major, 1 = minor)
        let mode = match song.settings.key.scale_type {
            crate::scale::ScaleType::Major => 0,
            _ => 1, // Treat modes as minor-ish for MIDI purposes
        };
        track.push(mode);

        // Track name meta event
        let title = song.metadata.title.as_bytes();
        self.write_var_length(&mut track, 0);
        track.push(0xFF);
        track.push(0x03); // Track name
        self.write_var_length(&mut track, title.len() as u32);
        track.extend_from_slice(title);

        // Build note events sorted by time
        let mut events: Vec<NoteEvent> = Vec::new();

        for note in &song.notes {
            events.push(NoteEvent {
                tick: note.start_tick,
                is_on: true,
                pitch: note.pitch,
                velocity: note.velocity,
            });
            events.push(NoteEvent {
                tick: note.end_tick(),
                is_on: false,
                pitch: note.pitch,
                velocity: 0,
            });
        }

        // Sort by tick, with note-offs before note-ons at same time
        events.sort_by(|a, b| {
            a.tick.cmp(&b.tick).then_with(|| a.is_on.cmp(&b.is_on))
        });

        // Write note events with delta times
        let mut last_tick = 0u32;
        for event in events {
            let delta = event.tick.saturating_sub(last_tick);
            self.write_var_length(&mut track, delta);

            if event.is_on {
                track.push(0x90); // Note on, channel 0
                track.push(event.pitch);
                track.push(event.velocity);
            } else {
                track.push(0x80); // Note off, channel 0
                track.push(event.pitch);
                track.push(0);
            }

            last_tick = event.tick;
        }

        // End of track meta event
        self.write_var_length(&mut track, 0);
        track.push(0xFF);
        track.push(0x2F);
        track.push(0x00);

        tracing::debug!("Built track with {} bytes", track.len());
        Ok(track)
    }

    fn write_track(&self, data: &mut Vec<u8>, track_data: &[u8]) -> Result<()> {
        data.extend_from_slice(b"MTrk");
        data.extend_from_slice(&(track_data.len() as u32).to_be_bytes());
        data.extend_from_slice(track_data);
        Ok(())
    }

    fn write_var_length(&self, data: &mut Vec<u8>, mut value: u32) {
        if value == 0 {
            data.push(0);
            return;
        }

        let mut buffer = [0u8; 4];
        let mut len = 0;

        while value > 0 {
            buffer[len] = (value & 0x7F) as u8;
            value >>= 7;
            len += 1;
        }

        // Write in reverse order, with continuation bits
        for i in (0..len).rev() {
            let mut byte = buffer[i];
            if i > 0 {
                byte |= 0x80; // Set continuation bit
            }
            data.push(byte);
        }
    }

    fn key_to_midi_key(&self, scale: &crate::scale::Scale) -> i8 {
        use crate::pitch::PitchClass;

        // MIDI key signature: -7 to +7 (flats to sharps)
        // Circle of fifths position
        match scale.root {
            pc if pc == PitchClass::C => 0,
            pc if pc == PitchClass::G => 1,
            pc if pc == PitchClass::D => 2,
            pc if pc == PitchClass::A => 3,
            pc if pc == PitchClass::E => 4,
            pc if pc == PitchClass::B => 5,
            pc if pc == PitchClass::F_SHARP => 6,
            pc if pc == PitchClass::C_SHARP => 7,
            pc if pc == PitchClass::F => -1,
            pc if pc == PitchClass::B_FLAT => -2,
            pc if pc == PitchClass::E_FLAT => -3,
            pc if pc == PitchClass::A_FLAT => -4,
            pc if pc == PitchClass::D_FLAT => -5,
            pc if pc == PitchClass::G_FLAT => -6,
            _ => 0,
        }
    }
}

#[derive(Debug)]
struct NoteEvent {
    tick: u32,
    is_on: bool,
    pitch: u8,
    velocity: u8,
}

/// Quick helper to export a song to MIDI
pub fn export_to_midi(song: &Song) -> Result<Vec<u8>> {
    MidiExporter::new().export(song)
}

/// Quick helper to export a song to a MIDI file
pub fn export_to_midi_file(song: &Song, path: impl AsRef<Path>) -> Result<()> {
    MidiExporter::new().export_to_file(song, path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note::Note;

    #[test]
    fn test_midi_export_basic() {
        let mut song = Song::with_title("MIDI Test");
        song.add_note(Note::new(60, 0, 480));
        song.add_note(Note::new(64, 480, 480));

        let midi = export_to_midi(&song).unwrap();

        // Check header
        assert_eq!(&midi[0..4], b"MThd");
        assert_eq!(&midi[8..10], &[0, 0]); // Format 0
        assert_eq!(&midi[10..12], &[0, 1]); // 1 track

        // Check track header exists
        let track_start = 14;
        assert_eq!(&midi[track_start..track_start + 4], b"MTrk");
    }

    #[test]
    fn test_var_length_encoding() {
        let exporter = MidiExporter::new();

        let mut data = Vec::new();
        exporter.write_var_length(&mut data, 0);
        assert_eq!(data, vec![0]);

        data.clear();
        exporter.write_var_length(&mut data, 127);
        assert_eq!(data, vec![127]);

        data.clear();
        exporter.write_var_length(&mut data, 128);
        assert_eq!(data, vec![0x81, 0x00]);

        data.clear();
        exporter.write_var_length(&mut data, 480); // Our standard quarter note
        // 480 = 0x1E0 = 0b111100000 -> 0x83 0x60
        assert_eq!(data, vec![0x83, 0x60]);
    }

    #[test]
    fn test_midi_export_multiple_notes() {
        let mut song = Song::new();
        // Add C major scale
        for (i, pitch) in [60, 62, 64, 65, 67, 69, 71, 72].iter().enumerate() {
            song.add_note(Note::new(*pitch, i as u32 * 480, 480));
        }

        let midi = export_to_midi(&song).unwrap();

        // Should have header + track
        assert!(midi.len() > 100);
        assert_eq!(&midi[0..4], b"MThd");
    }
}
