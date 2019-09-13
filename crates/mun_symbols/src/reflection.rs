use crate::prelude::*;
use uuid::Uuid;

use std::any::TypeId;

/// Reflection information about a type.
// TODO: How to resolve generic fields and methods?
#[derive(Debug)]
pub struct TypeInfo {
    pub type_id: TypeId,
    pub uuid: Uuid,
    pub name: &'static str,
    pub fields: &'static [&'static FieldInfo],
    pub methods: &'static [&'static MethodInfo],
}

impl TypeInfo {
    /// Finds the type's fields that match `filter`.
    pub fn find_fields(&self, filter: fn(&&FieldInfo) -> bool) -> impl Iterator<Item = &FieldInfo> {
        self.fields.iter().map(|f| *f).filter(filter)
    }

    /// Retrieves the type's field with the specified `name`, if it exists.
    pub fn get_field(&self, name: &str) -> Option<&FieldInfo> {
        self.fields.iter().find(|f| f.name == name).map(|f| *f)
    }

    /// Retrieves the type's fields.
    pub fn get_fields(&self) -> impl Iterator<Item = &FieldInfo> {
        self.fields.iter().map(|f| *f)
    }

    /// Finds the type's methods that match `filter`.
    pub fn find_methods(
        &self,
        filter: fn(&&MethodInfo) -> bool,
    ) -> impl Iterator<Item = &MethodInfo> {
        self.methods.iter().map(|m| *m).filter(filter)
    }

    /// Retrieves the type's method with the specified `name`, if it exists.
    pub fn get_method(&self, name: &str) -> Option<&MethodInfo> {
        self.methods.iter().find(|f| f.name == name).map(|f| *f)
    }

    /// Retrieves the type's methods.
    pub fn get_methods(&self) -> impl Iterator<Item = &MethodInfo> {
        self.methods.iter().map(|m| *m)
    }
}

/// A type to emulate dynamic typing across compilation units for static types.
pub trait Reflection: 'static {
    /// Retrieves the type's `TypeInfo`.
    fn type_info() -> &'static TypeInfo;

    /// Retrieves the type's `ModuleInfo`.
    fn module_info() -> &'static ModuleInfo;
}

/// A type to emulate dynamic typing across compilation units for type instances.
pub trait Reflectable: 'static {
    fn reflect(&self) -> &'static TypeInfo;
}

impl<T: 'static + Reflection> Reflectable for T {
    fn reflect(&self) -> &'static TypeInfo {
        Self::type_info()
    }
}

impl dyn Reflectable {
    /// Returns whether the reflectable's type is the same as `T`.
    pub fn is<T: Reflection>(&self) -> bool {
        // TypeId only works in the same compile unit
        T::type_info().uuid == self.reflect().uuid
    }

    /// Returns some reference to the reflectable if it is of type `T`, or `None` if it
    /// isn't.
    pub fn downcast_ref<T: Reflection>(&self) -> Option<&T> {
        if self.is::<T>() {
            Some(unsafe { &*(self as *const dyn Reflectable as *const T) })
        } else {
            None
        }
    }
}

lazy_static! {
    static ref F32_TYPE_INFO: TypeInfo = TypeInfo {
        type_id: TypeId::of::<f32>(),
        uuid: Uuid::parse_str("fc4bacef-cd0e-4d58-8d4d-19504d58d87f").unwrap(),
        name: "f32",
        fields: &[],
        methods: &[],
    };
    static ref F64_TYPE_INFO: TypeInfo = TypeInfo {
        type_id: TypeId::of::<f64>(),
        uuid: Uuid::parse_str("fe58c2ab-f8db-4dab-80b1-578d871bc769").unwrap(),
        name: "f64",
        fields: &[],
        methods: &[],
    };
    static ref EMPTY_TYPE_INFO: TypeInfo = TypeInfo {
        type_id: TypeId::of::<()>(),
        uuid: Uuid::parse_str("3575c27d-fee0-4240-a658-d9c3edb73d0e").unwrap(),
        name: "()",
        fields: &[],
        methods: &[],
    };
    static ref CORE_MODULE: ModuleInfo = { ModuleInfo::new("core", &[], &[], &[]) };
}

impl Reflection for f32 {
    fn type_info() -> &'static TypeInfo {
        &F32_TYPE_INFO
    }

    fn module_info() -> &'static ModuleInfo {
        &CORE_MODULE
    }
}

impl Reflection for f64 {
    fn type_info() -> &'static TypeInfo {
        &F64_TYPE_INFO
    }

    fn module_info() -> &'static ModuleInfo {
        &CORE_MODULE
    }
}

impl Reflection for () {
    fn type_info() -> &'static TypeInfo {
        &EMPTY_TYPE_INFO
    }

    fn module_info() -> &'static ModuleInfo {
        &CORE_MODULE
    }
}
