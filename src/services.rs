use std::{fs::File, io::BufReader, path::PathBuf};
use std::result::Result::Ok as StdOk;

use anyhow::{Context, Ok, Result};
use rodio::{
    Decoder,
    source::{TrackPosition, UniformSourceIterator},
};

use crate::models::{
    config::{self, Config},
    player_queue::PlayerQueue,
    slot::Slot,
    ytdlp_worker::IteratorExt,
    ytdlp_worker::YtdlpWorker,
};

pub struct AppService {
    playlist: PlayerQueue,
    slot: Slot<TrackPosition<UniformSourceIterator<Decoder<BufReader<File>>>>>,
    index: usize,
    cfg: Config,
}

impl AppService {
    pub fn new(config: Config) -> Self {
        let pl = PlayerQueue::new(config.data_dir.join("main.m3u8")).unwrap();
        let slot = Slot::new();
        AppService {
            playlist: pl,
            slot: slot,
            index: 0,
            cfg: config,
        }
    }

    pub fn add_track(&mut self, url: &str) -> Result<impl Iterator> {
        // 1. Get the Vec<String> from self.cfg.ytdlp_option
        let options_vec: &Vec<String> = &self.cfg.ytdlp_option;

        // 2. Convert the Vec<String> into a Vec<&str> by iterating and borrowing each String
        let options_str_vec: Vec<&str> = options_vec.iter().map(|s| s.as_str()).collect();

        // 3. Get a slice [&str] from the temporary Vec<&str>
        let options_slice: &[&str] = &options_str_vec;

        // 4. Pass the slice to the YtdlpWorker constructor
        Ok(
            YtdlpWorker::new(url, options_slice)?.on_completion(|inner| {
                if let Some(id) = inner
                    .dst
                    .as_ref()
                    .and_then(|path| path.file_name())
                    .and_then(|os_str| os_str.to_str())
                {
                    // This block runs only if `to_str()` returned `Some(&str)`.
                    // `id` is the `&str` value, and it can be used here.
                    let _ = self.playlist.append(id);
                }
            }),
        )
    }

    fn play_slot(&mut self, output_buffer: &mut [u8]) -> Result<usize> {
        let mut binding = vec![0; output_buffer.len() / 2];
        let mut output_buffer_i16: &mut [i16] = binding.as_mut_slice();
        let count = self.slot.play(&mut output_buffer_i16)?;
        for (i, chunk) in output_buffer.chunks_exact_mut(2).enumerate() {
            let sample_bytes = output_buffer_i16[i].to_le_bytes();
            chunk[0] = sample_bytes[0]; // Lower byte
            chunk[1] = sample_bytes[1]; // Upper byte
        }
        Ok(count)
    }

    pub fn play(&mut self, output_buffer: &mut [u8]) -> Result<usize> {
        if let StdOk(count) = self.play_slot(output_buffer) {
            if 2 * count < output_buffer.len() {
                // End of track reached
                self.slot = Slot::new(); // Eject cassette
                if let StdOk((idx, path)) = self.playlist.get_next(self.index) {
                    self.index = idx;
                    let full_path = self.cfg.data_dir.join(path);
                    self.slot = Slot::insert(full_path);
                    // 二回で足りるはず
                    let left = self.play_slot(&mut output_buffer[count * 2..])?;
                    return Ok(left + count);
                } else {
                    return Ok(2 * count); // No more tracks
                }
            } else { 
                return Ok(2 * count); // Still playing current track
            }
        } else {
            // No cassette in slot
            if let StdOk(path) = self.playlist.get_at(self.index) {
                let full_path = self.cfg.data_dir.join(path);
                self.slot = Slot::insert(full_path);
                let count = self.play_slot(output_buffer)?;
                return Ok(count);
            } else {
                return Ok(0); // No cassette and no track to play
            }
        }
    }
}
