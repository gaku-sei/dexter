#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    borrow::Cow,
    fs::File,
    io::{self, Cursor, Read, Seek, Write},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    path::Path,
    result,
};

use bytes::Bytes;
use camino::Utf8Path;
use zip::{read::ZipFile, write::FileOptions, ZipArchive, ZipWriter};

pub use crate::errors::{Error, Result};

pub mod errors;
pub mod image;

/// We artificially limit the amount of accepted files to 65535 files per Cbz
/// First as it'd be rather impractical for the user to read such enormous Cbz
/// Also, this size has been chosen as it was the limit of the very first zip spec
pub static MAX_FILE_NUMBER: usize = u16::MAX as usize;

/// The length of 65535 used to name the inserted file with a proper padding
pub static COUNTER_SIZE: usize = 5;

pub trait Cbz {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait CbzRead: Cbz {
    fn file_names(&self) -> Vec<&str>;

    /// Lookup the file by `name` in Cbz and returns a `CbzFile`
    ///
    /// ## Errors
    ///
    /// Fails if file size is too large to fit a `usize` on host machine
    /// or if the content can't be read
    fn read_by_name(&mut self, name: &str) -> Result<CbzFile<'_>>;

    fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(Result<CbzFile<'_>>),
    {
        let mut file_names = self
            .file_names()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name));
        }
    }

    /// Iterate over files present in the Cbz.
    /// If the closure returns an error, this error is returned immediately.
    ///
    /// ## Errors
    ///
    /// Returns an error immediately if the provided closure returns an error
    fn try_for_each<F, E>(&mut self, mut f: F) -> result::Result<(), E>
    where
        F: FnMut(Result<CbzFile<'_>>) -> result::Result<(), E>,
    {
        let mut file_names = self
            .file_names()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        file_names.sort();

        for file_name in file_names {
            f(self.read_by_name(&file_name))?;
        }

        Ok(())
    }
}

pub trait CbzWrite {
    fn size(&self) -> usize;

    /// High level `insert` method, prefer this over the raw `insert_from_bytes_slice_with_options` method
    ///
    /// ## Errors
    ///
    /// Same behavior as `insert_from_bytes_slice_with_options`
    fn insert(&mut self, insertion: CbzWriterInsertion<'_, '_, Auto>) -> Result<()> {
        let filename = format!(
            "{:0>COUNTER_SIZE$}.{}",
            self.size() + 1,
            insertion.extension
        );

        self.insert_from_bytes_slice_with_options(
            filename,
            &insertion.bytes,
            insertion.file_options,
        )
    }

    /// High level `insert_at` method, prefer this over the raw `insert_from_bytes_slice_with_options` method
    ///
    /// ## Errors
    ///
    /// Same behavior as `insert_from_bytes_slice_with_options`
    fn insert_at(&mut self, insertion: CbzWriterInsertion<'_, '_, Indexed>) -> Result<()> {
        let filename = format!(
            "{:0>COUNTER_SIZE$}.{}",
            *insertion.type_, insertion.extension
        );

        self.insert_from_bytes_slice_with_options(
            filename,
            &insertion.bytes,
            insertion.file_options,
        )
    }

    /// High level `insert_custom_str` method, prefer this over the raw `insert_from_bytes_slice_with_options` method
    ///
    /// ## Errors
    ///
    /// Same behavior as `insert_from_bytes_slice_with_options`
    fn insert_custom_str(
        &mut self,
        insertion: CbzWriterInsertion<'_, '_, CustomStr<'_>>,
    ) -> Result<()> {
        let filename = format!(
            "{:0>COUNTER_SIZE$}.{}",
            &*insertion.type_, insertion.extension
        );

        self.insert_from_bytes_slice_with_options(
            filename,
            &insertion.bytes,
            insertion.file_options,
        )
    }

    /// This is the method ultimately called to insert the bytes into the Cbz
    ///
    /// ## Errors
    ///
    /// This fails if the Cbz writer can't be written or if it's full (i.e. its size equals `MAX_FILE_NUMBER`)
    fn insert_from_bytes_slice_with_options(
        &mut self,
        filename: impl Into<String>,
        bytes: &[u8],
        file_options: FileOptions,
    ) -> Result<()>;
}

