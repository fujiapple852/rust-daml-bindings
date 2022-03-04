//! Procedural macros for generating Rust types and conversions from Daml types and Archives.
//!
//! # Overview
//!
//! Two mechanisms are provided for representing Daml types in Rust:
//! * Custom attributes which can be applied to Rust structures which generate Daml type converters.
//! * A procedural macro code generator which takes a Daml `dar` file as input and generates Rust types annotated with
//! the custom attributes.
//!
//! # Custom Attributes
//!
//! This section explains how to use the provided custom attributes to annotate Rust types to generate the Daml ledger
//! API data conversion code required to be able to use them with a Daml ledger.
//!
//! ### Mapping Daml Structures to Rust
//!
//! Daml structures are modelled using various Rust language constructs in conjunction with custom attributes
//! procedural macros as shown in the following table:
//!
//! | Daml Concept            | Rust Construct | Custom Attribute       |
//! |-------------------------|----------------|------------------------|
//! | [Daml Template]         | `struct`       | [`macro@DamlTemplate`] |
//! | [Daml Template Choices] | `impl` block   | [`macro@DamlChoices`]  |
//! | [Daml Data (Record)]    | `struct`       | [`macro@DamlData`]     |
//! | [Daml Data (Variant)]   | `enum`         | [`macro@DamlVariant`]  |
//! | [Daml Enum]             | `enum`         | [`macro@DamlEnum`]     |
//!
//! ### Mapping Daml Data Types to Rust
//!
//! The following table lists the mappings between
//! [Daml build-in primitive types](https://docs.daml.com/daml/reference/data-types.html#built-in-types) and Rust type
//! aliases:
//!
//! | Daml Type         | Rust Type Alias     | Concrete Rust Type       | Notes                                      |
//! |-------------------|---------------------|--------------------------|--------------------------------------------|
//! | `Int`             | [`DamlInt64`]       | `i64`                    |                                            |
//! | `Numeric`         | [`DamlNumeric`]     | `bigdecimal::BigDecimal` | **Note: BigDecimal crate to be replaced**  |
//! | `Text`            | [`DamlText`]        | `String`                 |                                            |
//! | `Bool`            | [`DamlBool`]        | `bool`                   |                                            |
//! | `Party`           | [`DamlParty`]       | `String`                 |                                            |
//! | `Date`            | [`DamlDate`]        | `chrono::Date`           |                                            |
//! | `Time`            | [`DamlTime`]        | `chrono::DateTime`       |                                            |
//! | `()`              | [`DamlUnit`]        | `()`                     |                                            |
//! | `ContractId a`    | [`DamlContractId`]  | `String`                 | **Note: this mapping is likely to change** |
//! | `List a` or `[a]` | [`DamlList<T>`]     | `Vec<T>`                 | type `T` must be another Rust type alias   |
//! | `TextMap a`       | [`DamlTextMap<T>`]  | `HashMap<String, T>`     | type `T` must be another Rust type alias   |
//! | `Optional a`      | [`DamlOptional<T>`] | `Option<T>`              | type `T` must be another Rust type alias   |
//!
//!
//! Note that the concrete Rust types are shown here as a convenience only, in all cases the Rust type alias _must_ be
//! used when representing Daml constructs so that the Daml types can be determined.
//!
//! ### Parameterized Types
//!
//! The parameterized types (`List<T>`, `TextMap<T>` and `Optional<T>`) may be freely nested to an arbitrary depth and
//! may be used in all context a type is expected such as templates & data fields as well as choice parameters.
//!
//! For example these are examples of valid types:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # pub struct MyData {}
//! let int: DamlInt64;
//! let party: DamlParty;
//! let opt_numeric: DamlOptional<DamlNumeric10>;
//! let list_of_int: DamlList<DamlInt64>;
//! let list_of_opt_int: DamlList<DamlOptional<DamlInt64>>;
//! let list_of_opt_map_party: DamlList<DamlOptional<DamlTextMap<DamlParty>>>;
//! let opt_list_data: DamlOptional<DamlList<MyData>>;
//! ```
//!
//! ### Recursive Data Types
//!
//! Daml Data (both Records and Variants) may be recursive.  For example:
//!
//! ```daml
//! data Foo = Foo
//!   with
//!     bar : Optional Text
//!     foo : Foo
//! ```
//!
//! Both [`macro@DamlData`] and [`macro@DamlVariant`] types may therefore be defined recursively.  However modelling
//! such structures in Rust requires that any recurisvely defined items be held via an indirection, typically
//! via a heap allocation smart pointer such as `Box<T>`, to ensure a non-infinite size for the `struct` or `enum`
//! (see [here](https://doc.rust-lang.org/error-index.html#E0072) for details).
//!
//! The above example can therefore be represented as follows:
//!
//! ```no_run
//! # use daml::prelude::*;
//! #[DamlData]
//! pub struct Foo {
//!     bar: DamlOptional<DamlText>,
//!     foo: Box<Foo>,
//! }
//! ```
//!
//! Note that `Box<T>` is the only form of indirection currently supported and it may be used anywhere `T` is used.
//!
//! ### Prelude
//!
//! All of the above Rust type aliases are defined in the [`prelude`](https://docs.rs/daml/0.2.0/daml/prelude/index.html) module of
//! the [`daml`](../daml/index.html) crate and can included by using `daml::prelude::*`.
//!
//! ### Modules
//!
//! Rust `struct` and `enum` types annotated with the custom attributes provided by this crate are
//! _not_ required to be nested in Rust `modules` that mirror the Daml `module` heirarchy.  All of the standard Rust
//! name resolution and visiblity rules apply and therefore it is recommedned to mirror the Daml heirarchy where
//! possible to avoid namespace collisions.
//!
//! For example, the `MyData` data type defined in the `Fuji.MyModule.MySubModule` Daml module would likely be declared
//! as follows:
//!
//! ```no_run
//! mod fuji {
//!     mod my_module {
//!         mod my_sub_module {
//!             use daml::prelude::*;
//!             #[DamlData]
//!             pub struct MyData {}
//!         }
//!     }
//! }
//! ```
//!
//! ### Example
//!
//! Given the following Daml template declared in the `Fuji.PingPong` module of a given package:
//!
//! ```daml
//! template Ping
//!   with
//!     sender: Party
//!     receiver: Party
//!     count: Int
//!   where
//!     signatory sender
//!     observer receiver
//!
//!     controller receiver can
//!       ResetCount : ()
//!         with
//!           new_count: Int
//!         do
//!           create Pong with sender; receiver; count = new_count
//!           return ()
//! ```
//!
//! This can be represented in Rust by using the [`macro@DamlTemplate`] and [`macro@DamlChoices`] custom attributes:
//!
//! ```no_run
//! use daml::prelude::*;
//!
//! #[DamlTemplate(package_id = r"...package id hash omitted...", module_name = "Fuji.PingPong")]
//! pub struct Ping {
//!     pub sender: DamlParty,
//!     pub receiver: DamlParty,
//!     pub count: DamlInt64,
//! }
//!
//! #[DamlChoices]
//! impl Ping {
//!     #[ResetCount]
//!     fn reset_count(&self, new_count: DamlInt64) {}
//! }
//! ```
//!
//! A new `Ping` can then be created as follows:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # #[DamlTemplate(package_id = r"", module_name = "Fuji.PingPong")]
//! # pub struct Ping {
//! #    pub sender: DamlParty,
//! #    pub receiver: DamlParty,
//! #    pub count: DamlInt64,
//! # }
//! let ping = Ping::new("Alice", "Bob", 0);
//! ```
//!
//! To create an instance of the `Ping` template on a Daml ledger a [`DamlCreateCommand`] specfic to our `ping` data
//! needs to be constructed.  This can be done as follows:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # #[DamlTemplate(package_id = r"", module_name = "Fuji.PingPong")]
//! # pub struct Ping {
//! #    pub sender: DamlParty,
//! #    pub receiver: DamlParty,
//! #    pub count: DamlInt64,
//! # }
//! # let ping = Ping::new("Alice", "Bob", 0);
//! let create_ping_command = ping.create_command();
//! ```
//!
//! The generated [`DamlCreateCommand`] can then be submitted to the Daml ledger via the [`DamlCommandService`] or
//! [`DamlCommandSubmissionService`] as usual.
//!
//! Once the contract instance has been created on the Daml ledger and the corresponding [`DamlCreatedEvent`] has been
//! received then it can be converted into a Rust type as follows:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # use std::convert::TryInto;
//! # #[DamlTemplate(package_id = r"", module_name = "Fuji.PingPong")]
//! # pub struct Ping {
//! #    pub sender: DamlParty,
//! #    pub receiver: DamlParty,
//! #    pub count: DamlInt64,
//! # }
//! # fn main() -> DamlResult<()> {
//! # let created_event = DamlCreatedEvent::new("", "", DamlIdentifier::new("", "", ""), None, DamlRecord::new(vec![], None::<DamlIdentifier>), vec![], vec![], vec![], "");
//! let ping_contract: PingContract = created_event.try_into()?;
//! # Ok::<(), DamlError>(())
//! }
//! ```
//!
//! Note that the [`DamlCreatedEvent`] returned by the Daml ledger is converted into a `PingContract` rather than a
//! plain `Ping`.  The `PingContract` type is a `struct` and provides methods `data() -> Ping` and
//! `id() -> &PingContractId` to access the `Ping` data and contract id respectively:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # use std::convert::TryInto;
//! # #[DamlTemplate(package_id = r"", module_name = "Fuji.PingPong")]
//! # pub struct Ping {
//! #    pub sender: DamlParty,
//! #    pub receiver: DamlParty,
//! #    pub count: DamlInt64,
//! # }
//! # fn main() -> DamlResult<()> {
//! # let created_event = DamlCreatedEvent::new("", "", DamlIdentifier::new("", "", ""), None, DamlRecord::new(vec![], None::<DamlIdentifier>), vec![], vec![], vec![], "");
//! # let ping_contract: PingContract = created_event.try_into()?;
//! assert_eq!("Alice", ping_contract.data().sender);
//! assert_eq!("Bob", ping_contract.data().receiver);
//! assert_eq!(0, ping_contract.data().count);
//! assert_eq!("#0:0", ping_contract.id().contract_id);
//! # Ok::<(), DamlError>(())
//! # }
//! ```
//! > **_NOTE:_**  The contract id may be refactored to use a separate type in future.
//!
//! The `PingContract` types provides a method for each `choice` defined by the Daml `template` along with any
//! parameters that choice may have.  To exercise a choice on a Daml ledger a [`DamlExerciseCommand`] specific to our
//! contract is needed.  The can be constructed as follows:
//!
//! ```no_run
//! # use daml::prelude::*;
//! # use std::convert::TryInto;
//! # #[DamlTemplate(package_id = r"", module_name = "Fuji.PingPong")]
//! # pub struct Ping {
//! #    pub sender: DamlParty,
//! #    pub receiver: DamlParty,
//! #    pub count: DamlInt64,
//! # }
//! # #[DamlChoices]
//! # impl Ping {
//! #     #[ResetCount]
//! #     fn reset_count(&self, new_count: DamlInt64) {}
//! # }
//! # fn main() -> DamlResult<()> {
//! # let created_event = DamlCreatedEvent::new("", "", DamlIdentifier::new("", "", ""), None, DamlRecord::new(vec![], None::<DamlIdentifier>), vec![], vec![], vec![], "");
//! # let ping_contract: PingContract = created_event.try_into()?;
//! let exercise_command = ping_contract.id().reset_count_command(5);
//! # Ok::<(), DamlError>(())
//! }
//! ```
//!
//! The generated [`DamlExerciseCommand`] can then be submitted to the Daml ledger via the [`DamlCommandService`] or
//! [`DamlCommandSubmissionService`] as usual.
//!
//! Note that the name of the choice method _must_ match the name of the Daml choice (in snake_case) with a `_command`
//! suffix and the choice parameters _must_ match between the Daml and Rust representations.
//!
//! See the documentation for [`macro@DamlTemplate`], [`macro@DamlChoices`] & [`macro@DamlData`] for full details and
//! examples.
//!
//! ### Errors
//!
//! Returns the underlying [`DamlError`] (runtime-only) if the `try_into()` conversion from a [`DamlValue`] to an
//! annotated type fails.
//!
//! ### Panics
//!
//! Panics (compile-time only) if errors are detected in the annotated `struct`, `enum` or `impl` blocks.
//!
//! # Code Generator
//!
//! Thsi section describes how to use use the procedural macro to generating Rust types from Daml `dar` ("Daml Archive")
//! files.
//!
//! ### Daml & Rust Modules
//!
//! ### Example
//!
//! Given the following Daml code in module `MyOrg.MyModule` of `MyDamlApplication` compiled to
//! `resources/MyDamlApplication.dar`:
//!
//! ```daml
//! template Ping
//!   with
//!     sender: Party
//!     receiver: Party
//!     count: Int
//!   where
//!     signatory sender
//!     observer receiver
//!
//!     controller receiver can
//!       ResetCount : ()
//!         with
//!           new_count: Int
//!         do
//!           create Pong with sender; receiver; count = new_count
//!           return ()
//! ```
//!
//! The Rust types and methods required to represent this template can be generated by using `daml_codegen` as
//! follows:
//!
//! ```ignore
//! daml_codegen!(dar_file = r"MyDamlApplication.dar");
//! ```
//!
//! This produces the following Rust code:
//!
//! ```no_run
//! pub mod my_daml_application {
//!     pub mod my_org {
//!         pub mod my_module {
//!             use daml::prelude::*;
//!             #[DamlTemplate(package_id = r"...", module_name = "MyOrg.MyModule")]
//!             pub struct Ping {
//!                 pub sender: DamlParty,
//!                 pub receiver: DamlParty,
//!                 pub count: DamlInt64,
//!             }
//!
//!             #[DamlChoices]
//!             impl Ping {
//!                 #[ResetCount]
//!                 fn reset_count(&self, new_count: DamlInt64) {}
//!             }
//!         }
//!     }
//! }
//! ```
//! See the above for details generated custom attributes such as [`macro@DamlTemplate`] and
//! [`macro@DamlChoices`].
//!
//! ### Panics
//!
//! Panics (compile-time only) if errors are detected during code generation.
//!
//! [`DamlInt64`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlInt64.html
//! [`DamlNumeric`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlNumeric.html
//! [`DamlText`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlText.html
//! [`DamlBool`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlBool.html
//! [`DamlParty`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlParty.html
//! [`DamlDate`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlDate.html
//! [`DamlTime`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlTime.html
//! [`DamlUnit`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlUnit.html
//! [`DamlContractId`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlContractId.html
//! [`DamlList<T>`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlList.html
//! [`DamlTextMap<T>`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlTextMap.html
//! [`DamlOptional<T>`]: https://docs.rs/daml/0.2.0/daml/prelude/type.DamlOptional.html
//! [`DamlCreateCommand`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/data/command/struct.DamlCreateCommand.html
//! [`DamlExerciseCommand`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/data/command/struct.DamlExerciseCommand.html
//! [`DamlCommandService`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/service/struct.DamlCommandService.html
//! [`DamlCommandSubmissionService`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/service/struct.DamlCommandSubmissionService.html
//! [`DamlCreatedEvent`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/data/event/struct.DamlCreatedEvent.html
//! [`DamlError`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/data/enum.DamlError.html
//! [`DamlValue`]: https://docs.rs/daml-grpc/0.2.0/daml_grpc/data/value/enum.DamlValue.html
//! [Daml Template]: https://docs.daml.com/daml/reference/templates.html
//! [Daml Template Choices]: https://docs.daml.com/daml/reference/choices.html
//! [Daml Data (Record)]: https://docs.daml.com/daml/reference/data-types.html
//! [Daml Data (Variant)]: https://docs.daml.com/daml/reference/data-types.html#sum-types
//! [Daml Variant]: https://docs.daml.com/daml/reference/data-types.html#sum-types
//! [Daml Enum]: https://docs.daml.com/daml/reference/data-types.html#sum-types
//! [Daml primitive type alias]: ../daml-derive/index.html#mapping-daml-data-types-to-rust
#![warn(clippy::all, clippy::pedantic, clippy::nursery, rust_2018_idioms)]
#![allow(
    clippy::module_name_repetitions,
    clippy::default_trait_access,
    clippy::needless_pass_by_value,
    clippy::manual_assert
)]
#![allow(non_snake_case, unused_extern_crates)]
#![forbid(unsafe_code)]
#![doc(html_favicon_url = "https://docs.daml.com/_static/images/favicon/favicon-32x32.png")]
#![doc(html_logo_url = "https://docs.daml.com/_static/images/DAML_Logo_Blue.svg")]
#![doc(html_root_url = "https://docs.rs/daml-derive/0.2.0")]

