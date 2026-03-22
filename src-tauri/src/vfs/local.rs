//! Local file system backend for VFS
//! 
//! Uses memory-mapped files for efficient large file handling.

use super::{VfsBackend, VfsFile, VfsMetadata, VfsDirEntry};
use std::os::unix::fs::PermissionsExt;
use std::fs::{self, File, OpenOptions, DirEntry};
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::path::Path;
use memmap2::{Mmap, MmapMut};

/// Local file system backend
pub struct LocalBackend;

impl LocalBackend {
    pub fn new() -> Self {
        LocalBackend
    }
}

impl VfsBackend for LocalBackend {
    fn open_read(&self, path: &str) -> io::Result<Box<dyn VfsFile>> {
        let file = OpenOptions::new().read(true).open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        
        Ok(Box::new(LocalFile {
            path: path.to_string(),
            mmap: Some(mmap),
            writable_mmap: None,
            size: file.metadata()?.len(),
        }))
    }
    
    fn open_write(&self, path: &str) -> io::Result<Box<dyn VfsFile>> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let size = file.metadata()?.len();
        
        Ok(Box::new(LocalFile {
            path: path.to_string(),
            mmap: None,
            writable_mmap: None,
            size,
        }))
    }
    
    fn exists(&self, path: &str) -> io::Result<bool> {
        Ok(Path::new(path).exists())
    }
    
    fn metadata(&self, path: &str) -> io::Result<VfsMetadata> {
        let metadata = fs::metadata(path)?;
        
        Ok(VfsMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            created: metadata.created().ok(),
            permissions: Some(metadata.permissions().mode() as u32),
        })
    }
    
    fn read_dir(&self, path: &str) -> io::Result<Vec<VfsDirEntry>> {
        let entries = fs::read_dir(path)?;
        let mut result = Vec::new();
        
        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let full_path = entry.path().to_string_lossy().to_string();
            
            result.push(VfsDirEntry {
                name,
                path: full_path,
                metadata: VfsMetadata {
                    size: metadata.len(),
                    is_file: metadata.is_file(),
                    is_dir: metadata.is_dir(),
                    modified: metadata.modified().ok(),
                    accessed: metadata.accessed().ok(),
                    created: metadata.created().ok(),
                    permissions: Some(metadata.permissions().mode() as u32),
                },
            });
        }
        
        Ok(result)
    }
    
    fn create_dir(&self, path: &str) -> io::Result<()> {
        fs::create_dir_all(path)
    }
    
    fn remove_file(&self, path: &str) -> io::Result<()> {
        fs::remove_file(path)
    }
    
    fn remove_dir(&self, path: &str) -> io::Result<()> {
        fs::remove_dir_all(path)
    }
}

/// Local file implementation
pub struct LocalFile {
    path: String,
    mmap: Option<Mmap>,
    writable_mmap: Option<MmapMut>,
    size: u64,
}

impl LocalFile {
    fn ensure_writable(&mut self) -> io::Result<()> {
        if self.writable_mmap.is_none() {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.path)?;
            let mmap = unsafe { MmapMut::map_mut(&file)? };
            self.writable_mmap = Some(mmap);
        }
        Ok(())
    }
}

impl Read for LocalFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref mmap) = self.mmap {
            let len = buf.len().min(mmap.len());
            buf[..len].copy_from_slice(&mmap[..len]);
            Ok(len)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "File not open for reading"))
        }
    }
}

impl Write for LocalFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.ensure_writable()?;
        if let Some(ref mut mmap) = self.writable_mmap {
            let len = buf.len().min(mmap.len());
            mmap[..len].copy_from_slice(&buf[..len]);
            Ok(len)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to open for writing"))
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        if let Some(ref mut mmap) = self.writable_mmap {
            mmap.flush()
        } else {
            Ok(())
        }
    }
}

impl Seek for LocalFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        // Memory-mapped files don't need explicit seek
        // We handle offset in read_at/write_at
        Ok(0)
    }
}

impl VfsFile for LocalFile {
    fn size(&self) -> io::Result<u64> {
        Ok(self.size)
    }
    
    fn sync(&mut self) -> io::Result<()> {
        self.flush()
    }
    
    fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        if let Some(ref mmap) = self.mmap {
            let offset = offset as usize;
            let available = mmap.len().saturating_sub(offset);
            let to_read = buf.len().min(available);
            
            if to_read > 0 {
                buf[..to_read].copy_from_slice(&mmap[offset..offset + to_read]);
            }
            
            Ok(to_read)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "File not open for reading"))
        }
    }
    
    fn write_at(&mut self, offset: u64, buf: &[u8]) -> io::Result<usize> {
        self.ensure_writable()?;
        
        if let Some(ref mut mmap) = self.writable_mmap {
            let offset = offset as usize;
            let available = mmap.len().saturating_sub(offset);
            let to_write = buf.len().min(available);
            
            if to_write > 0 {
                mmap[offset..offset + to_write].copy_from_slice(&buf[..to_write]);
            }
            
            Ok(to_write)
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to open for writing"))
        }
    }
}

impl Drop for LocalFile {
    fn drop(&mut self) {
        // Flush on drop if writable
        if self.writable_mmap.is_some() {
            let _ = self.flush();
        }
    }
}
