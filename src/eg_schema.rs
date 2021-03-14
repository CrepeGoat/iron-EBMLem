/*
Example schema (https://github.com/ietf-wg-cellar/ebml-specification/blob/master/ebml_schema_example.xml):

<?xml version="1.0" encoding="utf-8"?>
<EBMLSchema xmlns="urn:ietf:rfc:8794"
  docType="files-in-ebml-demo" version="1">
 <!-- constraints to the range of two EBML Header Elements -->
 <element name="EBMLReadVersion" path="\EBML\EBMLReadVersion"
   id="0x42F7" minOccurs="1" maxOccurs="1" range="1" default="1"
   type="uinteger"/>
 <element name="EBMLMaxSizeLength"
   path="\EBML\EBMLMaxSizeLength" id="0x42F3" minOccurs="1"
   maxOccurs="1" range="8" default="8" type="uinteger"/>
 <!-- Root Element-->
 <element name="Files" path="\Files" id="0x1946696C"
   type="master">
  <documentation lang="en"
    purpose="definition">Container of data and
  attributes representing one or many files.</documentation>
 </element>
 <element name="File" path="\Files\File" id="0x6146"
   type="master" minOccurs="1">
  <documentation lang="en" purpose="definition">
    An attached file.
  </documentation>
 </element>
 <element name="FileName" path="\Files\File\FileName"
   id="0x614E" type="utf-8"
   minOccurs="1">
  <documentation lang="en" purpose="definition">
    Filename of the attached file.
  </documentation>
 </element>
 <element name="MimeType" path="\Files\File\MimeType"
   id="0x464D" type="string"
     minOccurs="1">
  <documentation lang="en" purpose="definition">
    MIME type of the file.
  </documentation>
 </element>
 <element name="ModificationTimestamp"
   path="\Files\File\ModificationTimestamp" id="0x4654"
   type="date" minOccurs="1">
  <documentation lang="en" purpose="definition">
    Modification timestamp of the file.
  </documentation>
 </element>
 <element name="Data" path="\Files\File\Data" id="0x4664"
   type="binary" minOccurs="1">
  <documentation lang="en" purpose="definition">
    The data of the file.
  </documentation>
 </element>
</EBMLSchema>
*/

use std::convert::TryInto;

use crate::schema_types::Bound;
use crate::schema_types::{
    BinaryElement, DateElement, Element, MasterElement, RangeDef, StringElement, UIntElement,
    UTF8Element,
};
use crate::schema_types::{ElementParsingStage, EmptyEnum};
use crate::stream::{parse, ElementLength};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Implicit Items

// parent: None
#[derive(Debug, Clone)]
struct Document(
    usize,
    ElementParsingStage<<Self as MasterElement>::SubElements, <Self as MasterElement>::SubGlobals>,
);

impl Element for Document {
    const ID: u32 = 0; // no ID for this element

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;

    fn next<'a>(&mut self, stream: &'a [u8]) -> nom::IResult<&'a [u8], bool, ()> {
        match self {
            Self(_, ElementParsingStage::Start) | Self(_, ElementParsingStage::Interlude) => {
                let (stream, id) = parse::element_id(stream)?;
                match id {
                    EBML::ID => {
                        let (stream, len) = parse::element_len(stream)?;
                        match len {
                            ElementLength::Known(len) => {
                                let len: usize = len.try_into().unwrap();
                                self.0 -= len;
                                self.1 =
                                    ElementParsingStage::Child(Document_SubElements::EBML_Variant(
                                        EBML(len, ElementParsingStage::Start),
                                    ));

                                Ok((stream, false))
                            }
                            ElementLength::Unknown => Err(nom::Err::Failure(())),
                        }
                    }
                    Files::ID => {
                        let (stream, len) = parse::element_len(stream)?;
                        match len {
                            ElementLength::Known(len) => {
                                let len: usize = len.try_into().unwrap();
                                self.0 -= len;
                                self.1 = ElementParsingStage::Child(
                                    Document_SubElements::Files_Variant(Files(
                                        len,
                                        ElementParsingStage::Start,
                                    )),
                                );

                                Ok((stream, false))
                            }
                            ElementLength::Unknown => Err(nom::Err::Failure(())),
                        }
                    }
                    _ => Err(nom::Err::Failure(())),
                }
            }
            Self(_, ElementParsingStage::Finish) | Self(_, ElementParsingStage::EndOfStream) => {
                Ok((stream, true))
            }
            Self(length_rem, ElementParsingStage::Child(variant)) => {
                let (stream, ended) = match variant {
                    Document_SubElements::EBML_Variant(e) => e.next(stream),
                    Document_SubElements::Files_Variant(e) => e.next(stream),
                }?;

                if ended {
                    self.1 = if *length_rem == 0 {
                        ElementParsingStage::Finish
                    } else {
                        ElementParsingStage::Interlude
                    };
                }

                Ok((stream, false))
            }
            Self(_length_rem, ElementParsingStage::Global(_global, _child)) => todo!(),
        }
    }
}

