// This file is generated by rust-protobuf 2.22.0-pre. Do not edit
// @generated

// https://github.com/rust-lang/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![allow(unused_attributes)]
#![cfg_attr(rustfmt, rustfmt::skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `google/protobuf/source_context.proto`

#[derive(PartialEq,Clone,Default)]
#[cfg_attr(feature = "with-serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct SourceContext {
    // message fields
    pub file_name: ::std::string::String,
    // special fields
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub unknown_fields: crate::UnknownFields,
    #[cfg_attr(feature = "with-serde", serde(skip))]
    pub cached_size: crate::CachedSize,
}

impl<'a> ::std::default::Default for &'a SourceContext {
    fn default() -> &'a SourceContext {
        <SourceContext as crate::Message>::default_instance()
    }
}

impl SourceContext {
    pub fn new() -> SourceContext {
        ::std::default::Default::default()
    }

    // string file_name = 1;


    pub fn get_file_name(&self) -> &str {
        &self.file_name
    }
    pub fn clear_file_name(&mut self) {
        self.file_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_file_name(&mut self, v: ::std::string::String) {
        self.file_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_file_name(&mut self) -> &mut ::std::string::String {
        &mut self.file_name
    }

    // Take field
    pub fn take_file_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.file_name, ::std::string::String::new())
    }
}

impl crate::Message for SourceContext {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut crate::CodedInputStream<'_>) -> crate::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    crate::rt::read_singular_proto3_string_into(wire_type, is, &mut self.file_name)?;
                },
                _ => {
                    crate::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.file_name.is_empty() {
            my_size += crate::rt::string_size(1, &self.file_name);
        }
        my_size += crate::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut crate::CodedOutputStream<'_>) -> crate::ProtobufResult<()> {
        if !self.file_name.is_empty() {
            os.write_string(1, &self.file_name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &crate::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut crate::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &dyn (::std::any::Any) {
        self as &dyn (::std::any::Any)
    }
    fn as_any_mut(&mut self) -> &mut dyn (::std::any::Any) {
        self as &mut dyn (::std::any::Any)
    }
    fn into_any(self: ::std::boxed::Box<Self>) -> ::std::boxed::Box<dyn (::std::any::Any)> {
        self
    }

    fn descriptor(&self) -> &'static crate::reflect::MessageDescriptor {
        Self::descriptor_static()
    }

    fn new() -> SourceContext {
        SourceContext::new()
    }

    fn descriptor_static() -> &'static crate::reflect::MessageDescriptor {
        static descriptor: crate::rt::LazyV2<crate::reflect::MessageDescriptor> = crate::rt::LazyV2::INIT;
        descriptor.get(|| {
            let mut fields = ::std::vec::Vec::new();
            fields.push(crate::reflect::accessor::make_simple_field_accessor::<_, crate::types::ProtobufTypeString>(
                "file_name",
                |m: &SourceContext| { &m.file_name },
                |m: &mut SourceContext| { &mut m.file_name },
            ));
            crate::reflect::MessageDescriptor::new_pb_name::<SourceContext>(
                "SourceContext",
                fields,
                file_descriptor_proto()
            )
        })
    }

    fn default_instance() -> &'static SourceContext {
        static instance: crate::rt::LazyV2<SourceContext> = crate::rt::LazyV2::INIT;
        instance.get(SourceContext::new)
    }
}

