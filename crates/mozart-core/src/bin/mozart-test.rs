//! Mozart Test CLI
//!
//! A command-line tool for testing the Mozart music engine

use mozart_core::*;
use std::io::{self, BufRead, Write};

fn main() {
    // Initialize logging
    init_logging();
    println!("Mozart Test CLI v0.1.0");
    println!("Type 'help' for available commands\n");

    let mut song = Song::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("mozart> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let cmd = parts[0];
        let args = parts.get(1).copied().unwrap_or("");

        match cmd {
            "help" => print_help(),
            "quit" | "exit" | "q" => break,

            "new" => {
                song = Song::new();
                println!("Created new song");
            }

            "info" => {
                println!("Title: {}", song.metadata.title);
                println!("Tempo: {} BPM", song.settings.tempo);
                println!("Time Signature: {}", song.settings.time_signature);
                println!("Key: {}", song.settings.key);
                println!("Notes: {}", song.notes.len());
                println!("Duration: {:.2}s", song.duration_seconds());
                println!("Measures: {}", song.measure_count());
            }

            "title" => {
                if args.is_empty() {
                    println!("Current title: {}", song.metadata.title);
                } else {
                    song.metadata.title = args.to_string();
                    println!("Title set to: {}", args);
                }
            }

            "tempo" => {
                if args.is_empty() {
                    println!("Current tempo: {} BPM", song.settings.tempo);
                } else if let Ok(tempo) = args.parse::<u16>() {
                    song.set_tempo(tempo);
                    println!("Tempo set to {} BPM", song.settings.tempo);
                } else {
                    println!("Invalid tempo: {}", args);
                }
            }

            "time" => {
                if args.is_empty() {
                    println!("Current time signature: {}", song.settings.time_signature);
                    println!("Accents: {}", song.settings.time_signature.accents);
                } else {
                    match time::TimeSignature::parse(args) {
                        Ok(ts) => {
                            song.set_time_signature(ts);
                            println!("Time signature set to {}", song.settings.time_signature);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "key" => {
                if args.is_empty() {
                    println!("Current key: {}", song.settings.key);
                    println!("Scale notes: {:?}", song.settings.key.pitch_classes());
                } else {
                    match scale::Scale::parse(args) {
                        Ok(scale) => {
                            song.set_key(scale);
                            println!("Key set to {}", song.settings.key);
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "melody" => {
                if args.is_empty() {
                    println!("Current melody: {}", note::format_melody(&song.notes));
                } else {
                    match note::parse_melody(args) {
                        Ok(notes) => {
                            song.clear_notes();
                            song.add_notes(notes);
                            println!("Melody set: {} notes", song.notes.len());
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "notes" => {
                if song.notes.is_empty() {
                    println!("No notes");
                } else {
                    for (i, note) in song.notes.iter().enumerate() {
                        println!(
                            "  [{}] {} at tick {} (duration {})",
                            i, note, note.start_tick, note.duration_ticks
                        );
                    }
                }
            }

            "clear" => {
                song.clear_notes();
                println!("Notes cleared");
            }

            "transpose" => {
                if args.is_empty() {
                    println!("Usage:");
                    println!("  transpose chromatic <semitones>   (e.g., transpose chromatic 2)");
                    println!("  transpose diatonic <degrees>      (e.g., transpose diatonic 2)");
                } else {
                    let parts: Vec<&str> = args.split_whitespace().collect();
                    if parts.len() < 2 {
                        println!("Missing argument");
                        continue;
                    }

                    let mode = match parts[0] {
                        "chromatic" | "c" => {
                            if let Ok(semitones) = parts[1].parse::<i8>() {
                                transpose::TransposeMode::chromatic(semitones)
                            } else {
                                println!("Invalid semitones: {}", parts[1]);
                                continue;
                            }
                        }
                        "diatonic" | "d" => {
                            if let Ok(degrees) = parts[1].parse::<i8>() {
                                transpose::TransposeMode::diatonic(song.settings.key, degrees)
                            } else {
                                println!("Invalid degrees: {}", parts[1]);
                                continue;
                            }
                        }
                        _ => {
                            println!("Unknown transpose mode: {}", parts[0]);
                            continue;
                        }
                    };

                    println!("Transposing: {}", mode.description());

                    match transpose::transpose_notes(&song.notes, &mode) {
                        Ok(transposed) => {
                            song.notes = transposed;
                            println!("Transposed {} notes", song.notes.len());
                            println!("New melody: {}", note::format_melody(&song.notes));
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "detect" => {
                if song.notes.is_empty() {
                    println!("No notes to analyze");
                } else {
                    match transpose::detect_scale(&song.notes) {
                        Some(scale) => println!("Detected scale: {}", scale),
                        None => println!("Could not detect scale"),
                    }
                }
            }

            "save" => {
                if args.is_empty() {
                    println!("Usage: save <filename>");
                } else {
                    let path = if args.ends_with(".mozart.json") {
                        args.to_string()
                    } else {
                        format!("{}.mozart.json", args)
                    };
                    match song.save(&path) {
                        Ok(()) => println!("Saved to {}", path),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "load" => {
                if args.is_empty() {
                    println!("Usage: load <filename>");
                } else {
                    match Song::load(args) {
                        Ok(loaded) => {
                            song = loaded;
                            println!("Loaded: {} ({} notes)", song.metadata.title, song.notes.len());
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "midi" => {
                if args.is_empty() {
                    println!("Usage: midi <filename>");
                } else {
                    let path = if args.ends_with(".mid") {
                        args.to_string()
                    } else {
                        format!("{}.mid", args)
                    };
                    match midi::export_to_midi_file(&song, &path) {
                        Ok(()) => println!("Exported MIDI to {}", path),
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }

            "json" => {
                match song.to_json() {
                    Ok(json) => println!("{}", json),
                    Err(e) => println!("Error: {}", e),
                }
            }

            "scales" => {
                println!("Available scales:");
                for scale_type in scale::ScaleType::all() {
                    println!("  - {}", scale_type.name());
                }
            }

            "demo" => {
                println!("Loading demo melody...");
                song = Song::with_title("Demo Song");
                song.set_tempo(120);
                song.set_key(scale::Scale::c_major());

                let melody = note::parse_melody("C4q D4q E4q F4q G4h Rq G4q A4q B4q C5w").unwrap();
                song.add_notes(melody);

                println!("Loaded demo song with {} notes", song.notes.len());
                println!("Melody: {}", note::format_melody(&song.notes));
            }

            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", cmd);
            }
        }
    }

    println!("Goodbye!");
}

fn print_help() {
    println!("Available commands:");
    println!();
    println!("  Song Management:");
    println!("    new                       Create a new song");
    println!("    info                      Show song information");
    println!("    title [name]              Get/set song title");
    println!("    demo                      Load a demo melody");
    println!();
    println!("  Settings:");
    println!("    tempo [bpm]               Get/set tempo");
    println!("    time [n/d]                Get/set time signature (e.g., 7/8)");
    println!("    key [root scale]          Get/set key (e.g., 'C major', 'F# dorian')");
    println!();
    println!("  Notes:");
    println!("    melody [notation]         Get/set melody (e.g., 'C4q D4q E4h')");
    println!("    notes                     List all notes");
    println!("    clear                     Clear all notes");
    println!();
    println!("  Transposition:");
    println!("    transpose chromatic <n>   Transpose by n semitones");
    println!("    transpose diatonic <n>    Transpose by n scale degrees");
    println!("    detect                    Detect the scale from notes");
    println!();
    println!("  Files:");
    println!("    save <file>               Save to .mozart.json file");
    println!("    load <file>               Load from file");
    println!("    midi <file>               Export to MIDI file");
    println!("    json                      Print song as JSON");
    println!();
    println!("  Other:");
    println!("    scales                    List available scale types");
    println!("    help                      Show this help");
    println!("    quit                      Exit the program");
}