mod convert;
mod generator;

use darling::FromMeta;
use syn::{parse_macro_input, AttributeArgs, DeriveInput, ItemImpl};

/// Custom attribute for modelling Daml templates.
///
/// A [Daml Template](https://docs.daml.com/daml/reference/templates.html) is modelled in Rust as a `struct` with the
/// custom `DamlTemplate` attribute.
///
/// # Format
///
/// ```ignore
/// #[DamlTemplate(package_id = "...", module_name = "...")]
/// pub struct MyTemplate {
///     ... fields ...
/// }
/// ```
///
/// The `DamlTemplate` attribute takes two mandatory parameters:
/// - `package_id` - the id of the Daml package which contains the module which declares this template
/// - `module_name` - the fully qualified Daml module name within the package
///
/// Each field within the `struct` takes the form `field_name: FieldType` and fields are separated with an (optionally
/// trailing) comma as usual.  Any [Daml primitive type alias] or a custom [`macro@DamlData`] type may be used.  Note
/// that all fields must be owned by the `struct`, references and lifetimes are not supported.
///
/// Note that the supplied `struct` is fully replaced by this custom attribute and only the `struct` name, field names
/// and types are read, all other information such as visibility modifiers or other attributes are discarded.
///
/// The generated `struct` (such as `MyTemplate`) represents the Daml template. The custom attribute also generates
/// another `struct` (named as `MyTemplateContract`) which represents a contract instance of template on the Daml
/// ledger.  See below for how these two `struct` types can be used together to create and observe contract instances
/// on the Daml ledger.
///
/// # Panics
///
/// Panics if the template struct cannot be parsed.
///
/// # Examples
///
/// Given the following Daml template declared in the `Fuji.PingPong` module of a given package:
///
/// ```daml
/// template Ping
///   with
///     sender: Party
///     receiver: Party
///     count: Int
///   where
///     signatory sender
///     observer receiver
/// ```
///
/// This can be represented in Rust as follows:
///
/// ```no_run
/// use daml::prelude::*;
///
/// #[DamlTemplate(package_id = r"...package id hash omitted...", module_name = "Fuji.PingPong")]
/// pub struct Ping {
///     pub sender: DamlParty,
///     pub receiver: DamlParty,
///     pub count: DamlInt64,
/// }
/// ```
/// [Daml primitive type alias]: ../daml-derive/index.html#mapping-daml-data-types-to-rust
#[proc_macro_attribute]
pub fn DamlTemplate(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let template_info: DamlTemplateInfo =
        DamlTemplateInfo::from_list(&parse_macro_input!(attr as AttributeArgs)).unwrap_or_else(|e| panic!("{}", e));
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    generator::generate_template(input, template_info.package_id, template_info.module_name)
}