impl crate::Clear for SourceContext {
    fn clear(&mut self) {
        self.file_name.clear();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for SourceContext {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        crate::text_format::fmt(self, f)
    }
}

impl crate::reflect::ProtobufValue for SourceContext {
    fn as_ref(&self) -> crate::reflect::ReflectValueRef {
        crate::reflect::ReflectValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n$google/protobuf/source_context.proto\x12\x0fgoogle.protobuf\",\n\rSou\
    rceContext\x12\x1b\n\tfile_name\x18\x01\x20\x01(\tR\x08fileNameB\x8a\x01\
    \n\x13com.google.protobufB\x12SourceContextProtoP\x01Z6google.golang.org\
    /protobuf/types/known/sourcecontextpb\xa2\x02\x03GPB\xaa\x02\x1eGoogle.P\
    rotobuf.WellKnownTypesJ\xc1\x10\n\x06\x12\x04\x1e\0/\x01\n\xcc\x0c\n\x01\
    \x0c\x12\x03\x1e\0\x122\xc1\x0c\x20Protocol\x20Buffers\x20-\x20Google's\
    \x20data\x20interchange\x20format\n\x20Copyright\x202008\x20Google\x20In\
    c.\x20\x20All\x20rights\x20reserved.\n\x20https://developers.google.com/\
    protocol-buffers/\n\n\x20Redistribution\x20and\x20use\x20in\x20source\
    \x20and\x20binary\x20forms,\x20with\x20or\x20without\n\x20modification,\
    \x20are\x20permitted\x20provided\x20that\x20the\x20following\x20conditio\
    ns\x20are\n\x20met:\n\n\x20\x20\x20\x20\x20*\x20Redistributions\x20of\
    \x20source\x20code\x20must\x20retain\x20the\x20above\x20copyright\n\x20n\
    otice,\x20this\x20list\x20of\x20conditions\x20and\x20the\x20following\
    \x20disclaimer.\n\x20\x20\x20\x20\x20*\x20Redistributions\x20in\x20binar\
    y\x20form\x20must\x20reproduce\x20the\x20above\n\x20copyright\x20notice,\
    \x20this\x20list\x20of\x20conditions\x20and\x20the\x20following\x20discl\
    aimer\n\x20in\x20the\x20documentation\x20and/or\x20other\x20materials\
    \x20provided\x20with\x20the\n\x20distribution.\n\x20\x20\x20\x20\x20*\
    \x20Neither\x20the\x20name\x20of\x20Google\x20Inc.\x20nor\x20the\x20name\
    s\x20of\x20its\n\x20contributors\x20may\x20be\x20used\x20to\x20endorse\
    \x20or\x20promote\x20products\x20derived\x20from\n\x20this\x20software\
    \x20without\x20specific\x20prior\x20written\x20permission.\n\n\x20THIS\
    \x20SOFTWARE\x20IS\x20PROVIDED\x20BY\x20THE\x20COPYRIGHT\x20HOLDERS\x20A\
    ND\x20CONTRIBUTORS\n\x20\"AS\x20IS\"\x20AND\x20ANY\x20EXPRESS\x20OR\x20I\
    MPLIED\x20WARRANTIES,\x20INCLUDING,\x20BUT\x20NOT\n\x20LIMITED\x20TO,\
    \x20THE\x20IMPLIED\x20WARRANTIES\x20OF\x20MERCHANTABILITY\x20AND\x20FITN\
    ESS\x20FOR\n\x20A\x20PARTICULAR\x20PURPOSE\x20ARE\x20DISCLAIMED.\x20IN\
    \x20NO\x20EVENT\x20SHALL\x20THE\x20COPYRIGHT\n\x20OWNER\x20OR\x20CONTRIB\
    UTORS\x20BE\x20LIABLE\x20FOR\x20ANY\x20DIRECT,\x20INDIRECT,\x20INCIDENTA\
    L,\n\x20SPECIAL,\x20EXEMPLARY,\x20OR\x20CONSEQUENTIAL\x20DAMAGES\x20(INC\
    LUDING,\x20BUT\x20NOT\n\x20LIMITED\x20TO,\x20PROCUREMENT\x20OF\x20SUBSTI\
    TUTE\x20GOODS\x20OR\x20SERVICES;\x20LOSS\x20OF\x20USE,\n\x20DATA,\x20OR\
    \x20PROFITS;\x20OR\x20BUSINESS\x20INTERRUPTION)\x20HOWEVER\x20CAUSED\x20\
    AND\x20ON\x20ANY\n\x20THEORY\x20OF\x20LIABILITY,\x20WHETHER\x20IN\x20CON\
    TRACT,\x20STRICT\x20LIABILITY,\x20OR\x20TORT\n\x20(INCLUDING\x20NEGLIGEN\
    CE\x20OR\x20OTHERWISE)\x20ARISING\x20IN\x20ANY\x20WAY\x20OUT\x20OF\x20TH\
    E\x20USE\n\x20OF\x20THIS\x20SOFTWARE,\x20EVEN\x20IF\x20ADVISED\x20OF\x20\
    THE\x20POSSIBILITY\x20OF\x20SUCH\x20DAMAGE.\n\n\x08\n\x01\x02\x12\x03\
    \x20\0\x18\n\x08\n\x01\x08\x12\x03\"\0;\n\t\n\x02\x08%\x12\x03\"\0;\n\
    \x08\n\x01\x08\x12\x03#\0,\n\t\n\x02\x08\x01\x12\x03#\0,\n\x08\n\x01\x08\
    \x12\x03$\03\n\t\n\x02\x08\x08\x12\x03$\03\n\x08\n\x01\x08\x12\x03%\0\"\
    \n\t\n\x02\x08\n\x12\x03%\0\"\n\x08\n\x01\x08\x12\x03&\0!\n\t\n\x02\x08$\
    \x12\x03&\0!\n\x08\n\x01\x08\x12\x03'\0M\n\t\n\x02\x08\x0b\x12\x03'\0M\n\
    \x83\x01\n\x02\x04\0\x12\x04+\0/\x01\x1aw\x20`SourceContext`\x20represen\
    ts\x20information\x20about\x20the\x20source\x20of\x20a\n\x20protobuf\x20\
    element,\x20like\x20the\x20file\x20in\x20which\x20it\x20is\x20defined.\n\
    \n\n\n\x03\x04\0\x01\x12\x03+\x08\x15\n\xa3\x01\n\x04\x04\0\x02\0\x12\
    \x03.\x02\x17\x1a\x95\x01\x20The\x20path-qualified\x20name\x20of\x20the\
    \x20.proto\x20file\x20that\x20contained\x20the\x20associated\n\x20protob\
    uf\x20element.\x20\x20For\x20example:\x20`\"google/protobuf/source_conte\
    xt.proto\"`.\n\n\x0c\n\x05\x04\0\x02\0\x05\x12\x03.\x02\x08\n\x0c\n\x05\
    \x04\0\x02\0\x01\x12\x03.\t\x12\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03.\x15\
    \x16b\x06proto3\
";

static file_descriptor_proto_lazy: crate::rt::LazyV2<crate::descriptor::FileDescriptorProto> = crate::rt::LazyV2::INIT;

fn parse_descriptor_proto() -> crate::descriptor::FileDescriptorProto {
    crate::Message::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static crate::descriptor::FileDescriptorProto {
    file_descriptor_proto_lazy.get(|| {
        parse_descriptor_proto()
    })
}
