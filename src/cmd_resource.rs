use std::{collections::HashMap, marker::PhantomData};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use tf_provider::{
    attribute_path::{AttributePath, AttributePathStep},
    map, value, Attribute, AttributeConstraint, AttributeType, Block, Description, Diagnostics,
    NestedBlock, Resource, Schema, Value, ValueEmpty, ValueMap, ValueString,
};

use crate::connection::Connection;

#[derive(Debug, Default)]
pub struct CmdResource<T: Connection> {
    ph: PhantomData<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct State<T>
where
    T: Connection,
    T: Serialize,
    T: for<'a> Deserialize<'a>,
{
    pub id: ValueString,
    pub inputs: ValueMap<ValueString>,
    pub state: ValueMap<ValueString>,
    pub read: HashMap<String, Value<StateRead>>,
    #[serde(with = "value::serde_from_vec")]
    pub create: Value<StateCreate>,
    #[serde(with = "value::serde_from_vec")]
    pub destroy: Value<StateDestroy>,
    pub update: Vec<Value<StateUpdate>>,
    #[serde(with = "value::serde_from_vec")]
    pub connection: Value<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateCmd {
    pub cmd: ValueString,
    pub env: ValueMap<ValueString>,
}

impl StateCmd {
    fn validate(&self, diags: &mut Diagnostics, mut attr_path: AttributePath) -> Option<()> {
        attr_path += AttributePathStep::Attribute("cmd".into());
        match self.cmd {
            Value::Value(_) => Some(()),
            Value::Null => {
                diags.error_short("`cmd` cannot be null", attr_path);
                None
            }
            Value::Unknown => {
                diags.warning("`cmd` is not known during planning", "It is recommended that the command does not depend on any resource, and use variables instead.", attr_path);
                Some(())
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateUpdate {
    #[serde(flatten)]
    pub cmd: StateCmd,
    pub triggers: ValueMap<ValueString>,
    pub reloads: ValueMap<ValueString>,
}

impl StateUpdate {
    fn validate(&self, diags: &mut Diagnostics, attr_path: AttributePath) -> Option<()> {
        self.cmd.validate(diags, attr_path)
    }
}

pub type StateRead = StateCmd;
pub type StateCreate = StateCmd;
pub type StateDestroy = StateCmd;

#[async_trait]
impl<T: Connection> Resource for CmdResource<T>
where
    T: for<'e> Deserialize<'e>,
    T: Serialize,
{
    type State = Value<State<T>>;
    type PrivateState = ValueEmpty;
    type ProviderMetaState = ValueEmpty;

    fn schema(&self, _diags: &mut Diagnostics) -> Option<Schema> {
        let cmd_attribute = Attribute {
            attr_type: AttributeType::String,
            description: Description::plain("Command to execute when reading the attribute"),
            constraint: AttributeConstraint::Required,
            ..Default::default()
        };
        let env_attribute = Attribute {
            attr_type: AttributeType::Map(AttributeType::String.into()),
            description: Description::plain("Environment used to execute the command"),
            constraint: AttributeConstraint::Optional,
            ..Default::default()
        };
        Some(Schema {
            version: 1,
            block: Block {
                version: 1,
                attributes: map! {
                    "id" => Attribute {
                        attr_type: AttributeType::String,
                        description: Description::plain("Random id for the command"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                    "inputs" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("Execute command locally"),
                        constraint: AttributeConstraint::OptionalComputed,
                        ..Default::default()
                    },
                    "state" => Attribute {
                        attr_type: AttributeType::Map(AttributeType::String.into()),
                        description: Description::plain("State of the resource"),
                        constraint: AttributeConstraint::Computed,
                        ..Default::default()
                    },
                },
                blocks: map! {
                    "read" => NestedBlock::Map(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to get the value of the output",
                        ),
                        ..Default::default()
                    }),
                    "create" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to create the resource",
                        ),
                        ..Default::default()
                    }),
                    "destroy" => NestedBlock::Optional(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                        },
                        description: Description::plain(
                            "Command to execute to destroy the resource",
                        ),
                        ..Default::default()
                    }),
                    "update" => NestedBlock::Set(Block {
                        attributes: map! {
                            "cmd" => cmd_attribute.clone(),
                            "env" => env_attribute.clone(),
                            "triggers" => Attribute {
                                attr_type: AttributeType::Map(AttributeType::String.into()),
                                description: Description::plain(
                                    "What input changes should trigger this update",
                                ),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                            "reloads" => Attribute {
                                attr_type: AttributeType::Map(AttributeType::String.into()),
                                description: Description::plain(
                                    "What outputs should be read again after this update",
                                ),
                                constraint: AttributeConstraint::Optional,
                                ..Default::default()
                            },
                        },
                        description: Description::plain(
                            "Command to execute when an input changes",
                        ),
                        ..Default::default()
                    }),
                    "connection" => NestedBlock::Optional(Block {
                        attributes: T::schema(),
                        description: Description::plain("Connection information"),
                        ..Default::default()
                    }),
                },
                description: Description::plain("Custom resource managed with local commands"),
                deprecated: false,
            },
        })
    }

    async fn validate(&self, diags: &mut Diagnostics, config: Self::State) -> Option<()> {
        let Value::Value(config) = config else {
            return Some(());
        };
        if let Value::Value(connection) = config.connection {
            _ = connection
                .validate(
                    diags,
                    AttributePathStep::Attribute("connection".into()).into(),
                )
                .await?;
        }
        if let Value::Value(create) = config.create {
            _ = create.validate(diags, AttributePathStep::Attribute("create".into()).into())
        }
        if let Value::Value(destroy) = config.destroy {
            _ = destroy.validate(diags, AttributePathStep::Attribute("destroy".into()).into())
        }
        for (name, read) in config.read {
            let mut attr_path: AttributePath = AttributePathStep::Attribute("read".into()).into();
            attr_path.add_key(name);
            if let Value::Value(read) = read {
                _ = read.validate(diags, attr_path)?;
            }
        }
        for (i, update) in config.update.into_iter().enumerate() {
            let mut attr_path: AttributePath = AttributePathStep::Attribute("update".into()).into();
            attr_path.add_index(i as i64);
            if let Value::Value(update) = update {
                _ = update.validate(diags, attr_path)?;
            }
        }
        Some(())
    }

    async fn read(
        &self,
        _diags: &mut Diagnostics,
        state: Self::State,
        private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((state, private_state))
    }

    async fn plan(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        proposed_state: Self::State,
        _config_state: Self::State,
        prior_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(
        Self::State,
        Self::PrivateState,
        Vec<tf_provider::attribute_path::AttributePath>,
    )> {
        Some((proposed_state, prior_private_state, vec![]))
        //Some((State::default().into(), prior_private_state, vec![]))
    }

    async fn apply(
        &self,
        _diags: &mut Diagnostics,
        _prior_state: Self::State,
        planned_state: Self::State,
        _config_state: Self::State,
        planned_private_state: Self::PrivateState,
        _provider_meta_state: Self::ProviderMetaState,
    ) -> Option<(Self::State, Self::PrivateState)> {
        Some((planned_state, planned_private_state))
    }
}