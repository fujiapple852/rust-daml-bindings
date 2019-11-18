use daml_lf::protobuf_autogen::daml_lf_1::DottedName;

pub fn leaf_name(dotted_name: &DottedName) -> &str {
    dotted_name.segments.last().unwrap_or_else(|| panic!("DottedName"))
}