/// Custom attribute for modelling Daml choices.
///
/// Choices on Daml templates are modelled as `impl` blocks on the `struct` which defines the template.
///
/// # Format
///
/// ```ignore
/// #[DamlChoices]
/// impl MyTemplate {
///
///     #[MyChoice]
///     fn my_choice(&self) {}
///
///     #[MyChoiceWithParams]
///     fn my_choice_with_params(&self, my_first, param: DamlInt64, my_second_param: DamlParty) {}
/// }
/// ```
///
/// Note that:
///
/// - There can be many choices defined with a single impl block
/// - Each choice must take `&self` as the first parameter and returns `()`
/// - Each choice method may take any number of additional parameters
/// - The name of the Daml choice (i.e. `MyChoice`) must match the Daml template choice name
/// - The name of the Daml method (i.e. `my_choice`) must match the Daml template choice name in `snake_case`
/// - Any method body provided is ignored
/// - No distinction is made between consuming & non-consuming choices
/// - All paramters must be either a [Daml primitive type alias] or a user defined [`macro@DamlData`]
///
/// # Examples
///
/// Given the following Daml template declared in the `Fuji.PingPong` module of a given package:
///
/// ```daml
/// template Ping
///   with
///     sender: Party
///     receiver: Party
///     count: Int
///   where
///     signatory sender
///     observer receiver
///
///     controller receiver can
///       ResetCount : ()
///         with
///           new_count: Int
///         do
///           create Pong with sender; receiver; count = new_count
///           return ()
/// ```
///
/// This can be represented in Rust by using the [`macro@DamlTemplate`] and [`macro@DamlChoices`] custom attributes:
///
/// ```no_run
/// use daml::prelude::*;
///
/// #[DamlTemplate(package_id = r"...package id hash omitted...", module_name = "Fuji.PingPong")]
/// pub struct Ping {
///     pub sender: DamlParty,
///     pub receiver: DamlParty,
///     pub count: DamlInt64,
/// }
///
/// #[DamlChoices]
/// impl Ping {
///     #[ResetCount]
///     fn reset_count(&self, new_count: DamlInt64) {}
/// }
/// ```
/// [Daml primitive type alias]: ../daml-derive/index.html#mapping-daml-data-types-to-rust
#[proc_macro_attribute]
pub fn DamlChoices(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: ItemImpl = parse_macro_input!(input as ItemImpl);
    generator::generate_choices(input)
}

