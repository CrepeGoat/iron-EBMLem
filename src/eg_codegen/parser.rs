use core::convert::{From, TryInto};
use core::marker::PhantomData;

use crate::eg_codegen::element_defs;
use crate::element_defs::{ElementDef, ParentOf};
use crate::parser::{ElementReader, ElementState, ReaderError, StateError, StateOf};
use crate::stream::{parse, serialize, stream_diff};

// _Document Objects #########################################################################

type _DocumentState = ElementState<(), ()>;
type _DocumentReader<R> = ElementReader<R, _DocumentState>;

#[derive(Debug, Clone, PartialEq)]
enum _DocumentNextStates {
    Files(FilesState),
    None,
}

#[derive(Debug, PartialEq)]
enum _DocumentNextReaders<R> {
    Files(FilesReader<R>),
    None(R),
}

impl _DocumentNextStates {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> _DocumentNextReaders<R> {
        match self {
            _DocumentNextStates::Files(state) => {
                _DocumentNextReaders::Files(state.into_reader(reader))
            }
            _DocumentNextStates::None => _DocumentNextReaders::None(reader),
        }
    }
}

impl<R> From<_DocumentNextReaders<R>> for _DocumentNextStates {
    fn from(enumed_reader: _DocumentNextReaders<R>) -> _DocumentNextStates {
        match enumed_reader {
            _DocumentNextReaders::Files(reader) => _DocumentNextStates::Files(reader.state),
            _DocumentNextReaders::None(_) => _DocumentNextStates::None,
        }
    }
}

impl _DocumentState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> _DocumentReader<R> {
        _DocumentReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], (), ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], _DocumentNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, _DocumentNextStates::None)),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
                    .try_into()
                    .expect("overflow in storing element bytelength");

                self.bytes_left -= len + stream_diff(orig_stream, stream);

                Ok((
                    stream,
                    match id {
                        <element_defs::FilesDef as ElementDef>::ID => {
                            _DocumentNextStates::Files(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        id => return Err(nom::Err::Failure(StateError::InvalidChildID(None, id))),
                    },
                ))
            }
        }
    }
}

impl<R: std::io::BufRead> _DocumentReader<R> {
    fn new(reader: R, state: _DocumentState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<R, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, _next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(self.reader)
    }

    fn next(mut self) -> Result<_DocumentNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Files Objects #########################################################################

type FilesReader<R> = ElementReader<R, FilesState>;
type FilesState = ElementState<element_defs::FilesDef, _DocumentState>;

#[derive(Debug, Clone, PartialEq)]
enum FilesNextStates {
    File(FileState),
    Parent(_DocumentState),
}

#[derive(Debug, PartialEq)]
enum FilesNextReaders<R> {
    File(FileReader<R>),
    Parent(_DocumentReader<R>),
}

impl FilesNextStates {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> FilesNextReaders<R> {
        match self {
            FilesNextStates::File(state) => FilesNextReaders::File(state.into_reader(reader)),
            FilesNextStates::Parent(state) => FilesNextReaders::Parent(state.into_reader(reader)),
        }
    }
}

impl<R> From<FilesNextReaders<R>> for FilesNextStates {
    fn from(enumed_reader: FilesNextReaders<R>) -> FilesNextStates {
        match enumed_reader {
            FilesNextReaders::File(reader) => FilesNextStates::File(reader.state),
            FilesNextReaders::Parent(reader) => FilesNextStates::Parent(reader.state),
        }
    }
}

impl FilesState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> FilesReader<R> {
        FilesReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], _DocumentState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], FilesNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FilesNextStates::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
                    .try_into()
                    .expect("overflow in storing element bytelength");

                self.bytes_left -= len + stream_diff(orig_stream, stream);

                Ok((
                    stream,
                    match id {
                        <element_defs::FileDef as ElementDef>::ID => {
                            FilesNextStates::File(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        id => {
                            return Err(nom::Err::Failure(StateError::InvalidChildID(
                                Some(<<Self as StateOf>::Element as ElementDef>::ID),
                                id,
                            )))
                        }
                    },
                ))
            }
        }
    }
}

