// Module mc_instance
// Types:
//  McInstance, McInstanceList

use regex::Regex;
use std::borrow::Cow;
use std::{ops::Deref, ops::DerefMut};

use super::McAddress;
use super::McDimType;
use super::McIdentifier;
use super::McObjectType;
use super::McSupportData;
use super::McValueType;
use super::Registry;
use super::RegistryError;
use serde::Deserialize;
use serde::Serialize;

//-------------------------------------------------------------------------------------------------
// Instance
// Measurement and calibration object instances

/// Instance which may be a parameter, axis, or measurement value of complex type
#[derive(Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct McInstance {
    /// Name of the instance, unique together with the event id
    pub name: McIdentifier,
    /// Type, metadata and matrix dimensions, recursion here if McValueType::TypeDef
    pub dim_type: McDimType,
    /// Metadata for the instance
    pub mc_support_data: McSupportData,
    /// Addressing information for the instance
    pub address: McAddress,
}

impl McInstance {
    /// Create an instance description, without adding it to a registry
    pub fn new<T: Into<McIdentifier>>(name: T, dim_type: McDimType, mc_support_data: McSupportData, address: McAddress) -> McInstance {
        McInstance {
            name: name.into(),
            dim_type,
            mc_support_data,
            address,
        }
    }

    /// Get type and dimensions
    pub fn get_dim_type(&self) -> &McDimType {
        &self.dim_type
    }

    /// Get the metadata
    pub fn get_mc_support_data(&self) -> &McSupportData {
        &self.mc_support_data
    }

    /// Get the addressing information
    pub fn get_address(&self) -> &McAddress {
        &self.address
    }

