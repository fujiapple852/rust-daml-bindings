use crate::error::{DamlJsonReqConError, DamlJsonReqConResult};
use crate::request::{
    DamlJsonCreateAndExerciseRequest, DamlJsonCreateRequest, DamlJsonExerciseByKeyRequest, DamlJsonExerciseRequest,
};
use crate::util::fst;
use crate::value_decode::JsonValueDecoder;
use daml_grpc::data::command::{
    DamlCreateAndExerciseCommand, DamlCreateCommand, DamlExerciseByKeyCommand, DamlExerciseCommand,
};
use daml_grpc::data::value::{DamlRecord, DamlValue};
use daml_grpc::data::DamlIdentifier;
use daml_lf::element::{DamlArchive, DamlData, DamlTemplate, DamlType};
use itertools::Itertools;
use serde_json::Value;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};

/// Convert a JSON API request to a GRPC API command.
#[derive(Debug)]
pub struct JsonToGrpcRequestConverter<'a> {
    arc: &'a DamlArchive<'a>,
    decoder: JsonValueDecoder<'a>,
}

impl<'a> JsonToGrpcRequestConverter<'a> {
    pub const fn new(arc: &'a DamlArchive<'a>) -> Self {
        Self {
            arc,
            decoder: JsonValueDecoder::new(arc),
        }
    }

    /// Convert a JSON API [`DamlJsonCreateRequest`] to a GRPC [`DamlCreateCommand`].
    pub fn convert_create_request(&self, create: &DamlJsonCreateRequest) -> DamlJsonReqConResult<DamlCreateCommand> {
        let template_id = DamlJsonTemplateId::try_from(create.template_id.as_str())?;
        let package_id = self.resolve_template(&template_id).map(fst)?;
        let grpc_identifier = make_grpc_identifier(package_id, &template_id.module, &template_id.entity);
        let decoded_value = self.decode_data(package_id, &template_id.module, &template_id.entity, &create.payload)?;
        let grpc_create_arguments = DamlRecord::try_from(decoded_value)?;
        Ok(DamlCreateCommand::new(grpc_identifier, grpc_create_arguments))
    }

    /// Convert a JSON API [`DamlJsonExerciseRequest`] to a GRPC [`DamlExerciseCommand`].
    pub fn convert_exercise_request(
        &self,
        exercise: &DamlJsonExerciseRequest,
    ) -> DamlJsonReqConResult<DamlExerciseCommand> {
        let template_id = DamlJsonTemplateId::try_from(exercise.template_id.as_str())?;
        let package_id = self.resolve_template(&template_id).map(fst)?;
        let grpc_identifier = make_grpc_identifier(package_id, &template_id.module, &template_id.entity);
        let choice_args = self.decode_data(package_id, &template_id.module, &exercise.choice, &exercise.argument)?;
        Ok(DamlExerciseCommand::new(grpc_identifier, &exercise.contract_id, &exercise.choice, choice_args))
    }

    /// Convert a JSON API [`DamlJsonExerciseByKeyRequest`] to a GRPC [`DamlExerciseByKeyCommand`].
    pub fn convert_exercise_by_key_request(
        &self,
        exercise: &DamlJsonExerciseByKeyRequest,
    ) -> DamlJsonReqConResult<DamlExerciseByKeyCommand> {
        let template_id = DamlJsonTemplateId::try_from(exercise.template_id.as_str())?;
        let (package_id, template) = self.resolve_template(&template_id)?;
        let grpc_identifier = make_grpc_identifier(package_id, &template_id.module, &template_id.entity);
        let choice_key = self.decode_template_key(template, &exercise.key)?;
        let choice_args = self.decode_data(package_id, &template_id.module, &exercise.choice, &exercise.argument)?;
        Ok(DamlExerciseByKeyCommand::new(grpc_identifier, choice_key, &exercise.choice, choice_args))
    }

    /// Convert a JSON API [`DamlJsonCreateAndExerciseRequest`] to a GRPC [`DamlCreateAndExerciseCommand`].
    pub fn convert_create_and_exercise_request(
        &self,
        create_and_exercise: &DamlJsonCreateAndExerciseRequest,
    ) -> DamlJsonReqConResult<DamlCreateAndExerciseCommand> {
        let template_id = DamlJsonTemplateId::try_from(create_and_exercise.template_id.as_str())?;
        let package_id = self.resolve_template(&template_id).map(fst)?;
        let grpc_identifier = make_grpc_identifier(package_id, &template_id.module, &template_id.entity);
        let decoded_value =
            self.decode_data(package_id, &template_id.module, &template_id.entity, &create_and_exercise.payload)?;
        let grpc_create_arguments = DamlRecord::try_from(decoded_value)?;
        let choice_args = self.decode_data(
            package_id,
            &template_id.module,
            &create_and_exercise.choice,
            &create_and_exercise.argument,
        )?;
        Ok(DamlCreateAndExerciseCommand::new(
            grpc_identifier,
            grpc_create_arguments,
            &create_and_exercise.choice,
            choice_args,
        ))
    }

