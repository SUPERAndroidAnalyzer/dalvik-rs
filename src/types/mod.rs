//! Types module.

pub mod read;
use self::read::ClassData;
use crate::error;
use bitflags::bitflags;
use std::{fmt, ops::Deref, str::FromStr};

#[derive(Debug, Clone)]
/// Basic built-in types.
pub enum Type {
    /// Void type.
    Void,
    /// Boolean.
    Boolean,
    /// Byte (8 bits).
    Byte,
    /// Short (16 bits).
    Short,
    /// Char (16 bits).
    Char,
    /// Int (32 bits).
    Int,
    /// Long (64 bits).
    Long,
    /// Float (32 bits).
    Float,
    /// Double (64 bits).
    Double,
    /// Fully qualified named type.
    ///
    /// Example: an object.
    FullyQualifiedName(String),
    /// Array.
    Array {
        /// Array dimensions.
        dimensions: u8,
        /// Type of the array.
        array_type: Box<Type>,
    },
}

impl FromStr for Type {
    type Err = error::Parse;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        match chars.next() {
            Some('V') => Ok(Self::Void),
            Some('Z') => Ok(Self::Boolean),
            Some('B') => Ok(Self::Byte),
            Some('S') => Ok(Self::Short),
            Some('C') => Ok(Self::Char),
            Some('I') => Ok(Self::Int),
            Some('J') => Ok(Self::Long),
            Some('F') => Ok(Self::Float),
            Some('D') => Ok(Self::Double),
            Some('L') => Ok(Self::FullyQualifiedName(chars.collect())),
            Some('[') => {
                let mut dimensions = 1;
                loop {
                    match chars.next() {
                        Some('[') => dimensions += 1,
                        Some(t) => {
                            let mut type_str = String::with_capacity(s.len() - dimensions as usize);
                            type_str.push(t);
                            type_str.push_str(chars.as_str());
                            return Ok(Self::Array {
                                dimensions,
                                array_type: Box::new(type_str.parse()?),
                            });
                        }
                        None => {
                            return Err(error::Parse::InvalidTypeDescriptor(s.to_owned()));
                        }
                    }
                }
            }
            _ => Err(error::Parse::InvalidTypeDescriptor(s.to_owned())),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Boolean => write!(f, "boolean"),
            Self::Byte => write!(f, "byte"),
            Self::Short => write!(f, "short"),
            Self::Char => write!(f, "char"),
            Self::Int => write!(f, "int"),
            Self::Long => write!(f, "long"),
            Self::Float => write!(f, "float"),
            Self::Double => write!(f, "double"),
            Self::FullyQualifiedName(name) => write!(f, "{}", name),
            Self::Array {
                dimensions,
                array_type,
            } => write!(f, "{}[{}]", array_type, dimensions),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShortyReturnType {
    Void,
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Reference,
}

impl ShortyReturnType {
    fn from_char(c: char) -> Result<Self, error::Parse> {
        match c {
            'V' => Ok(Self::Void),
            'Z' => Ok(Self::Boolean),
            'B' => Ok(Self::Byte),
            'S' => Ok(Self::Short),
            'C' => Ok(Self::Char),
            'I' => Ok(Self::Int),
            'J' => Ok(Self::Long),
            'F' => Ok(Self::Float),
            'D' => Ok(Self::Double),
            'L' => Ok(Self::Reference),
            _ => Err(error::Parse::InvalidShortyType(c)),
        }
    }
}

impl From<Type> for ShortyReturnType {
    fn from(t: Type) -> Self {
        match t {
            Type::Void => Self::Void,
            Type::Boolean => Self::Boolean,
            Type::Byte => Self::Byte,
            Type::Short => Self::Short,
            Type::Char => Self::Char,
            Type::Int => Self::Int,
            Type::Long => Self::Long,
            Type::Float => Self::Float,
            Type::Double => Self::Double,
            Type::FullyQualifiedName(_) | Type::Array { .. } => Self::Reference,
        }
    }
}

impl From<ShortyFieldType> for ShortyReturnType {
    fn from(ft: ShortyFieldType) -> Self {
        match ft {
            ShortyFieldType::Boolean => Self::Boolean,
            ShortyFieldType::Byte => Self::Byte,
            ShortyFieldType::Short => Self::Short,
            ShortyFieldType::Char => Self::Char,
            ShortyFieldType::Int => Self::Int,
            ShortyFieldType::Long => Self::Long,
            ShortyFieldType::Float => Self::Float,
            ShortyFieldType::Double => Self::Double,
            ShortyFieldType::Reference => Self::Reference,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShortyFieldType {
    Boolean,
    Byte,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Reference,
}

impl ShortyFieldType {
    fn from_char(c: char) -> Result<Self, error::Parse> {
        match c {
            'Z' => Ok(Self::Boolean),
            'B' => Ok(Self::Byte),
            'S' => Ok(Self::Short),
            'C' => Ok(Self::Char),
            'I' => Ok(Self::Int),
            'J' => Ok(Self::Long),
            'F' => Ok(Self::Float),
            'D' => Ok(Self::Double),
            'L' => Ok(Self::Reference),
            _ => Err(error::Parse::InvalidShortyType(c)),
        }
    }
}

/// Short form of type descriptor.
#[derive(Debug)]
pub struct ShortyDescriptor {
    return_type: ShortyReturnType,
    field_types: Box<[ShortyFieldType]>,
}

impl FromStr for ShortyDescriptor {
    type Err = error::Parse;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let return_type = if let Some(c) = chars.next() {
            ShortyReturnType::from_char(c)?
        } else {
            return Err(error::Parse::InvalidShortyDescriptor(s.to_owned()));
        };
        let mut field_types = Vec::with_capacity(s.len() - 1);
        for c in chars {
            field_types.push(ShortyFieldType::from_char(c)?);
        }
        Ok(Self {
            return_type,
            field_types: field_types.into_boxed_slice(),
        })
    }
}

/// Prototype implementation.
#[derive(Debug)]
pub struct Prototype {
    descriptor: ShortyDescriptor,
    return_type: Type,
    parameters: Option<Box<[Type]>>,
}

impl Prototype {
    /// Creates a new prototype.
    pub fn new<TA: Into<Option<Box<[Type]>>>>(
        descriptor: ShortyDescriptor,
        return_type: Type,
        parameters: TA,
    ) -> Self {
        Self {
            descriptor,
            return_type,
            parameters: parameters.into(),
        }
    }
}

/// Annotation visibility.
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// Build time visibility.
    Build,
    /// Runtime visibility.
    Runtime,
    /// System visibility.
    System,
}

/// Value of a variable.
#[derive(Debug, Clone)]
pub enum Value {
    /// Byte.
    Byte(i8),
    /// Short (16 bits).
    Short(i16),
    /// Char (16 bts).
    Char(u16),
    /// Int (32 bits).
    Int(i32),
    /// Long (64 bits).
    Long(i64),
    /// Float (32 bits).
    Float(f32),
    /// Double (64 bits).
    Double(f64),
    /// String, with the index into the string IDs list.
    String(u32),
    /// Type, with the index into the type IDs list.
    Type(u32),
    /// Field, with the index into the field IDs list.
    Field(u32),
    /// Method with the index into the prototype IDs list.
    Method(u32),
    /// Enum with the index into the fields IDs list.
    Enum(u32),
    /// An array of values.
    Array(Array),
    /// Annotation.
    Annotation(EncodedAnnotation),
    /// Null.
    Null,
    /// Boolean.
    Boolean(bool),
}

/// Array.
#[derive(Debug, Clone)]
pub struct Array {
    inner: Box<[Value]>,
}

/// Annotation element.
#[derive(Debug, Clone)]
pub struct AnnotationElement {
    name: u32,
    value: Value,
}

impl AnnotationElement {
    /// Gets the index of the name string.
    pub fn name_index(&self) -> u32 {
        self.name
    }
}

impl Deref for AnnotationElement {
    type Target = Value;

    fn deref(&self) -> &Value {
        &self.value
    }
}

/// Annotation.
#[derive(Debug, Clone)]
pub struct EncodedAnnotation {
    type_id: u32,
    elements: Box<[AnnotationElement]>,
}

impl EncodedAnnotation {
    /// Gets the index of the type of the annotation.
    pub fn type_index(&self) -> u32 {
        self.type_id
    }

    /// Gets the elements of the annotation.
    pub fn elements(&self) -> &[AnnotationElement] {
        &self.elements
    }
}

/// Annotation item
#[derive(Debug, Clone)]
pub struct Annotation {
    visibility: Visibility,
    annotation: EncodedAnnotation,
}

impl Annotation {
    /// Gets the visibility of the annotation item.
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }
}

impl Deref for Annotation {
    type Target = EncodedAnnotation;

    fn deref(&self) -> &EncodedAnnotation {
        &self.annotation
    }
}

/// Annotations directory.
#[derive(Debug, Clone)]
pub struct AnnotationsDirectory {
    class_annotations: Box<[Annotation]>,
    field_annotations: Box<[FieldAnnotations]>,
    method_annotations: Box<[MethodAnnotations]>,
    parameter_annotations: Box<[ParameterAnnotations]>,
}

impl AnnotationsDirectory {
    /// Creates a new annotations directory.
    pub fn new<CA, FA, MA, PA>(
        class_annotations: CA,
        field_annotations: FA,
        method_annotations: MA,
        parameter_annotations: PA,
    ) -> Self
    where
        CA: Into<Box<[Annotation]>>,
        FA: Into<Box<[FieldAnnotations]>>,
        MA: Into<Box<[MethodAnnotations]>>,
        PA: Into<Box<[ParameterAnnotations]>>,
    {
        Self {
            class_annotations: class_annotations.into(),
            field_annotations: field_annotations.into(),
            method_annotations: method_annotations.into(),
            parameter_annotations: parameter_annotations.into(),
        }
    }

    /// Gets the list of class annotations.
    pub fn class_annotations(&self) -> &[Annotation] {
        &self.class_annotations
    }

    /// Gets the list of field annotations.
    pub fn field_annotations(&self) -> &[FieldAnnotations] {
        &self.field_annotations
    }

    /// Gets the list of method annotations.
    pub fn method_annotations(&self) -> &[MethodAnnotations] {
        &self.method_annotations
    }

    /// Gets the list of parameter annotations.
    pub fn parameter_annotations(&self) -> &[ParameterAnnotations] {
        &self.parameter_annotations
    }
}

/// Field annotations.
#[derive(Debug, Clone)]
pub struct FieldAnnotations {
    field_id: u32,
    annotations: Box<[Annotation]>,
}

impl FieldAnnotations {
    /// Creates a new list of field annotations.
    pub fn new(field_id: u32, annotations: Box<[Annotation]>) -> Self {
        Self {
            field_id,
            annotations,
        }
    }

    /// Gets the index of the annotated field.
    pub fn field_index(&self) -> u32 {
        self.field_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// Method annotations.
#[derive(Debug, Clone)]
pub struct MethodAnnotations {
    method_id: u32,
    annotations: Box<[Annotation]>,
}

impl MethodAnnotations {
    /// Creates a new list of method annotations.
    pub fn new(method_id: u32, annotations: Box<[Annotation]>) -> Self {
        Self {
            method_id,
            annotations,
        }
    }

    /// Gets the index of the annotated method.
    pub fn method_index(&self) -> u32 {
        self.method_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

/// Parameter annotations.
#[derive(Debug, Clone)]
pub struct ParameterAnnotations {
    method_id: u32,
    annotations: Box<[Annotation]>,
}

impl ParameterAnnotations {
    /// Creates a new list of method annotations.
    pub fn new(method_id: u32, annotations: Box<[Annotation]>) -> Self {
        Self {
            method_id,
            annotations,
        }
    }

    /// Gets the index of the annotated method.
    pub fn method_index(&self) -> u32 {
        self.method_id
    }

    /// Gets the list of annotations.
    pub fn annotations(&self) -> &[Annotation] {
        &self.annotations
    }
}

bitflags! {
    /// Access flags.
    pub struct AccessFlags: u32 {
        /// Public access.
        const ACC_PUBLIC = 0x1;
        /// Private access.
        const ACC_PRIVATE = 0x2;
        /// Protected access.
        const ACC_PROTECTED = 0x4;
        /// Static access.
        const ACC_STATIC = 0x8;
        /// Final element (non modifiable).
        const ACC_FINAL = 0x10;
        /// Thread - synchronized element.
        const ACC_SYNCHRONIZED = 0x20;
        /// Volatile element.
        const ACC_VOLATILE = 0x40;
        /// Bridge.
        const ACC_BRIDGE = 0x40;
        /// Transient.
        const ACC_TRANSIENT = 0x80;
        /// Varargs.
        const ACC_VARARGS = 0x80;
        /// Native element.
        const ACC_NATIVE = 0x100;
        /// Interface.
        const ACC_INTERFACE = 0x200;
        /// Abstract element.
        const ACC_ABSTRACT = 0x400;
        /// Strict.
        const ACC_STRICT = 0x800;
        /// Synthetic.
        const ACC_SYNTHETIC = 0x1000;
        /// Annotation.
        const ACC_ANNOTATION = 0x2000;
        /// Enum.
        const ACC_ENUM = 0x4000;
        /// Constructor.
        const ACC_CONSTRUCTOR = 0x10000;
        /// Declared as synchronized element.
        const ACC_DECLARED_SYNCHRONIZED = 0x20000;
    }
}

impl fmt::Display for AccessFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();

        if self.contains(Self::ACC_PUBLIC) {
            out.push_str("public ");
        }

        if self.contains(Self::ACC_PRIVATE) {
            out.push_str("private ");
        }

        if self.contains(Self::ACC_PROTECTED) {
            out.push_str("protected ");
        }

        if self.contains(Self::ACC_STATIC) {
            out.push_str("static ");
        }

        if self.contains(Self::ACC_FINAL) {
            out.push_str("final ");
        }

        if self.contains(Self::ACC_SYNCHRONIZED) {
            out.push_str("synchronized ");
        }

        if self.contains(Self::ACC_VOLATILE) {
            out.push_str("volatile ");
        }

        if self.contains(Self::ACC_BRIDGE) {
            out.push_str("bridge ");
        }

        if self.contains(Self::ACC_TRANSIENT) {
            out.push_str("transient ");
        }

        if self.contains(Self::ACC_VARARGS) {
            out.push_str("varargs ");
        }

        if self.contains(Self::ACC_NATIVE) {
            out.push_str("native ");
        }

        if self.contains(Self::ACC_ABSTRACT) {
            out.push_str("abstract ");
        }

        if self.contains(Self::ACC_INTERFACE) {
            out.push_str("interface ");
        }

        if self.contains(Self::ACC_STRICT) {
            out.push_str("strict ");
        }

        if self.contains(Self::ACC_SYNTHETIC) {
            out.push_str("synthetic ");
        }

        // if self.contains(Self::ACC_ANNOTATION) {
        //     out.push_str("annotation ");
        // }

        if self.contains(Self::ACC_ENUM) {
            out.push_str("enum ");
        }

        if self.contains(Self::ACC_CONSTRUCTOR) {
            out.push_str("constructor ");
        }

        if self.contains(Self::ACC_DECLARED_SYNCHRONIZED) {
            out.push_str("synchronized");
        }

        write!(f, "{}", out.trim())
    }
}

/// Structure representing a class.
#[derive(Debug)]
pub struct Class {
    class_index: u32,
    access_flags: AccessFlags,
    superclass_index: Option<u32>,
    interfaces: Box<[Type]>,
    source_file_index: Option<u32>,
    annotations: Option<AnnotationsDirectory>,
    class_data: Option<ClassData>,
    static_values: Option<Array>,
}

impl Class {
    /// Creates a new class.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        class_index: u32,
        access_flags: AccessFlags,
        superclass_index: Option<u32>,
        interfaces: Box<[Type]>,
        source_file_index: Option<u32>,
        annotations: Option<AnnotationsDirectory>,
        class_data: Option<ClassData>,
        static_values: Option<Array>,
    ) -> Self {
        Self {
            class_index,
            access_flags,
            superclass_index,
            interfaces,
            source_file_index,
            annotations,
            class_data,
            static_values,
        }
    }

    /// Gets the index of the class in the type IDs list.
    pub fn class_index(&self) -> u32 {
        self.class_index
    }

    /// Gets the access flags of the class.
    pub fn access_flags(&self) -> AccessFlags {
        self.access_flags
    }

    /// Gets the index in the type IDs list of the superclass for this class, ifd it exists.
    pub fn superclass_index(&self) -> Option<u32> {
        self.superclass_index
    }

    /// Gets the list of interfaces implemented by the class.
    pub fn interfaces(&self) -> &[Type] {
        &self.interfaces
    }

    /// Gets the index of the source file in the string list if it's known.
    pub fn source_file_index(&self) -> Option<u32> {
        self.source_file_index
    }

    /// Gets the annotations for the class, if there are any.
    pub fn annotations(&self) -> Option<&AnnotationsDirectory> {
        self.annotations.as_ref()
    }

    /// Gets the data associated with the class.
    pub fn class_data(&self) -> Option<&ClassData> {
        self.class_data.as_ref()
    }

    /// Gets the arrays with the values for the static files in this class.
    ///
    /// The values are in the same order as the static_field_ids in the class data of the class. If
    /// a value is not found, it is considered `0` or `NULL` depending on the type of the variable.
    pub fn static_values(&self) -> Option<&Array> {
        self.static_values.as_ref()
    }

    // static_values: static_values,
}

#[cfg(test)]
mod test {
    use super::AccessFlags;

    #[test]
    fn it_can_display_access() {
        let access = AccessFlags::ACC_PUBLIC;

        let display = format!("{}", access);

        assert_eq!("public", display);
    }

    #[test]
    fn it_can_display_mixed_access_bitflags() {
        let access = AccessFlags::ACC_PUBLIC | AccessFlags::ACC_DECLARED_SYNCHRONIZED;

        let display = format!("{}", access);

        assert_eq!("public synchronized", display);
    }

    #[test]
    fn it_can_display_mixed_access_bitflags_protected_static_abstract() {
        let access =
            AccessFlags::ACC_PROTECTED | AccessFlags::ACC_ABSTRACT | AccessFlags::ACC_STATIC;

        let display = format!("{}", access);

        assert_eq!("protected static abstract", display);
    }

    #[test]
    fn it_can_display_mixed_access_bitflags_public_interface_abstract_annotation() {
        let access = AccessFlags::ACC_PUBLIC
            | AccessFlags::ACC_INTERFACE
            | AccessFlags::ACC_ABSTRACT
            | AccessFlags::ACC_ANNOTATION;

        let display = format!("{}", access);

        assert_eq!("public abstract interface", display);
    }
}
