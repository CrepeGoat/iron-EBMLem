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