/// Custom attribute for modelling Daml data structures.
///
/// A [Daml Data](https://docs.daml.com/daml/reference/data-types.html#records-and-record-types) representing a `Record`
/// and can be modelled in Rust as `struct` with the `DamlData` custom attribute.
///
/// # Record Format
///
/// Given the following Daml `data` definition:
///
/// ```daml
/// data RGBA = RGBA
///     with
///         red: Int
///         green: Int
///         blue: Int
///         alpha: Int
///     deriving (Eq, Show)
/// ```
/// This can be represented as a Rust `struct` with the `DamlData` custom attribute as follows:
///
/// ```no_run
/// use daml::prelude::*;
///
/// #[DamlData]
/// pub struct RGBA {
///     pub red: DamlInt64,
///     pub green: DamlInt64,
///     pub blue: DamlInt64,
///     pub alpha: DamlInt64,
/// }
/// ```
/// Each field within the `struct` takes the form `field_name: FieldType` and fields are separated with an (optionally
/// trailing) comma as usual.  Any [Daml primitive type alias] or a custom [`macro@DamlData`] type may be used.  Note
/// that all fields must be owned by the `struct`, references and lifetimes are not supported.
///
/// Note that the supplied `struct` is fully replaced by this custom attribute and only the `struct` name, field names
/// and types are read, all other information such as visibility modifiers or other attributes are discarded.
///
/// [Daml primitive type alias]: ../daml-derive/index.html#mapping-daml-data-types-to-rust
#[proc_macro_attribute]
pub fn DamlData(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    generator::generate_data_struct(input)
}