pub struct CbzFile<'a>(ZipFile<'a>);

impl<'a> CbzFile<'a> {
    pub fn name(&self) -> &str {
        self.0.name()
    }

    pub fn size(&self) -> u64 {
        self.0.size()
    }

    /// Convert the file convent to  `Bytes`
    ///
    /// ## Errors
    ///
    /// Fails if file size is too large to fit a `usize` on host machine
    /// or if the content can't be read
    pub fn to_bytes(&mut self) -> Result<Bytes> {
        let mut buf = Vec::with_capacity(
            self.size()
                .try_into()
                .map_err(|_| Error::CbzFileSizeConversion)?,
        );

        self.0.read_to_end(&mut buf)?;

        Ok(buf.into())
    }

    /// Copy the content of the file to the provided `Write`
    ///
    /// ## Errors
    ///
    /// Fails if copy itself fails
    pub fn copy_to(&mut self, writer: &mut impl Write) -> Result<u64> {
        io::copy(&mut self.0, writer).map_err(Into::into)
    }
}

impl<'a> Read for CbzFile<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<'a> From<ZipFile<'a>> for CbzFile<'a> {
    fn from(zip_file: ZipFile<'a>) -> Self {
        Self(zip_file)
    }
}

impl<'a> From<CbzFile<'a>> for ZipFile<'a> {
    fn from(cbz_file: CbzFile<'a>) -> Self {
        cbz_file.0
    }
}

#[derive(Debug)]
pub struct CbzReader<'a, R> {
    archive: ZipArchive<R>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, R> CbzReader<'a, R> {
    pub fn new(archive: ZipArchive<R>) -> Self {
        Self {
            archive,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, R> CbzReader<'a, R>
where
    R: Read + Seek,
{
    /// Creates `CbzReader` from a `Read`
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn from_reader(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader)?;

        Ok(Self::new(archive))
    }
}

impl<'a> CbzReader<'a, File> {
    /// Creates `CbzReader` from a path
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref())?;

        Self::from_reader(file)
    }
}

impl<'a, 'b> CbzReader<'a, Cursor<&'b [u8]>> {
    /// Creates `CbzReader` from a bytes slice
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn from_bytes_slice(bytes: &'b [u8]) -> Result<Self> {
        let cursor = Cursor::new(bytes);

        Self::from_reader(cursor)
    }
}

impl<'a> CbzReader<'a, Cursor<Bytes>> {
    /// Creates `CbzReader` from bytes
    ///
    /// ## Errors
    ///
    /// Fails if the underlying `ZipArchive` can't be created
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Result<Self> {
        let cursor = Cursor::new(bytes.into());

        Self::from_reader(cursor)
    }
}

impl<'a, R> Cbz for CbzReader<'a, R>
where
    R: Read + Seek,
{
    fn len(&self) -> usize {
        self.archive.len()
    }
}

impl<'a, R> CbzRead for CbzReader<'a, R>
where
    R: Read + Seek,
{
    fn file_names(&self) -> Vec<&str> {
        self.archive.file_names().collect()
    }

    fn read_by_name(&mut self, name: &str) -> Result<CbzFile<'_>> {
        let archive_file = self.archive.by_name(name)?;

        Ok(archive_file.into())
    }
}

impl<'a, R> From<ZipArchive<R>> for CbzReader<'a, R> {
    fn from(archive: ZipArchive<R>) -> Self {
        Self::new(archive)
    }
}

impl<'a, R> From<CbzReader<'a, R>> for ZipArchive<R> {
    fn from(cbz: CbzReader<'a, R>) -> Self {
        cbz.archive
    }
}

impl<'a, R> Deref for CbzReader<'a, R> {
    type Target = ZipArchive<R>;

    fn deref(&self) -> &Self::Target {
        &self.archive
    }
}

impl<'a, R> DerefMut for CbzReader<'a, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.archive
    }
}