impl MasterElement for Document {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = Some(true);
    const RECURSIVE: Option<bool> = None;

    type SubElements = Document_SubElements;
    type SubGlobals = EmptyEnum;
}

#[derive(Debug, Clone)]
enum Document_SubElements {
    EBML_Variant(EBML),
    Files_Variant(Files),
}

// parent: None
#[derive(Debug, Clone)]
struct EBML(
    usize,
    ElementParsingStage<<Self as MasterElement>::SubElements, <Self as MasterElement>::SubGlobals>,
);

impl Element for EBML {
    const ID: u32 = 0x1A45DFA3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for EBML {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;

    type SubElements = EBML_SubElements;
    type SubGlobals = EmptyEnum;
}

#[derive(Debug, Clone)]
enum EBML_SubElements {
    EBMLVersion_Variant(EBMLVersion),
    EBMLReadVersion_Variant(EBMLReadVersion),
    EBMLMaxIDLength_Variant(EBMLMaxIDLength),
    EBMLMaxSizeLength_Variant(EBMLMaxSizeLength),
    DocType_Variant(DocType),
    DocTypeVersion_Variant(DocTypeVersion),
    DocTypeReadVersion_Variant(DocTypeReadVersion),
    DocTypeExtension_Variant(DocTypeExtension),
}

// parent: EBML
#[derive(Debug, Clone)]
struct EBMLVersion {}

impl Element for EBMLVersion {
    const ID: u32 = 0x4286;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

/*
// parent: EBML
#[derive(Debug, Clone)]
struct EBMLReadVersion {}

impl Element for EBMLReadVersion {
    const ID: u32 = 0x42F7;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(1));
    const DEFAULT: Option<u64> = Some(1);
}
*/

// parent: EBML
#[derive(Debug, Clone)]
struct EBMLMaxIDLength {}

impl Element for EBMLMaxIDLength {
    const ID: u32 = 0x42F2;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxIDLength {
    const RANGE: Option<RangeDef<u64>> =
        Some(RangeDef::IsWithin(Bound::Included(4), Bound::Unbounded));
    const DEFAULT: Option<u64> = Some(4);
}

/*
// parent: EBML
#[derive(Debug, Clone)]
struct EBMLMaxSizeLength {}

impl Element for EBMLMaxSizeLength {
    const ID: u32 = 0x42F3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxSizeLength {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(8);
}
*/

// parent: EBML
#[derive(Debug, Clone)]
struct DocType {}

impl Element for DocType {
    const ID: u32 = 0x4282;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> =
        Some(RangeDef::IsWithin(Bound::Excluded(0), Bound::Unbounded));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for DocType {
    const DEFAULT: Option<&'static str> = None;
}

// parent: EBML
#[derive(Debug, Clone)]
struct DocTypeVersion {}

impl Element for DocTypeVersion {
    const ID: u32 = 0x4287;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

// parent: EBML
#[derive(Debug, Clone)]
struct DocTypeReadVersion {}

impl Element for DocTypeReadVersion {
    const ID: u32 = 0x4285;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = Some(1);
}

// parent: EBML
#[derive(Debug, Clone)]
struct DocTypeExtension(
    usize,
    ElementParsingStage<<Self as MasterElement>::SubElements, <Self as MasterElement>::SubGlobals>,
);

impl Element for DocTypeExtension {
    const ID: u32 = 0x4281;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for DocTypeExtension {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;

    type SubElements = DocTypeExtension_SubElements;
    type SubGlobals = EmptyEnum;
}

#[derive(Debug, Clone)]
enum DocTypeExtension_SubElements {
    DocTypeExtensionName_Variant(DocTypeExtensionName),
    DocTypeExtensionVersion_Variant(DocTypeExtensionVersion),
}

// parent: DocTypeExtension
#[derive(Debug, Clone)]
struct DocTypeExtensionName {}

impl Element for DocTypeExtensionName {
    const ID: u32 = 0x4283;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> =
        Some(RangeDef::IsWithin(Bound::Included(1), Bound::Unbounded));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for DocTypeExtensionName {
    const DEFAULT: Option<&'static str> = None;
}

// parent: DocTypeExtension
#[derive(Debug, Clone)]
struct DocTypeExtensionVersion {}

impl Element for DocTypeExtensionVersion {
    const ID: u32 = 0x4284;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for DocTypeExtensionVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::Excludes(0));
    const DEFAULT: Option<u64> = None;
}

#[derive(Debug, Clone)]
struct CRC32 {}

impl Element for CRC32 {
    const ID: u32 = 0xBF;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = Some(RangeDef::IsExactly(4));
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for CRC32 {
    const DEFAULT: Option<&'static [u8]> = None;
}

#[derive(Debug, Clone)]
struct Void {}

impl Element for Void {
    const ID: u32 = 0xEC;