    /// Attempt to resolve a `DamlJsonTemplateId` to a `DamlTemplate` and containing package id.
    ///
    /// If the given `DamlJsonTemplateId` contains a package id then this is used to locate the required
    /// `DamlTemplate`.
    ///
    /// If the given `DamlJsonTemplateId` does not contain a package id then a search of all packages is performed for
    /// unique template which matches the required module path and entity name.
    ///
    /// If no unique `DamlTemplate` is found then an error is returned.
    fn resolve_template(
        &'a self,
        template_id: &'a DamlJsonTemplateId,
    ) -> DamlJsonReqConResult<(&'a str, &'a DamlTemplate<'_>)> {
        let (pid, data) = match template_id.package_id.as_deref() {
            Some(package_id) => self
                .arc
                .data(package_id, &template_id.module, &template_id.entity)
                .map(|data| (package_id, data))
                .ok_or_else(|| DamlJsonReqConError::UnknownTemplateId(template_id.to_string()))?,
            None => self.find_data_and_package(&template_id.module, &template_id.entity)?,
        };
        if let DamlData::Template(template) = data {
            Ok((pid, template))
        } else {
            Err(DamlJsonReqConError::ExpectedTemplateError(data.name().to_owned()))
        }
    }

    /// Attempt to find a given template in all packages and return the found template and containing package id.
    ///
    /// Returns an error if no such template is found or if the template exists in multiple packages.
    fn find_data_and_package<S>(
        &self,
        module_path: &[S],
        template_entity: &str,
    ) -> DamlJsonReqConResult<(&str, &DamlData<'a>)>
    where
        S: AsRef<str> + Display,
    {
        let mut matcher = self
            .arc
            .packages()
            .filter_map(|package| {
                package.root_module().child_module_path(module_path).map(|module| (package.package_id(), module))
            })
            .flat_map(|(package, m)| m.data_types().map(move |data| (package, data)))
            .filter_map(|(package, data)| (data.name() == template_entity).then(|| (package, data)));
        match (matcher.next(), matcher.next()) {
            (None, _) => Err(DamlJsonReqConError::UnknownTemplateId(template_entity.to_string())),
            (Some((p, d)), None) => Ok((p, d)),
            (Some((p1, _)), Some((p2, _))) => Err(DamlJsonReqConError::MultipleMatchingTemplates(
                format!("{}:{}", module_path.iter().join("."), template_entity),
                vec![String::from(p1), String::from(p2)],
            )),
        }
    }

    /// Decode a JSON encoded Daml `data` type.
    fn decode_data(
        &self,
        package_id: &str,
        module: &[String],
        entity: &str,
        json_value: &Value,
    ) -> DamlJsonReqConResult<DamlValue> {
        Ok(self.decoder.decode(json_value, &DamlType::make_tycon(package_id, module, entity))?)
    }

    /// Decode a JSON encoded Daml template key.
    fn decode_template_key(
        &self,
        template: &DamlTemplate<'_>,
        json_key_value: &Value,
    ) -> DamlJsonReqConResult<DamlValue> {
        let key_ty =
            template.key().ok_or_else(|| DamlJsonReqConError::TemplateNoKeyError(template.name().to_string()))?.ty();
        Ok(self.decoder.decode(json_key_value, key_ty)?)
    }
}

fn make_grpc_identifier(package_id: &str, module: &[String], entity: &str) -> DamlIdentifier {
    DamlIdentifier::new(package_id, module.iter().join("."), entity)
}

/// A helper representation of a DAML template id.
///
/// This type exists to provide a convenient `TryFrom` impl for converting from a String.
///
/// Template ids are represented as JSON strings of the form `[package_id:]module:entity` in the DAML JSON API and as
/// such this struct does is not required to be `Serialize` or `Deserialize`.
#[derive(Debug)]
pub struct DamlJsonTemplateId {
    pub package_id: Option<String>,
    pub module: Vec<String>,
    pub entity: String,
}

impl DamlJsonTemplateId {
    pub fn new(package_id: Option<String>, module: Vec<String>, entity: String) -> Self {
        Self {
            package_id,
            module,
            entity,
        }
    }
}

impl TryFrom<&str> for DamlJsonTemplateId {
    type Error = DamlJsonReqConError;