/// Custom attribute for modelling Daml variants.
///
/// A [Daml Variant](https://docs.daml.com/daml/reference/data-types.html#sum-types) representing a `Sum` types
/// (variant) can be modelled in Rust as `enum` with the `DamlVariant` custom attribute.
///
/// # Format
///
/// Given the following Daml `data` definition:
///
/// ```daml
/// data Color =
///     Red |
///     Green |
///     Blue |
///     Custom [Int] |
///     Other RGBA
///     deriving (Eq, Show)
/// ```
///
/// This can be represented as a Rust `enum` with the `DamlVariant` custom attribute as follows:
///
/// ```no_run
/// use daml::prelude::*;
///
/// #[DamlData]
/// pub struct RGBA {
///     pub red: DamlInt64,
///     pub green: DamlInt64,
///     pub blue: DamlInt64,
///     pub alpha: DamlInt64,
/// }
///
/// #[DamlVariant]
/// pub enum Color {
///     Red,
///     Green,
///     Blue,
///     Custom(DamlList<DamlInt64>),
///     Other(RGBA),
/// }
/// ```
///
/// Each Daml `Sum` variant constructor is represented as a Rust `enum` variant.  Each variant may have either zero
/// or a single type parameter of any [Daml primitive type alias] or a custom [`macro@DamlData`] type.
///
/// For clarify, in the above example there are three separate cases:
///
/// - No parameter: simple cases such as `Red`, `Green` and `Blue` in the example above
/// - Single [Daml primitive type alias] type parameter: for cases such as `Custom` in the example above
/// - Single [`macro@DamlData`] type parameter: for cases of nested record types such as `Other` in the example above
///
/// [Daml primitive type alias]: ../daml-derive/index.html#mapping-daml-data-types-to-rust
#[proc_macro_attribute]
pub fn DamlVariant(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    generator::generate_data_variant(input)
}