    const MIN_OCCURS: Option<usize> = Some(0);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for Void {
    const DEFAULT: Option<&'static [u8]> = None;
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// Explicit Items

#[derive(Debug, Clone)]
struct EBMLReadVersion {}

impl Element for EBMLReadVersion {
    const ID: u32 = 0x42F7;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLReadVersion {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(1));
    const DEFAULT: Option<u64> = Some(1);
}

#[derive(Debug, Clone)]
struct EBMLMaxSizeLength {}

impl Element for EBMLMaxSizeLength {
    const ID: u32 = 0x42F3;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = Some(1);
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UIntElement for EBMLMaxSizeLength {
    const RANGE: Option<RangeDef<u64>> = Some(RangeDef::IsExactly(8));
    const DEFAULT: Option<u64> = Some(8);
}

#[derive(Debug, Clone)]
struct Files(
    usize,
    ElementParsingStage<<Self as MasterElement>::SubElements, <Self as MasterElement>::SubGlobals>,
);

impl Element for Files {
    const ID: u32 = 0x1946696C;

    const MIN_OCCURS: Option<usize> = None;
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for Files {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;

    type SubElements = Files_SubElements;
    type SubGlobals = Files_SubGlobals;
}

#[derive(Debug, Clone)]
enum Files_SubElements {
    File_Variant(File),
}

#[derive(Debug, Clone)]
enum Files_SubGlobals {
    CRC32_Variant(CRC32),
    Void_Variant(Void),
}

#[derive(Debug, Clone)]
struct File(
    usize,
    ElementParsingStage<<Self as MasterElement>::SubElements, <Self as MasterElement>::SubGlobals>,
);

impl Element for File {
    const ID: u32 = 0x6146;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl MasterElement for File {
    const UNKNOWN_SIZE_ALLOWED: Option<bool> = None;
    const RECURSIVE: Option<bool> = None;

    type SubElements = File_SubElements;
    type SubGlobals = EmptyEnum;
}

#[derive(Debug, Clone)]
enum File_SubElements {
    FileName_Variant(FileName),
    MimeType_Variant(MimeType),
    ModificationTimestamp_Variant(ModificationTimestamp),
    Data_Variant(Data),
}

#[derive(Debug, Clone)]
struct FileName {}

impl Element for FileName {
    const ID: u32 = 0x614E;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl UTF8Element for FileName {
    const DEFAULT: Option<&'static str> = None;
}

#[derive(Debug, Clone)]
struct MimeType {}

impl Element for MimeType {
    const ID: u32 = 0x464D;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl StringElement for MimeType {
    const DEFAULT: Option<&'static str> = None;
}

#[derive(Debug, Clone)]
struct ModificationTimestamp {}

impl Element for ModificationTimestamp {
    const ID: u32 = 0x4654;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl DateElement for ModificationTimestamp {
    const RANGE: Option<RangeDef<i64>> = None;
    const DEFAULT: Option<i64> = None;
}

#[derive(Debug, Clone)]
struct Data {}

impl Element for Data {
    const ID: u32 = 0x4664;

    const MIN_OCCURS: Option<usize> = Some(1);
    const MAX_OCCURS: Option<usize> = None;
    const LENGTH: Option<RangeDef<usize>> = None;
    const RECURRING: Option<bool> = None;
    const MIN_VERSION: Option<u64> = None;
    const MAX_VERSION: Option<u64> = None;
}

impl BinaryElement for Data {
    const DEFAULT: Option<&'static [u8]> = None;
}