pub struct CbzWriter<'a, W: Write + Seek> {
    archive: ZipWriter<W>,
    size: usize,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a, W> CbzWriter<'a, W>
where
    W: Write + Seek,
{
    pub fn new(archive: ZipWriter<W>) -> Self {
        Self {
            archive,
            size: 0,
            _lifetime: PhantomData,
        }
    }
}

impl<'a, W> CbzWriter<'a, W>
where
    W: Write + Seek,
{
    /// Creates a `CbzWriter` from a `Write`
    fn from_writer(writer: W) -> Self {
        let archive = ZipWriter::new(writer);

        Self::new(archive)
    }

    /// Terminates the Cbz archiving, called on drop anyway but error can't be handled
    ///
    /// ## Errors
    ///
    /// Same errors as the underlying `ZipWriter::finish` method
    pub fn finish(&mut self) -> Result<CbzWriterFinished<W>> {
        let writer = self.archive.finish()?;

        Ok(CbzWriterFinished::new(writer))
    }
}

impl<'a> Default for CbzWriter<'a, Cursor<Vec<u8>>> {
    fn default() -> Self {
        Self::from_writer(Cursor::new(Vec::new()))
    }
}

impl<'a, W> Cbz for CbzWriter<'a, W>
where
    W: Write + Seek,
{
    fn len(&self) -> usize {
        self.size
    }
}

impl<'a, W> CbzWrite for CbzWriter<'a, W>
where
    W: Write + Seek,
{
    fn size(&self) -> usize {
        self.size
    }

    fn insert_from_bytes_slice_with_options(
        &mut self,
        filename: impl Into<String>,
        bytes: &[u8],
        file_options: FileOptions,
    ) -> Result<()> {
        if self.size >= MAX_FILE_NUMBER {
            return Err(Error::CbzTooLarge(MAX_FILE_NUMBER));
        }

        self.archive.start_file(filename, file_options)?;

        self.archive.write_all(bytes)?;

        self.size += 1;

        Ok(())
    }
}

impl<'a, W> From<ZipWriter<W>> for CbzWriter<'a, W>
where
    W: Write + Seek,
{
    fn from(archive: ZipWriter<W>) -> Self {
        Self::new(archive)
    }
}

impl<'a, W> From<CbzWriter<'a, W>> for ZipWriter<W>
where
    W: Write + Seek,
{
    fn from(cbz: CbzWriter<'a, W>) -> Self {
        cbz.archive
    }
}

pub struct ManualCbzWriter<'a, W: Write + Seek>(CbzWriter<'a, W>);

pub struct Indexed(usize);

impl Deref for Indexed {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct CustomStr<'a>(&'a str);

impl<'a> Deref for CustomStr<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct Auto;

pub struct CbzWriterInsertion<'a, 'b, T> {
    extension: Cow<'a, str>,
    file_options: FileOptions,
    bytes: Cow<'b, [u8]>,
    type_: T,
}