    fn try_from(value: &str) -> DamlJsonReqConResult<Self> {
        let splits: Vec<_> = value.split(':').collect();
        match *splits.as_slice() {
            [module, entity] =>
                Ok(Self::new(None, module.split('.').map(ToOwned::to_owned).collect(), entity.to_owned())),
            [package_id, module, entity] => Ok(Self::new(
                Some(package_id.to_owned()),
                module.split('.').map(ToOwned::to_owned).collect(),
                entity.to_owned(),
            )),
            _ => Err(DamlJsonReqConError::TemplateIdFormatError(value.to_owned())),
        }
    }
}

impl Display for DamlJsonTemplateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.package_id {
            Some(package_id) => write!(f, "{}:{}:{}", package_id, self.module.join("."), self.entity),
            None => write!(f, "{}:{}", self.module.join("."), self.entity),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DamlJsonCodecError;
    use anyhow::Result;
    use daml::macros::daml_path;
    use daml_grpc::primitive_types::{DamlParty, DamlText};
    use daml_lf::DarFile;
    use serde_json::json;

    static TESTING_TYPES_DAR_PATH: &str = "../resources/testing_types_sandbox/TestingTypes-latest.dar";

    #[test]
    fn test_convert_create_request() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new(
            "DA.RentDemo:RentalAgreement",
            json!(
                {
                     "landlord": "Alice",
                     "tenant": "Bob",
                     "terms": "test terms",
                }
            ),
        );
        let command = request_converter.convert_create_request(&request)?;
        assert_eq!("RentalAgreement", command.template_id().entity_name());
        assert_eq!("DA.RentDemo", command.template_id().module_name());
        assert_eq!(&DamlParty::from("Alice"), command.create_arguments().extract(daml_path![landlord#p])?);
        assert_eq!(&DamlParty::from("Bob"), command.create_arguments().extract(daml_path![tenant#p])?);
        assert_eq!(&DamlText::from("test terms"), command.create_arguments().extract(daml_path![terms#t])?);
        Ok(())
    }

    #[test]
    fn test_convert_exercise_request() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonExerciseRequest::new(
            "DA.RentDemo:RentalAgreement",
            "#0:0",
            "Accept",
            json!(
                {
                     "foo": "this is foo",
                     "bar": 100,
                }
            ),
        );
        let command = request_converter.convert_exercise_request(&request)?;
        assert_eq!("RentalAgreement", command.template_id().entity_name());
        assert_eq!("DA.RentDemo", command.template_id().module_name());
        assert_eq!("#0:0", command.contract_id());
        assert_eq!("Accept", command.choice());
        assert_eq!(&DamlText::from("this is foo"), command.choice_argument().extract(daml_path![foo#t])?);
        assert_eq!(&100, command.choice_argument().extract(daml_path![bar#i])?);
        Ok(())
    }

    #[test]
    fn test_convert_exercise_by_key_request() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonExerciseByKeyRequest::new(
            "DA.PingPong:Ping",
            json!({ "sender" : "Alice", "count": 99 }),
            "FromUserData",
            json!(
                {
                     "new_count": 5,
                     "new_data": { "name": "Bob", "new_value": 55 },
                }
            ),
        );
        let command = request_converter.convert_exercise_by_key_request(&request)?;
        assert_eq!("Ping", command.template_id().entity_name());
        assert_eq!("DA.PingPong", command.template_id().module_name());
        assert_eq!(&DamlParty::from("Alice"), command.contract_key().extract(daml_path![sender#p])?);
        assert_eq!(&99, command.contract_key().extract(daml_path![count#i])?);
        assert_eq!("FromUserData", command.choice());
        assert_eq!(&5, command.choice_argument().extract(daml_path![new_count#i])?);
        assert_eq!(&DamlParty::from("Bob"), command.choice_argument().extract(daml_path![new_data/name#p])?);
        assert_eq!(&55, command.choice_argument().extract(daml_path![new_data/new_value#i])?);
        Ok(())
    }

    #[test]
    fn test_convert_create_and_exercise_request() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateAndExerciseRequest::new(
            "DA.RentDemo:RentalAgreement",
            json!(
            {
                 "landlord": "Alice",
                 "tenant": "Bob",
                 "terms": "test terms",
            }),
            "Accept",
            json!(
                {
                     "foo": "this is foo",
                     "bar": 100,
                }
            ),
        );
        let command = request_converter.convert_create_and_exercise_request(&request)?;
        assert_eq!("RentalAgreement", command.template_id().entity_name());
        assert_eq!("DA.RentDemo", command.template_id().module_name());
        assert_eq!(&DamlParty::from("Alice"), command.create_arguments().extract(daml_path![landlord#p])?);
        assert_eq!(&DamlParty::from("Bob"), command.create_arguments().extract(daml_path![tenant#p])?);
        assert_eq!(&DamlText::from("test terms"), command.create_arguments().extract(daml_path![terms#t])?);
        assert_eq!("Accept", command.choice());
        assert_eq!(&DamlText::from("this is foo"), command.choice_argument().extract(daml_path![foo#t])?);
        assert_eq!(&100, command.choice_argument().extract(daml_path![bar#i])?);
        Ok(())
    }

    #[test]
    fn test_convert_create_request_with_package_id() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new(
            format!("{}:DA.RentDemo:RentalAgreement", arc.main_package_id()),
            json!(
                {
                     "landlord": "Alice",
                     "tenant": "Bob",
                     "terms": "test terms",
                }
            ),
        );
        let command = request_converter.convert_create_request(&request)?;
        assert_eq!("RentalAgreement", command.template_id().entity_name());
        assert_eq!("DA.RentDemo", command.template_id().module_name());
        assert_eq!(&DamlParty::from("Alice"), command.create_arguments().extract(daml_path![landlord#p])?);
        assert_eq!(&DamlParty::from("Bob"), command.create_arguments().extract(daml_path![tenant#p])?);
        assert_eq!(&DamlText::from("test terms"), command.create_arguments().extract(daml_path![terms#t])?);
        Ok(())
    }

    #[test]
    fn test_convert_create_request_unknown_template_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new("DA.RentDemo:FooTemplate", json!({}));
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::UnknownTemplateId(_)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_create_request_unknown_module_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new("Foo.RentDemo:RentalAgreement", json!({}));
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::UnknownTemplateId(_)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_create_request_unknown_package_id_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new("1234:DA.RentDemo:RentalAgreement", json!({}));
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::UnknownTemplateId(_)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_create_request_expected_template_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new("DA.PingPong:UserData", json!({}));
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::ExpectedTemplateError(_)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    // TODO - how to test MultipleMatchingTemplates (need a template in two packages)
    #[test]
    #[ignore]
    fn test_convert_create_request_multiple_match_templates_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new("DA.PingPong:UserData", json!({}));
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::MultipleMatchingTemplates(..)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_exercise_request_unknown_choice_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonExerciseRequest::new("DA.RentDemo:RentalAgreement", "#0:0", "UnknownChoice", json!({}));
        match request_converter.convert_exercise_request(&request) {
            Err(DamlJsonReqConError::CodecError(DamlJsonCodecError::DataNotFound(_))) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_exercise_by_key_request_template_no_key_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonExerciseByKeyRequest::new("DA.RentDemo:RentalAgreement", json!({}), "Dummy", json!({}));
        match request_converter.convert_exercise_by_key_request(&request) {
            Err(DamlJsonReqConError::TemplateNoKeyError(_)) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_create_request_missing_field_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateRequest::new(
            "DA.RentDemo:RentalAgreement",
            json!(
                {
                     "landlord": "Alice",
                     "tenant": "Bob",
                }
            ),
        );
        match request_converter.convert_create_request(&request) {
            Err(DamlJsonReqConError::CodecError(DamlJsonCodecError::MissingJsonRecordObjectField(_))) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }

    #[test]
    fn test_convert_create_and_exercise_request_missing_choice_field_err() -> Result<()> {
        let arc = DarFile::from_file(TESTING_TYPES_DAR_PATH)?.to_owned_archive()?;
        let request_converter = JsonToGrpcRequestConverter::new(&arc);
        let request = DamlJsonCreateAndExerciseRequest::new(
            "DA.RentDemo:RentalAgreement",
            json!(
            {
                 "landlord": "Alice",
                 "tenant": "Bob",
                 "terms": "test terms",
            }),
            "Accept",
            json!(
                {
                     "foo": "this is foo",
                }
            ),
        );
        match request_converter.convert_create_and_exercise_request(&request) {
            Err(DamlJsonReqConError::CodecError(DamlJsonCodecError::MissingJsonRecordObjectField(_))) => Ok(()),
            Err(e) => panic!("{}", e.to_string()),
            _ => panic!("test should fail"),
        }
    }
}
