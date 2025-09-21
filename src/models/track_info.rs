use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

// JSONのキーと一致するように構造体を定義
#[derive(Deserialize, Debug)]
pub struct TrackInfo {
    pub title: String,
    pub duration: Duration,
    pub artist: String,
}

impl TrackInfo {
    pub fn new(id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut file_path = PathBuf::from_str(id).unwrap();
        file_path.set_extension("info.json");
        let contents = fs::read_to_string(file_path)?;

        // JSONをHashMapにデシリアライズ
        let map: HashMap<String, Value> = serde_json::from_str(&contents)?;

        // HashMapから値を取り出し、TrackInfoを構築
        // .get()はOptionを返すため、unwrap_or_elseや問号演算子でエラーハンドリングが必要
        let title_val = map.get("title").ok_or("Missing 'title' key")?;
        let artist_val = map.get("channel").ok_or("Missing 'artist' key")?;
        let duration_val = map.get("duration").ok_or("Missing 'duration' key")?;

        // Value型からStringやu64に変換
        let title = title_val
            .as_str()
            .ok_or("Invalid 'title' value")?
            .to_string();
        let artist = artist_val
            .as_str()
            .ok_or("Invalid 'artist' value")?
            .to_string();
        let duration_secs = duration_val.as_u64().ok_or("Invalid 'duration' value")?;

        Ok(TrackInfo {
            title,
            duration: Duration::from_secs(duration_secs),
            artist,
        })
    }
}

#[cfg(test)]
mod test {
    use super::TrackInfo;

    #[test]
    fn read_info() {
        let id = "/root/austr/dev/rust/bbb-queue/private/media/ youtube.hOdQKfQsKpM.m4a";
        let info = TrackInfo::new(id).unwrap();
        println!("info: {:?}", info);
    }
}
