use core::time::Duration;
use rodio::source::{TrackPosition, UniformSourceIterator};
use rodio::{Decoder, Source};
use std::path::PathBuf;
use std::{fs::File, io::BufReader};

use rodio::conversions::SampleTypeConverter;

use anyhow::{Ok, Result};

pub struct Slot<I>
where
    I: Source,
{
    cassette: Option<I>,
    pub duration: Duration,
    pub playback_position: Duration,
}

impl Slot<TrackPosition<UniformSourceIterator<Decoder<BufReader<File>>>>> {
    pub fn new() -> Self {
        Slot {
            cassette: None,
            duration: Duration::from_nanos(0),
            playback_position: Duration::from_nanos(0),
        }
    }

    pub fn insert(path: PathBuf) -> Self {
        let file = File::open(path).unwrap();
        let decoder = Decoder::try_from(file).unwrap();
        let formatter = UniformSourceIterator::new(decoder, 2, 48_000).track_position();
        let dur = formatter.total_duration().unwrap();

        Slot {
            cassette: Some(formatter),
            duration: dur,
            playback_position: Duration::new(0, 0),
        }
    }

    pub fn play(&mut self, output_buffer: &mut [i16]) -> Result<usize> {
        match self.cassette.as_mut() {
            None => anyhow::bail!("No cassette inserted"),
            Some(cassette) => {
                let demand = cassette.take(output_buffer.len());
                let conv = SampleTypeConverter::new(demand);
                let mut count = 0;
                for (item, val) in output_buffer.iter_mut().zip(conv) {
                    *item = val;
                    count += 1;
                }
                let pb = cassette.get_pos();
                self.playback_position = pb;
                return Ok(count)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn insert_cassette() {
        let path_str = "";
        let path = PathBuf::from_str(path_str).unwrap();
        let slot = Slot::insert(path);
        println!("{:?}", slot.duration);
    }

    fn write_i16_to_file(path_str: &str, data: &[i16]) -> Result<()> {
        let path = PathBuf::from_str(path_str).unwrap();
        // 1. ファイルを開く（存在しない場合は作成）
        let mut file = OpenOptions::new().append(true).create(true).open(path)?;

        // 2. データをバイト列に変換して書き込む
        for &sample in data {
            // 各i16をバイト列に変換
            let bytes = sample.to_le_bytes();
            // ファイルに書き込む
            file.write_all(&bytes)?;
        }

        // 成功した場合はOk(())を返す
        Ok(())
    }

    #[test]
    fn play_n_samples() {
        let mut buf = vec![0; 32 * 1024];

        let path_str = "";
        let path = PathBuf::from_str(path_str).unwrap();
        let mut slot = Slot::insert(path);

        for i in 0..3 {
            let n_read = slot.play(&mut buf).unwrap();
            println!("{:?}", n_read);
            println!("{:?}", slot.playback_position);
            // let _ = write_i16_to_file("", &buf);
        }
    }
}