    /// Get the instance name
    /// The instance name may not be unique
    pub fn get_name(&self) -> &'static str {
        self.name.as_str()
    }

    /// Get the instance name with optional application name prefix
    /// The instance name may not be unique
    pub fn get_prefixed_name(&self, registry: &Registry) -> Cow<'static, str> {
        if registry.get_prefix_names_mode() {
            return Cow::Owned(format!("{}.{}", registry.application.get_name(), self.name));
        }
        Cow::Borrowed(self.name.as_str())
    }

    /// Get the unique instance name with optional application name prefix and postfixed by instance index
    /// If there is an event event index > 0, which means there are multiple instances of the same name
    pub fn get_unique_name(&self, registry: &Registry) -> Cow<'static, str> {
        if let Some(event_id) = self.address.get_event_id() {
            if let Some(event) = registry.event_list.find_event_id(event_id) {
                if event.index > 0 {
                    if registry.get_prefix_names_mode() {
                        return Cow::Owned(format!(r#"{}.{}_{}"#, registry.application.get_name(), self.name, event.index));
                    } else {
                        return Cow::Owned(format!(r#"{}_{}"#, self.name, event.index));
                    }
                }
            }
        }
        if registry.get_prefix_names_mode() {
            Cow::Owned(format!(r#"{}.{}"#, registry.application.get_name(), self.name))
        } else {
            Cow::Borrowed(self.name.as_str())
        }
    }

    /// Get the instance event index
    /// If there is an event event index > 0, which means there are multiple instances of the same name
    pub fn get_index(&self, registry: &Registry) -> u16 {
        if let Some(event_id) = self.address.get_event_id() {
            if let Some(event) = registry.event_list.find_event_id(event_id) {
                if event.index > 0 {
                    return event.index;
                }
            }
        }
        0
    }

    // Shortcuts to dim_type
    /// Size in bytes of the complete instance, including all array elements
    pub fn size(&self) -> usize {
        self.dim_type.get_size()
    }
    /// Size in bytes of one element
    pub fn value_size(&self) -> usize {
        self.dim_type.value_type.get_size()
    }
    /// Value type of one element
    pub fn value_type(&self) -> &McValueType {
        &self.dim_type.value_type
    }
    /// Check if the value type is a typedef and return the typedef name if it is
    pub fn get_typedef_name(&self) -> Option<&'static str> {
        match self.dim_type.value_type {
            McValueType::TypeDef(typedef_name) => Some(typedef_name.as_str()),
            _ => None,
        }
    }

    /// Array dimension in x direction, 1 for a scalar
    pub fn x_dim(&self) -> u16 {
        self.dim_type.get_dim()[0]
    }
    /// Array dimension in y direction, 1 if not a matrix
    pub fn y_dim(&self) -> u16 {
        self.dim_type.get_dim()[1]
    }

    // Shortcuts to mc_support_data
    /// Object type: measurement, characteristic or axis
    pub fn object_type(&self) -> McObjectType {
        self.get_mc_support_data().get_object_type()
    }
    /// Check if this is a calibration object (characteristic or axis)
    /// # Panics
    /// If the object type is `Unspecified`
    pub fn is_calibration_object(&self) -> bool {
        self.get_mc_support_data().is_calibration_object()
    }
    /// Check if this is a measurement object
    pub fn is_measurement_object(&self) -> bool {
        self.get_mc_support_data().is_measurement_object()
    }
    /// Check if this is an axis
    pub fn is_axis(&self) -> bool {
        self.get_mc_support_data().is_axis()
    }
    /// Check if this is a characteristic
    pub fn is_characteristic(&self) -> bool {
        self.get_mc_support_data().is_characteristic()
    }
    /// Physical unit, empty if none
    pub fn unit(&self) -> &'static str {
        self.get_mc_support_data().get_unit()
    }
    /// Description text, empty if none
    pub fn comment(&self) -> &'static str {
        self.get_mc_support_data().get_comment()
    }
    /// Name of the x axis referenced by a curve or map
    pub fn x_axis_ref(&self) -> Option<McIdentifier> {
        self.get_mc_support_data().get_x_axis_ref()
    }
    /// Name of the y axis referenced by a map
    pub fn y_axis_ref(&self) -> Option<McIdentifier> {
        self.get_mc_support_data().get_y_axis_ref()
    }
    /// Name of the conversion of a fixed x axis
    pub fn x_axis_conv(&self) -> Option<McIdentifier> {
        self.get_mc_support_data().get_x_axis_conv()
    }
    /// Name of the conversion of a fixed y axis
    pub fn y_axis_conv(&self) -> Option<McIdentifier> {
        self.get_mc_support_data().get_y_axis_conv()
    }
    /// Lower limit: the explicit minimum if set, otherwise the physical value of the minimum of the value type
    pub fn get_min(&self) -> Option<f64> {
        self.get_mc_support_data().get_min(self.dim_type.value_type)
    }
    /// Upper limit: the explicit maximum if set, otherwise the physical value of the maximum of the value type
    pub fn get_max(&self) -> Option<f64> {
        self.get_mc_support_data().get_max(self.dim_type.value_type)
    }

    // Shortcuts to address
    /// Event id of the associated event, `None` if there is none
    pub fn event_id(&self) -> Option<u16> {
        self.address.get_event_id()
    }
    /// Address offset relative to the event or the calibration segment
    /// # Panics
    /// If the address is not segment or event relative, see [`McAddress::get_addr_offset`]
    pub fn addr_offset(&self) -> i32 {
        self.address.get_addr_offset()
    }
    /// Name of the calibration segment of a calibration object
    pub fn calseg_name(&self) -> Option<McIdentifier> {
        self.address.get_calseg_name()
    }
}

//-------------------------------------------------------------------------------------------------
// InstanceList

/// List of all measurement and calibration instances of a registry
#[derive(Debug, Serialize, Deserialize)]
pub struct McInstanceList(Vec<McInstance>);

impl Default for McInstanceList {
    fn default() -> Self {
        McInstanceList::new()
    }
}

impl Deref for McInstanceList {
    type Target = [McInstance];
    fn deref(&self) -> &[McInstance] {
        &self.0
    }
}

impl DerefMut for McInstanceList {
    fn deref_mut(&mut self) -> &mut [McInstance] {
        &mut self.0
    }
}

impl McInstanceList {
    /// Create an empty list
    pub fn new() -> Self {
        McInstanceList(Vec::with_capacity(100))
    }

    /// Append an instance without any checks
    pub fn push(&mut self, object: McInstance) {
        self.0.push(object);
    }

    /// Move all instances of `other` to the end of this list
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(other.0.as_mut());
    }

    /// Sort the list by name
    pub fn sort_by_name(&mut self) {
        self.0.sort_by_key(|a| a.name);
    }

    /// Sort the list by name, instances with the same name by event id
    pub fn sort_by_name_and_event(&mut self) {
        self.0.sort_by(|a, b| {
            if a.name == b.name {
                a.address.get_event_id_unchecked().cmp(&b.address.get_event_id_unchecked())
            } else {
                a.name.cmp(&b.name)
            }
        });
    }

    //---------------------------------------------------------------------------------------------------------

    /// Add a measurement and calibration object instance
    /// # Errors
    ///   `RegistryError::Duplicate` if an instance with the same name and event id already exists
    ///   `RegistryError::ObjectTypeUnspecified` if the object type is unspecified
    /// # Arguments
    ///   * `name` - Name of the instance
    ///   * `dim_type` - Type, metadata and array dimensions
    ///   * `address` - Addressing information for the instance
    #[allow(clippy::too_many_arguments)]
    pub fn add_instance<T: Into<McIdentifier>>(&mut self, name: T, dim_type: McDimType, mc_support_data: McSupportData, address: McAddress) -> Result<(), RegistryError> {
        let name = name.into();

        log::debug!("Registry add_instance: {} dim_type={:?}  addr={}", name, dim_type, address);
        if mc_support_data.get_object_type() == McObjectType::Unspecified {
            return Err(RegistryError::ObjectTypeUnspecified(name.to_string()));
        }

        // Error if duplicate in instance namespace (A2l characteristics, measurements, axis and instances)
        // Note that multiple instances of the same name are allowed if they have different event_id
        if self.into_iter().any(|i| (i.name == name) && (i.event_id() == address.get_event_id())) {
            log::error!("Duplicate instance named '{}'!", name);
            return Err(RegistryError::Duplicate(name.to_string()));
        }

        let c: McInstance = McInstance::new(name, dim_type, mc_support_data, address);
        self.push(c);
        Ok(())
    }

    /// Get an instance by exact name, type (set to Unspecified if any) and event_id (set to None if any)
    /// Returns the instance or None
    pub fn get_instance(&self, name: &str, object_type: McObjectType, event_id: Option<u16>) -> Option<&McInstance> {
        self.into_iter().find(|i| {
            ((i.get_address().get_event_id() == event_id) || event_id.is_none())
                && ((object_type == McObjectType::Unspecified) || (i.object_type() == object_type))
                && (name == i.name.as_str())
        })
    }

    /// Get a mutable reference to an instance by exact name and optional event_id
    /// Returns the instance or None
    pub fn get_instance_mut(&mut self, name: &str, event_id: Option<u16>) -> Option<&mut McInstance> {
        self.0
            .iter_mut()
            .find(|i| ((i.address.get_event_id() == event_id) || event_id.is_none()) && (name == i.name.as_str()))
    }

    /// Find an instance by regular expression, optional by object type (set to Unspecified if any) or by event_id (set to None if any)
    /// Returns the first instance that matches the criteria or None
    pub fn find_instance_regex(&self, regex: &str, object_type: McObjectType, event_id: Option<u16>) -> Option<&McInstance> {
        if let Ok(regex) = Regex::new(regex) {
            self.into_iter().find(|i| {
                (i.get_address().get_event_id() == event_id || event_id.is_none())
                    && ((object_type == McObjectType::Unspecified || i.object_type() == object_type) && regex.is_match(i.name.as_str()))
            })
        } else {
            None
        }
    }

    /// Find all instances by regular expression, optional by object type (set to Unspecified if any) or by event_id (set to None if any)
    /// Return all instances that match the criteria
    pub fn find_instances_regex(&self, regex: &str, object_type: McObjectType, event_id: Option<u16>) -> Vec<String> {
        if let Ok(regex) = Regex::new(regex) {
            self.into_iter()
                .filter(|i| {
                    (i.get_address().get_event_id() == event_id || event_id.is_none())
                        && ((object_type == McObjectType::Unspecified || i.object_type() == object_type) && regex.is_match(i.name.as_str()))
                })
                .map(|i| i.name.to_string())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Keep only the instances for which `f` returns `true`
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&McInstance) -> bool,
    {
        self.0.retain(f);
    }
}

//-------------------------------------------------------------------------------------------------
// IntoIterator

impl<'a> IntoIterator for &'a McInstanceList {
    type Item = &'a McInstance;
    type IntoIter = std::slice::Iter<'a, McInstance>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut McInstanceList {
    type Item = &'a mut McInstance;
    type IntoIter = std::slice::IterMut<'a, McInstance>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
