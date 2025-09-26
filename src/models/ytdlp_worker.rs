use regex::Regex;
use std::io::{BufRead, BufReader, Lines};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

/// yt-dlpの出力をリアルタイムで提供するWorker構造体
pub struct YtdlpWorker {
    child: Child,
    lines: Lines<BufReader<std::process::ChildStdout>>,
    parse_re: Regex,
    pub dst: Option<PathBuf>,
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

        let re = Regex::new(
            r#"(?x)
            \[.*?\] \s+ Destination: \s+
            (?P<destination>.*)
        "#,
        )
        .unwrap();

        Ok(Self {
            child,
            lines,
            parse_re: re,
            dst: None,
        })
    }

    /// yt-dlpプロセスの終了コードを取得する
    /// プロセスがまだ実行中の場合はNoneを返す
    pub fn status(&mut self) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        self.child.try_wait()
    }
}

/// YtdlpWorkerをイテレーターとして実装
/// TODO: 出力のタイプを定義
impl Iterator for YtdlpWorker {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next_line) = self.lines.next() {
            if let Ok(line) = next_line {
                // 正規表現でマッチを試みる
                if let Some(captures) = self.parse_re.captures(&line) {
                    // 名前付きキャプチャで "destination" の値を取得
                    if let Some(dest) = captures.name("destination") {
                        // ここに、"destination"がマッチした場合の追加の処理を記述
                        println!("Matched destination: {}", dest.as_str());
                        // 例: ログ記録、別のデータ構造への保存など
                        self.dst = Some(PathBuf::from(dest.as_str()));
                    }
                }
                return Some(Ok(line));
            }

            // 元の行を返す
            return Some(next_line);
        }

        // イテレータが終了した場合はNoneを返す
        None
    }
}

// イテレータが完了したときに特定のFnOnceクロージャを実行するアダプタ
pub struct OnCompletion<I, F>
where
    I: Iterator,
    F: FnOnce(&I),
{
    iter: I,
    on_completion: Option<F>,
}

impl<I, F> Iterator for OnCompletion<I, F>
where
    I: Iterator,
    F: FnOnce(&I),
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(value) => Some(value),
            None => {
                if let Some(on_completion) = self.on_completion.take() {
                    on_completion(&self.iter);
                }
                None
            }
        }
    }
}

// 既存のイテレータに`on_completion`メソッドを追加するためのトレイト
pub trait IteratorExt: Iterator {
    fn on_completion<F>(self, on_completion: F) -> OnCompletion<Self, F>
    where
        Self: Sized,
        F: FnOnce(&Self),
    {
        OnCompletion {
            iter: self,
            on_completion: Some(on_completion),
        }
    }
}

// Iteratorトレイトを実装する全ての型にIteratorExtトレイトを実装
impl<T: Iterator> IteratorExt for T {}

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
