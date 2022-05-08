use std::{fs, io};
use std::fs::File;
use std::path::Path;
use std::io::Read;

/// Stores the contents of a file in a String and in a byte vector.

pub struct TorrentFile {
    pub string_contents: String,
    pub byte_contents: Vec<u8>,
}

impl TorrentFile {
    /// Reads the contents of a file in a specific path, saves it as a UTF-8 enconded String
    /// and its contents as a byte vector,
    /// and returns a TorrentFile with those contents.
    ///
    /// # Examples
    ///
    /// Basic Usage:
    ///
    /// ```
    /// let file_path = Path::new("path/to/file.txt");
    /// TorrentFile::load_file_from_path(file_path);
    /// ```
    pub fn load_file_from_path(path: &Path) -> Result<TorrentFile, io::Error> {
        let string_read = fs::read_to_string(path)?;
        let mut f:File = File::open(&path)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(TorrentFile{
            string_contents: string_read,
            byte_contents: buffer,
        })
    }
}


#[cfg(test)]
mod tests{    
    use super::*;

    #[test]
    fn loads_an_existing_file_correctly() {

        let string_expected :String = "Hola".to_string();
        let bytes_expected :Vec<u8> = "Hola".to_string().into_bytes();
        let file1_path = Path::new("tests/hola.txt"); 
        match TorrentFile::load_file_from_path(file1_path) {
            Ok(file)  => {
                assert_eq!(string_expected, file.string_contents);
                assert_eq!(bytes_expected, file.byte_contents);

            },
            Err(e) => assert!(false,"{}", e),
        };
        
    }
}