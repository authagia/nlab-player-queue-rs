use std::io::{BufRead, BufReader, Lines};
use std::process::{Child, Command, Stdio};

/// yt-dlpの出力をリアルタイムで提供するWorker構造体
pub struct YtdlpWorker {
    child: Child,
    lines: Lines<BufReader<std::process::ChildStdout>>,
}

impl YtdlpWorker {
    /// 新しいYtdlpWorkerインスタンスを作成する
    ///
    /// # Arguments
    ///
    /// * `url` - ダウンロードする動画のURL
    /// * `options` - yt-dlpに渡すオプション引数のリスト
    pub fn new(url: &str, options: &[&str]) -> Result<Self, std::io::Error> {
        let mut command = Command::new("yt-dlp");
        command.arg(url);
        command.args(options);
        command.stdout(Stdio::piped());

        let mut child = command.spawn()?;

        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture stdout")
        })?;

        let reader = BufReader::new(stdout);
        let lines = reader.lines();

        Ok(Self { child, lines })
    }

    /// yt-dlpプロセスの終了コードを取得する
    /// プロセスがまだ実行中の場合はNoneを返す
    pub fn status(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }
}

/// YtdlpWorkerをイテレーターとして実装
impl Iterator for YtdlpWorker {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}

#[cfg(test)]
mod test {
    use super::YtdlpWorker;

    #[test]
    fn download_audio() {
        let url = "";
        let wk = YtdlpWorker::new(
            url,
            &[
                "--newline",
                "--no-color",
                "--write-info-json",
                "--no-playlist",
                "-f bestaudio",
                "-o %(extractor)s.%(id)s.%(ext)s",
                "-P ~//media/",
                "-x",
                "--audio-format=m4a",
                "--audio-quality=144k",
            ],
        )
        .unwrap();
        for line in wk {
            print!("\r{:?}", line.unwrap());
        }
    }
}

/*
   newline: true,
      color: false,
      write_info_json: true,
      playlist: false,
    }

{
      extract_audio: true,
      format: "bestaudio",
      output: "%(extractor)s.%(id)s.%(ext)s",
      paths: "dl/",
     } */