#[derive(Debug, PartialEq, Eq)]
enum InsertionTypeDescriber<'a> {
    Filename(&'a str),
    Extension(&'a str),
}

pub struct CbzWriterInsertionBuilder<'a, 'b> {
    type_describer: InsertionTypeDescriber<'a>,
    file_options: Option<FileOptions>,
    bytes: Option<Cow<'b, [u8]>>,
}

impl<'a, 'b> CbzWriterInsertionBuilder<'a, 'b> {
    pub fn from_filename(filename: &'a (impl AsRef<str> + ?Sized)) -> Self {
        Self {
            type_describer: InsertionTypeDescriber::Filename(filename.as_ref()),
            file_options: None,
            bytes: None,
        }
    }

    pub fn from_extension(extension: &'a (impl AsRef<str> + ?Sized)) -> Self {
        Self {
            type_describer: InsertionTypeDescriber::Extension(extension.as_ref()),
            file_options: None,
            bytes: None,
        }
    }

    #[must_use]
    pub fn set_file_options(mut self, file_options: FileOptions) -> Self {
        self.file_options = Some(file_options);

        self
    }

    #[must_use]
    pub fn set_bytes_ref(mut self, bytes: &'b impl AsRef<[u8]>) -> Self {
        self.bytes = Some(bytes.as_ref().into());

        self
    }

    #[must_use]
    pub fn set_bytes(mut self, bytes: impl Into<Vec<u8>>) -> Self {
        self.bytes = Some(bytes.into().into());

        self
    }

    /// Set the `bytes` field from the provided `Read`
    ///
    /// ## Errors
    ///
    /// Can fail when reading the provided `Read`
    pub fn set_bytes_from_reader(mut self, mut reader: impl Read) -> Result<Self> {
        let mut buf = Vec::new();

        reader.read_to_end(&mut buf)?;

        self.bytes = Some(buf.into());

        Ok(self)
    }

    /// Builds a `CbzWriterInsertion`
    ///
    /// ## Errors
    ///
    /// Fails if the `bytes` field hasn't been populated or if the extension is empty
    pub fn build(self) -> Result<CbzWriterInsertion<'a, 'b, Auto>> {
        self.inner_build(Auto)
    }

    /// Builds a `CbzWriterInsertion`
    ///
    /// ## Errors
    ///
    /// Fails if the `bytes` field hasn't been populated or if the extension is empty
    pub fn build_indexed(self, index: usize) -> Result<CbzWriterInsertion<'a, 'b, Indexed>> {
        self.inner_build(Indexed(index))
    }

    /// Builds a `CbzWriterInsertion`
    ///
    /// ## Errors
    ///
    /// Fails if the `bytes` field hasn't been populated or if the extension is empty
    pub fn build_custom_str<'c>(
        self,
        s: &'c str,
    ) -> Result<CbzWriterInsertion<'a, 'b, CustomStr<'c>>> {
        self.inner_build(CustomStr(s))
    }

    fn inner_build<T>(self, type_: T) -> Result<CbzWriterInsertion<'a, 'b, T>> {
        let Some(bytes) = self.bytes else {
            return Err(Error::CbzInsertionNoBytes);
        };

        let extension = match self.type_describer {
            InsertionTypeDescriber::Extension(extension) => {
                if extension.is_empty() {
                    return Err(Error::CbzInsertionNoExtension);
                }

                extension.into()
            }
            InsertionTypeDescriber::Filename(filename) => {
                let extension = Utf8Path::new(filename)
                    .extension()
                    .and_then(|extension| (!extension.is_empty()).then_some(extension.to_string()));

                let Some(extension) = extension else {
                    return Err(Error::CbzInsertionNoExtension);
                };

                extension.into()
            }
        };

        Ok(CbzWriterInsertion {
            extension,
            file_options: self.file_options.unwrap_or_default(),
            bytes,
            type_,
        })
    }
}

pub struct CbzWriterFinished<W> {
    writer: W,
}

impl<W> CbzWriterFinished<W> {
    fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<T: AsRef<[u8]>> AsRef<[u8]> for CbzWriterFinished<Cursor<T>> {
    fn as_ref(&self) -> &[u8] {
        self.writer.get_ref().as_ref()
    }
}

impl<W> Deref for CbzWriterFinished<W> {
    type Target = W;

    fn deref(&self) -> &Self::Target {
        &self.writer
    }
}

impl<W> DerefMut for CbzWriterFinished<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.writer
    }
}

impl CbzWriterFinished<Cursor<Vec<u8>>> {
    /// Writes self into provided writer
    ///
    /// ## Errors
    ///
    /// Fails on write error
    pub fn write_to(self, mut writer: impl Write) -> Result<()> {
        writer.write_all(&self.writer.into_inner())?;

        Ok(())
    }

    /// Writes self into a File (that will be created) located under the provided path
    ///
    /// ## Errors
    ///
    /// Can fail on file creation or when writing the file content
    pub fn write_to_path(self, path: impl AsRef<Utf8Path>) -> Result<()> {
        let mut file = File::create(path.as_ref())?;

        self.write_to(&mut file)?;

        Ok(())
    }
}
