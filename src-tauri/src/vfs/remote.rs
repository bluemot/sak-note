//! SFTP remote file system backend for VFS
//!
//! Uses libssh2 (via ssh2 crate) for SFTP operations.

use super::{VfsBackend, VfsFile, VfsMetadata, VfsDirEntry};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::net::TcpStream;
use std::path::Path;

/// SSH/SFTP connection configuration
pub struct SftpConfig {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub auth: SftpAuth,
    pub known_hosts_path: Option<String>,
}

/// Authentication methods
pub enum SftpAuth {
    Password(String),
    PrivateKey { path: String, passphrase: Option<String> },
    Agent,
}

/// SFTP backend
pub struct SftpBackend {
    session: ssh2::Session,
    sftp: ssh2::Sftp,
}

impl SftpBackend {
    pub fn new(config: SftpConfig) -> io::Result<Self> {
        // Connect to SSH server
        let tcp = TcpStream::connect((config.hostname.as_str(), config.port))?;
        tcp.set_nodelay(true)?;

        // Create SSH session
        let mut session = ssh2::Session::new()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to create SSH session"))?;

        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("SSH handshake failed: {}", e)))?;

        // Authenticate
        match config.auth {
            SftpAuth::Password(password) => {
                session.userauth_password(&config.username, &password)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Password auth failed: {}", e)))?;
            }
            SftpAuth::PrivateKey { path, passphrase } => {
                session.userauth_pubkey_file(
                    &config.username,
                    None,
                    Path::new(&path),
                    passphrase.as_deref(),
                )
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Key auth failed: {}", e)))?;
            }
            SftpAuth::Agent => {
                session.userauth_agent(&config.username)
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Agent auth failed: {}", e)))?;
            }
        }

        // Initialize SFTP
        let sftp = session.sftp()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("SFTP init failed: {}", e)))?;

        Ok(SftpBackend { session, sftp })
    }

    /// Check if connection is alive
    pub fn is_connected(&self) -> bool {
        // Try to stat root to check connection
        self.sftp.stat(Path::new("/")).is_ok()
    }
}

impl VfsBackend for SftpBackend {
    fn open_read(&self, path: &str) -> io::Result<Box<dyn VfsFile>> {
        let file = self.sftp.open(Path::new(path))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open file: {}", e)))?;

        let stat = file.stat()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to stat file: {}", e)))?;

        Ok(Box::new(SftpFile {
            handle: file,
            path: path.to_string(),
            size: stat.size.unwrap_or(0),
            position: 0,
        }))
    }

    fn open_write(&self, path: &str) -> io::Result<Box<dyn VfsFile>> {
        let file = self.sftp.open_mode(
            Path::new(path),
            ssh2::OpenMode::WRITE | ssh2::OpenMode::CREATE,
            ssh2::OpenFileType::File,
            // Default permissions
            0o644,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open file for write: {}", e)))?;

        let stat = file.stat()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to stat file: {}", e)))?;

        Ok(Box::new(SftpFile {
            handle: file,
            path: path.to_string(),
            size: stat.size.unwrap_or(0),
            position: 0,
        }))
    }

    fn exists(&self, path: &str) -> io::Result<bool> {
        match self.sftp.stat(Path::new(path)) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    fn metadata(&self, path: &str) -> io::Result<VfsMetadata> {
        let stat = self.sftp.stat(Path::new(path))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to stat: {}", e)))?;

        Ok(VfsMetadata {
            size: stat.size.unwrap_or(0),
            is_file: stat.is_file(),
            is_dir: stat.is_dir(),
            modified: stat.mtime.map(|t| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
            accessed: stat.atime.map(|t| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
            created: None, // SFTP doesn't typically provide creation time
            permissions: stat.perm.map(|p| p as u32),
        })
    }

    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>> {
        let mut dir = self.sftp.opendir(Path::new(path))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to open dir: {}", e)))?;

        let mut entries = Vec::new();

        while let Some((filename, stat)) = dir.read().map_err(|e| {
            io::Error::new(io::ErrorKind::Other, format!("Failed to read dir: {}", e))
        })? {
            // Skip . and ..
            if filename == "." || filename == ".." {
                continue;
            }

            let full_path = format!("{}/{}", path.trim_end_matches('/'), filename);

            entries.push(VfsDirEntry {
                name: filename,
                path: full_path,
                metadata: VfsMetadata {
                    size: stat.size.unwrap_or(0),
                    is_file: stat.is_file(),
                    is_dir: stat.is_dir(),
                    modified: stat.mtime.map(|t| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
                    accessed: stat.atime.map(|t| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
                    created: None,
                    permissions: stat.perm.map(|p| p as u32),
                },
            });
        }

        Ok(entries)
    }

    fn create_dir(&self, path: &str) -> io::Result<()> {
        self.sftp.mkdir(Path::new(path), 0o755)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to create dir: {}", e)))
    }

    fn remove_file(&self, path: &str) -> io::Result<()> {
        self.sftp.unlink(Path::new(path))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to remove file: {}", e)))
    }

    fn remove_dir(&self, path: &str) -> io::Result<()> {
        self.sftp.rmdir(Path::new(path))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to remove dir: {}", e)))
    }
}

/// SFTP file implementation
pub struct SftpFile {
    handle: ssh2::File,
    path: String,
    size: u64,
    position: u64,
}

impl Read for SftpFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // Read at current position
        let bytes_read = self.handle.read(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Read failed: {}", e)))?;

        self.position += bytes_read as u64;
        Ok(bytes_read)
    }
}

impl Write for SftpFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let bytes_written = self.handle.write(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Write failed: {}", e)))?;

        self.position += bytes_written as u64;
        Ok(bytes_written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.handle.flush()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Flush failed: {}", e)))
    }
}

impl Seek for SftpFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(offset) => offset,
            SeekFrom::End(offset) => {
                if offset >= 0 {
                    self.size + offset as u64
                } else {
                    self.size.saturating_sub((-offset) as u64)
                }
            }
            SeekFrom::Current(offset) => {
                if offset >= 0 {
                    self.position + offset as u64
                } else {
                    self.position.saturating_sub((-offset) as u64)
                }
            }
        };

        // SFTP doesn't have explicit seek, we track position
        self.position = new_pos;
        Ok(new_pos)
    }
}

impl VfsFile for SftpFile {
    fn size(&self) -> io::Result<u64> {
        // Refresh stat to get current size
        let stat = self.handle.stat()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Stat failed: {}", e)))?;
        Ok(stat.size.unwrap_or(0))
    }

    fn sync(&mut self) -> io::Result<()> {
        self.flush()
    }

    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        // Seek to offset then read
        self.seek(SeekFrom::Start(offset))?;
        self.read(buf)
    }

    fn write_at(&mut self, offset: u64, buf: &[u8]) -> io::Result<usize> {
        // Seek to offset then write
        self.seek(SeekFrom::Start(offset))?;
        self.write(buf)
    }
}