/// Custom attribute for modelling Daml enums.
///
/// A [Daml Enum](https://docs.daml.com/daml/reference/data-types.html#sum-types) is a special case of a Daml variant
/// where all constructors are parameterless. This can be modelled in Rust as `enum` with the `DamlEnum` custom
/// attribute.
///
/// # Format
///
/// Given the following Daml `data` definition:
///
/// ```daml
/// data DayOfWeek
///   = Monday
///   | Tuesday
///   | Wednesday
///   | Thursday
///   | Friday
///   | Saturday
///   | Sunday
/// ```
///
/// This can be represented as a Rust `enum` with the `DamlEnum` custom attribute as follows:
///
/// ```no_run
/// use daml::prelude::*;
///
/// #[DamlEnum]
/// pub enum Color {
///     Red,
///     Monday,
///     Tuesday,
///     Wednesday,
///     Thursday,
///     Friday,
///     Saturday,
///     Sunday,
/// }
/// ```
#[proc_macro_attribute]
pub fn DamlEnum(_attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = parse_macro_input!(input as DeriveInput);
    generator::generate_data_enum(input)
}

/// Function-like procedural macro to generate Rust code for a Daml `dar` file.
///
/// # Examples
///
/// To generate Rust types for a `Foo.dar`:
///
/// ```ignore
/// daml_codegen!(dar_file = r"Foo.dar");
/// ```
///
/// To generate [`RenderMethod::Intermediate`](daml_codegen::generator::RenderMethod::Intermediate) Rust types for a
/// `Foo.dar` for Daml modules which match the regex `Fuji.*`:
///
/// ```ignore
/// daml_codegen!(dar_file = r"Foo.dar", module_filter_regex = "Fuji.*", mode = "intermediate");
/// ```
#[proc_macro]
pub fn daml_codegen(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(attr as AttributeArgs);
    generator::generate_tokens(args)
}

#[doc(hidden)]
#[derive(Debug, FromMeta)]
struct CodeGeneratorParameters {
    pub dar_file: String,
    #[darling(multiple)]
    pub module_filter_regex: Vec<String>,
    #[darling(default)]
    pub mode: Option<String>,
}

#[doc(hidden)]
#[derive(Debug, FromMeta)]
struct DamlTemplateInfo {
    pub package_id: String,
    pub module_name: String,
}
