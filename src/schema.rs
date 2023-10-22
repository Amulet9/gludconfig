use std::collections::BTreeMap;

use crate::{builder_get, property::Property, trigger::Trigger, value::Value};
#[derive(Debug, serde::Serialize, serde::Deserialize, zvariant::Type)]
pub struct Schema {
    name: String,
    version: f32,
    pub(crate) properties: Vec<Property>,
    pub(crate) triggers: Vec<Trigger>,
}

impl PartialEq for Schema {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Schema {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> f32 {
        self.version
    }

    pub fn iter_properties_mut(&mut self) -> impl Iterator<Item = &mut Property> {
        self.properties.iter_mut()
    }

    pub fn iter_triggers_mut(&mut self) -> impl Iterator<Item = &mut Trigger> {
        self.triggers.iter_mut()
    }

    pub fn into_triggers(self) -> impl Iterator<Item = Trigger> {
        self.triggers.into_iter()
    }

    pub fn into_properties(self) -> impl Iterator<Item = Property> {
        self.properties.into_iter()
    }

    pub fn get_property_mut(&mut self, name: &str) -> Option<&mut Property> {
        self.properties.iter_mut().find(|p| p.name() == name)
    }

    pub fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name() == name)
    }

    pub fn get_trigger(&self, name: &str) -> Option<&Trigger> {
        self.triggers.iter().find(|t| t.name() == name)
    }

    pub fn get_trigger_mut(&mut self, name: &str) -> Option<&mut Trigger> {
        self.triggers.iter_mut().find(|t| t.name() == name)
    }
}

#[derive(Default)]
pub struct SchemaBuilder {
    triggers: Vec<Trigger>,
    name: Option<String>,
    version: Option<f32>,
    properties: Vec<Property>,
}

impl SchemaBuilder {
    pub fn version(mut self, version: f32) -> Self {
        self.version = Some(version);
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn properties(mut self, properties: Vec<Property>) -> Self {
        properties
            .into_iter()
            .fold(self, |_self, prop| _self.property(prop))
    }

    pub fn triggers(mut self, triggers: Vec<Trigger>) -> Self {
        triggers
            .into_iter()
            .fold(self, |_self, trig| _self.trigger(trig))
    }

    pub fn trigger(mut self, trigger: Trigger) -> Self {
        match self.triggers.contains(&trigger) {
            true => panic!("Attempt to push duplicate trigger"),
            false => {
                self.triggers.push(trigger);
            }
        }
        self
    }

    pub fn property(mut self, property: Property) -> Self {
        match self.properties.contains(&property) {
            true => panic!("Attempt to push duplicate property"),
            false => self.properties.push(property),
        }
        self
    }

    pub fn build(mut self) -> anyhow::Result<Schema> {
        let name = builder_get!(
            self,
            name,
            "name",
            "SchemaBuilder",
            format!("Missing Property: name: Please use `SchemaBuilder::name` to set it")
        );

        let version = builder_get!(
            self,
            version,
            "version",
            "SchemaBuilder",
            format!("Missing Property: version: Please use `SchemaBuilder::version` to set it")
        );

        Ok(Schema {
            version,
            triggers: self.triggers,
            name: name,
            properties: self.properties,
        })
    }
}

impl Schema {
    pub fn builder() -> SchemaBuilder {
        SchemaBuilder::default()
    }
}