impl<R: std::io::BufRead> FilesReader<R> {
    fn new(reader: R, state: FilesState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<_DocumentReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FilesNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// File Objects #########################################################################

type FileState = ElementState<element_defs::FileDef, FilesState>;
type FileReader<R> = ElementReader<R, FileState>;

#[derive(Debug, Clone, PartialEq)]
enum FileNextStates {
    FileName(FileNameState),
    MimeType(MimeTypeState),
    ModificationTimestamp(ModificationTimestampState),
    Data(DataState),
    Parent(FilesState),
}

#[derive(Debug, PartialEq)]
enum FileNextReaders<R> {
    FileName(FileNameReader<R>),
    MimeType(MimeTypeReader<R>),
    ModificationTimestamp(ModificationTimestampReader<R>),
    Data(DataReader<R>),
    Parent(FilesReader<R>),
}

impl<R> From<FileNextReaders<R>> for FileNextStates {
    fn from(enumed_reader: FileNextReaders<R>) -> Self {
        match enumed_reader {
            FileNextReaders::FileName(reader) => Self::FileName(reader.state),
            FileNextReaders::MimeType(reader) => Self::MimeType(reader.state),
            FileNextReaders::ModificationTimestamp(reader) => {
                Self::ModificationTimestamp(reader.state)
            }
            FileNextReaders::Data(reader) => Self::Data(reader.state),
            FileNextReaders::Parent(reader) => Self::Parent(reader.state),
        }
    }
}

impl FileNextStates {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> FileNextReaders<R> {
        match self {
            Self::FileName(state) => FileNextReaders::<R>::FileName(state.into_reader(reader)),
            Self::MimeType(state) => FileNextReaders::<R>::MimeType(state.into_reader(reader)),
            Self::ModificationTimestamp(state) => {
                FileNextReaders::<R>::ModificationTimestamp(state.into_reader(reader))
            }
            Self::Data(state) => FileNextReaders::<R>::Data(state.into_reader(reader)),
            Self::Parent(state) => FileNextReaders::<R>::Parent(state.into_reader(reader)),
        }
    }
}

impl FileState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> FileReader<R> {
        FileReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FilesState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(mut self, stream: &[u8]) -> nom::IResult<&[u8], FileNextStates, StateError> {
        match self {
            Self {
                bytes_left: 0,
                parent_state: _,
                _phantom: _,
            } => Ok((stream, FileNextStates::Parent(self.parent_state))),
            _ => {
                let orig_stream = stream;

                let (stream, id) = parse::element_id(stream).map_err(nom::Err::convert)?;
                let (stream, len) = parse::element_len(stream).map_err(nom::Err::convert)?;
                let len: usize = len
                    .ok_or(nom::Err::Failure(StateError::Unimplemented(
                        "TODO: handle optionally unsized elements",
                    )))?
                    .try_into()
                    .expect("overflow in storing element bytelength");

                self.bytes_left -= len + stream_diff(orig_stream, stream);

                Ok((
                    stream,
                    match id {
                        <element_defs::FileNameDef as ElementDef>::ID => {
                            FileNextStates::FileName(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::MimeTypeDef as ElementDef>::ID => {
                            FileNextStates::MimeType(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::ModificationTimestampDef as ElementDef>::ID => {
                            FileNextStates::ModificationTimestamp(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        <element_defs::DataDef as ElementDef>::ID => {
                            FileNextStates::Data(ElementState {
                                bytes_left: len,
                                parent_state: self,
                                _phantom: PhantomData,
                            })
                        }
                        id => {
                            return Err(nom::Err::Failure(StateError::InvalidChildID(
                                Some(<<Self as StateOf>::Element as ElementDef>::ID),
                                id,
                            )))
                        }
                    },
                ))
            }
        }
    }
}

impl<R: std::io::BufRead> FileReader<R> {
    fn new(reader: R, state: FileState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<FilesReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FileNextReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// FileName Objects #########################################################################

type FileNameState = ElementState<element_defs::FileNameDef, FileState>;
type FileNameReader<R> = ElementReader<R, FileNameState>;

impl FileNameState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> FileNameReader<R> {
        FileNameReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream).map_err(nom::Err::convert)
    }
}

impl<R: std::io::BufRead> FileNameReader<R> {
    fn new(reader: R, state: FileNameState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// MimeType Objects #########################################################################

type MimeTypeState = ElementState<element_defs::MimeTypeDef, FileState>;
type MimeTypeReader<R> = ElementReader<R, MimeTypeState>;

impl MimeTypeState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> MimeTypeReader<R> {
        MimeTypeReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream).map_err(nom::Err::convert)
    }
}

impl<R: std::io::BufRead> MimeTypeReader<R> {
    fn new(reader: R, state: MimeTypeState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// ModificationTimestamp Objects #########################################################################

type ModificationTimestampState = ElementState<element_defs::ModificationTimestampDef, FileState>;
type ModificationTimestampReader<R> = ElementReader<R, ModificationTimestampState>;

impl ModificationTimestampState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> ModificationTimestampReader<R> {
        ModificationTimestampReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream).map_err(nom::Err::convert)
    }
}

impl<R: std::io::BufRead> ModificationTimestampReader<R> {
    fn new(reader: R, state: ModificationTimestampState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Data Objects #########################################################################

type DataState = ElementState<element_defs::DataDef, FileState>;
type DataReader<R> = ElementReader<R, DataState>;

impl DataState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> DataReader<R> {
        DataReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], FileState, StateError> {
        self.skip(stream).map_err(nom::Err::convert)
    }
}

impl<R: std::io::BufRead> DataReader<R> {
    fn new(reader: R, state: DataState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<FileReader<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

// Void Objects #########################################################################

#[derive(Debug, Clone, PartialEq)]
enum VoidPrevStates {
    Files(FilesState),
    File(FileState),
}
#[derive(Debug, PartialEq)]
enum VoidPrevReaders<R> {
    Files(FilesReader<R>),
    File(FileReader<R>),
}

impl VoidPrevStates {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> VoidPrevReaders<R> {
        match self {
            VoidPrevStates::Files(state) => VoidPrevReaders::Files(state.into_reader(reader)),
            VoidPrevStates::File(state) => VoidPrevReaders::File(state.into_reader(reader)),
        }
    }
}

impl<R> From<VoidPrevReaders<R>> for VoidPrevStates {
    fn from(enumed_reader: VoidPrevReaders<R>) -> VoidPrevStates {
        match enumed_reader {
            VoidPrevReaders::Files(reader) => VoidPrevStates::Files(reader.state),
            VoidPrevReaders::File(reader) => VoidPrevStates::File(reader.state),
        }
    }
}

type VoidState = ElementState<element_defs::DataDef, VoidPrevStates>;
type VoidReader<R> = ElementReader<R, VoidState>;

impl VoidState {
    fn into_reader<R: std::io::BufRead>(self, reader: R) -> VoidReader<R> {
        VoidReader::new(reader, self)
    }

    fn skip(self, stream: &[u8]) -> nom::IResult<&[u8], VoidPrevStates, ()> {
        let (stream, _) = nom::bytes::streaming::take(self.bytes_left)(stream)?;
        Ok((stream, self.parent_state))
    }

    fn next(self, stream: &[u8]) -> nom::IResult<&[u8], VoidPrevStates, StateError> {
        self.skip(stream).map_err(nom::Err::convert)
    }
}

impl<R: std::io::BufRead> VoidReader<R> {
    fn new(reader: R, state: VoidState) -> Self {
        Self { reader, state }
    }

    fn skip(mut self) -> Result<VoidPrevReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.skip(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }

    fn next(mut self) -> Result<VoidPrevReaders<R>, ReaderError> {
        let stream = self.reader.fill_buf()?;

        let (next_stream, next_state) = self.state.next(stream).map_err(nom::Err::convert)?;
        let stream_dist = stream.len() - next_stream.len();
        self.reader.consume(stream_dist);

        Ok(next_state.into_reader(self.reader))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    mod document {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                _DocumentState{bytes_left: 7, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], _DocumentNextStates::Files(FilesState{bytes_left: 2, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF][..], _DocumentNextStates::None)
            ),
        )]
        fn next(
            element: _DocumentState,
            source: &'static [u8],
            expt_result: (&'static [u8], _DocumentNextStates),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                _DocumentState{bytes_left: 7, parent_state: (), _phantom: PhantomData},
                &[0x19, 0x46, 0x69, 0x6C, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], ())
            ),
        )]
        fn skip(element: _DocumentState, source: &'static [u8], expt_result: (&'static [u8], ())) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod files {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FilesState{bytes_left: 5, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x46, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::File(FileState{bytes_left: 2, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF],
                (&[0xFF, 0xFF, 0xFF][..], FilesNextStates::Parent(_DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}))
            ),
        )]
        fn next(
            element: FilesState,
            source: &'static [u8],
            expt_result: (&'static [u8], FilesNextStates),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FilesState{bytes_left: 5, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FilesState,
            source: &'static [u8],
            expt_result: (&'static [u8], _DocumentState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod file {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::FileName(FileNameState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x4D, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::MimeType(MimeTypeState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x54, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::ModificationTimestamp(ModificationTimestampState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x46, 0x64, 0x82, 0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Data(DataState{bytes_left: 2, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}))
            ),
            case(
                FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF],
                (&[0xFF, 0xFF][..], FileNextStates::Parent(FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}))
            ),
        )]
        fn next(
            element: FileState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileNextStates),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FileState{bytes_left: 5, parent_state: FilesState{bytes_left: 1, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0x61, 0x4E, 0x82, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FilesState{bytes_left: 1, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FileState,
            source: &'static [u8],
            expt_result: (&'static [u8], FilesState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod filename {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: FileNameState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                FileNameState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: FileNameState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod mimetype {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: MimeTypeState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                MimeTypeState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: MimeTypeState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod modificationtimestamp {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: ModificationTimestampState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                ModificationTimestampState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: ModificationTimestampState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }

    mod data {
        use super::*;

        #[rstest(element, source, expt_result,
            case(
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}),
            ),
        )]
        fn next(
            element: DataState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.next(source).unwrap(), expt_result);
        }

        #[rstest(element, source, expt_result,
            case(
                DataState{bytes_left: 3, parent_state: FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData},
                &[0xFF, 0xFF, 0xFF, 0xFF],
                (&[0xFF][..], FileState{bytes_left: 0, parent_state: FilesState{bytes_left: 0, parent_state: _DocumentState{bytes_left: 0, parent_state: (), _phantom: PhantomData}, _phantom: PhantomData}, _phantom: PhantomData})
            ),
        )]
        fn skip(
            element: DataState,
            source: &'static [u8],
            expt_result: (&'static [u8], FileState),
        ) {
            assert_eq!(element.skip(source).unwrap(), expt_result);
        }
    }
}
