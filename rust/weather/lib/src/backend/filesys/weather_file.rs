#![allow(unused)]
use std::path::{Path, PathBuf};
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::ErrorKind,
};

/// The [WeatherFile] error builder.
macro_rules! error {
    ($id:expr, $reason:expr) => {
        crate::Error::from(format!("WeatherFile {}: {}", $id, $reason))
    };
}

/// The manager of a file within the weather directory.
#[derive(Debug)]
pub struct WeatherFile {
    /// The file name within the weather directory.
    pub filename: String,
    /// The file path.
    path: PathBuf,
}

impl std::fmt::Display for WeatherFile {
    /// Use the trait to get the pathname of the file.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl WeatherFile {
    /// Create the manager for files in the weather directory.
    ///
    /// # Arguments
    ///
    /// * `path` is the weather data file returned by the [`WeatherDir`].
    pub fn new(path: PathBuf) -> Self {
        // this should always work since the path comes from a DirEntry
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        WeatherFile { filename, path }
    }

    /// Return the weather file with a new file extension.
    ///
    /// # Argument
    ///
    /// * `extension` is the new file extension.
    ///
    pub fn with_extension(&self, extension: &str) -> Self {
        WeatherFile::new(self.path.with_extension(extension))
    }

    /// Indicates if the file exists or does not.
    pub fn exists(&self) -> bool {
        self.metadata().map_or(false, |_| true)
    }

    /// Get the size of the file.
    pub fn size(&self) -> u64 {
        self.metadata().map_or(0, |metadata| metadata.len())
    }

    /// Get the writer that can be used to update a Zip archive.
    pub fn writer(&self) -> crate::Result<File> {
        match File::options().read(true).write(true).open(&self.path) {
            Ok(file) => Ok(file),
            Err(err) => Err(error!(&self.filename, &format!("open read/write error ({}).", &err))),
        }
    }

    /// Get the reader that can be used to read the contents of a Zip archive.
    pub fn reader(&self) -> crate::Result<File> {
        match OpenOptions::new().read(true).open(&self.path) {
            Ok(file) => Ok(file),
            Err(err) => Err(error!(&self.filename, &format!("open read error ({})...", &err))),
        }
    }

    /// Get the weather file as a [Path].
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// Remove the weather file from the filesystem.
    pub fn remove(&self) -> crate::Result<()> {
        if !self.exists() {
            log::trace!("{}", error!(self.filename, "Does not exist..."));
            Ok(())
        } else if let Err(remove_error) = fs::remove_file(&self.path) {
            let error = error!(&self.filename, format!("Error removing {}: {}", self.path.display(), remove_error));
            log::error!("{}", error);
            Err(error)
        } else {
            Ok(())
        }
    }
    /// Either update the existing file access time or create the file.
    pub fn touch(&self) -> crate::Result<()> {
        let touch_result = if self.exists() {
            OpenOptions::new().read(true).open(self.path())
        } else {
            OpenOptions::new().write(true).create(true).open(self.path())
        };
        if let Err(error) = touch_result {
            let reason = error!(self.filename, format!("Error touching file ({}).", error));
            log::error!("{}", reason);
            Err(reason)
        } else {
            Ok(())
        }
    }

    /// Rename the weather file to another weather file.
    pub fn rename(&self, to: &WeatherFile) -> crate::Result<()> {
        if let Err(error) = fs::rename(&self.path, &to.path) {
            let reason = error!(self.filename, format!("Error renaming to {} ({})", to, error));
            log::error!("{}", reason);
            Err(reason)
        } else {
            Ok(())
        }
    }

    /// Copy the weather file to another weather file.
    pub fn copy(&self, to: &WeatherFile) -> crate::Result<()> {
        if let Err(error) = fs::copy(&self.path, &to.path) {
            let reason = error!(self.filename, format!("Error copying to {} ({})", to, error));
            log::error!("{}", reason);
            Err(reason)
        } else {
            Ok(())
        }
    }

    /// Safely get the underlying file stat metadata.
    fn metadata(&self) -> Option<Metadata> {
        match self.path.metadata() {
            Ok(metadata) => Some(metadata),
            Err(error) => {
                if error.kind() != ErrorKind::NotFound {
                    log::error!("{}", error!(&self.filename, &error));
                }
                None
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::testlib;
    use std::io::{Read, Write};

    #[test]
    fn weather_file() {
        let fixture = testlib::TestFixture::create();
        let filename = "test_file.dat";
        let mut testcase = WeatherFile::new(PathBuf::from(&fixture).join(filename));
        // verify metadata for a file that does not exist
        assert_eq!(testcase.filename, filename);
        assert!(!testcase.exists());
        assert_eq!(testcase.size(), 0);
        // create the file and content
        let content = "testcase file content...";
        testcase.touch().unwrap();
        testcase.writer().unwrap().write_all(content.as_bytes()).unwrap();
        assert!(testcase.exists());
        assert_eq!(testcase.size(), content.len() as u64);
        // verify reading the file content
        let mut file_content = String::new();
        let mut reader = testcase.reader().unwrap().read_to_string(&mut file_content).unwrap();
        assert_eq!(&file_content, content);
    }
}
