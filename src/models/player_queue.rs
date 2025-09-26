use anyhow::{Ok, Result, anyhow};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::result::Result::Ok as stdOk;

pub struct PlayerQueue {
    file_path: PathBuf,
}

impl PlayerQueue {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        // Ensure the file exists, creating it if necessary
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)?;

        Ok(Self { file_path })
    }

    pub fn size(&self) -> Result<usize> {
        Ok(self.get_entries()?.len())
    }

    pub fn list_all(&self) -> Result<Vec<String>> {
        self.get_entries()
    }

    pub fn get_at(&self, index: usize) -> Result<String> {
        match self.get_rel(index, 0) {
            stdOk((_, id)) => Ok(id),
            Err(e) => Err(e),
        }
    }

    pub fn get_next(&self, index: usize) -> Result<(usize, String)> {
        self.get_rel(index, 1)
    }

    pub fn get_prev(&self, index: usize) -> Result<(usize, String)> {
        self.get_rel(index, -1)
    }

    pub fn get_rel(&self, line_number: usize, step: isize) -> Result<(usize, String)> {
        let entries = self.get_entries()?;
        let len = entries.len();
        let len_isize = len as isize;

        // 行番号が有効な範囲内にあるか確認
        if line_number >= len {
            return Err(anyhow!("Invalid line number: {}", line_number));
        }
        let move_over = len_isize + step % len_isize;
        let new_line_number = ((line_number as isize + move_over) % len_isize) as usize;
        Ok((new_line_number, entries[new_line_number].clone()))
    }

    /// Appends a new ID to the end of the playlist file.
    pub fn append(&mut self, id: &str) -> Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.file_path)?;
        writeln!(file, "{}", id)?;
        Ok(())
    }

    /// 指定されたIDと行番号に一致する行をプレイリストファイルから削除します。
    pub fn remove_at(&mut self, id: &str, line_number: usize) -> Result<()> {
        let mut entries = self.get_entries()?;

        // 行番号が有効な範囲内にあるか確認
        if line_number >= entries.len() {
            return Err(anyhow!("Invalid line number: {}", line_number));
        }

        // 指定された行のIDが引数のIDと一致するか確認
        if entries[line_number] != id {
            return Err(anyhow!(
                "ID at line {} does not match the provided ID '{}'",
                line_number,
                id
            ));
        }
        // 指定された行を削除
        entries.remove(line_number);

        self.write_entries(&entries)
    }

    /// Moves a song from a specific line number to a position immediately after another specified song.
    ///
    /// The function requires the IDs and their corresponding line numbers to handle duplicates correctly.
    pub fn move_next_to_at(
        &mut self,
        src_id: &str,
        src_line: usize,
        dst_id: &str,
        dst_line: usize,
    ) -> Result<()> {
        let mut entries = self.get_entries()?;
        let num_entries = entries.len();

        // Validate line numbers and IDs
        if src_line >= num_entries || entries[src_line] != src_id {
            return Err(anyhow!(
                "Source ID '{}' not found at line {}",
                src_id,
                src_line
            ));
        }
        if dst_line >= num_entries || entries[dst_line] != dst_id {
            return Err(anyhow!(
                "Destination ID '{}' not found at line {}",
                dst_id,
                dst_line
            ));
        }

        // Perform the move
        let moved_entry = entries.remove(src_line);

        // Adjust destination index if the source was removed before it
        let new_dst_idx = if src_line < dst_line {
            dst_line - 1
        } else {
            dst_line
        };
        entries.insert(new_dst_idx + 1, moved_entry);

        self.write_entries(&entries)
    }

    fn get_entries(&self) -> Result<Vec<String>> {
        let file = File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for line in reader.lines() {
            entries.push(line?);
        }
        Ok(entries)
    }

    fn write_entries(&self, entries: &[String]) -> Result<()> {
        // Use a temporary file for safe atomic writes
        let temp_path = self.file_path.with_extension("tmp");
        let mut temp_file = BufWriter::new(File::create(&temp_path)?);

        for entry in entries {
            writeln!(temp_file, "{}", entry)?;
        }

        temp_file.flush()?;
        fs::rename(&temp_path, &self.file_path)?;
        Ok(())
    }
}
