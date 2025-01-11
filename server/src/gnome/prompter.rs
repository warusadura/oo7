// org.gnome.keyring.Prompter
// https://gitlab.gnome.org/GNOME/gcr/-/blob/main/gcr/org.gnome.keyring.Prompter.xml

use clap::error::Result;
use oo7::dbus::ServiceError;
use serde::{Deserialize, Serialize};
use zbus::{
    interface, proxy,
    zvariant::{DeserializeDict, OwnedObjectPath, OwnedValue, SerializeDict, Type, Value},
};

use crate::{
    gnome::secret_exchange::SecretExchange,
    prompt::{Prompt, PromptRole},
    service::Service,
};

#[derive(Debug, DeserializeDict, SerializeDict, Type)]
#[zvariant(signature = "a{sv}")]
// System prompt properties
pub struct Properties {
    title: Option<String>,
    #[zvariant(rename = "choice-label")]
    choice_label: Option<String>,
    description: Option<String>,
    message: Option<String>,
    #[zvariant(rename = "caller-window")]
    caller_window: Option<String>,
    warning: Option<String>,
    #[zvariant(rename = "password-new")]
    password_new: Option<bool>,
    #[zvariant(rename = "password-strength")]
    password_strength: Option<u32>,
    #[zvariant(rename = "choice-chosen")]
    choice_chosen: Option<bool>,
    #[zvariant(rename = "continue-label")]
    continue_label: Option<String>,
    #[zvariant(rename = "cancel-label")]
    cancel_label: Option<String>,
}

impl Properties {
    fn for_lock() -> Self {
        Self {
            title: Some("Lock Keyring".to_string()),
            choice_label: None,
            description: Some("Confirm locking 'login' Keyring".to_string()),
            message: Some("Lock Keyring".to_string()),
            caller_window: None,
            warning: None,
            password_new: Some(false),
            password_strength: Some(0),
            choice_chosen: Some(false),
            continue_label: Some("Lock".to_string()),
            cancel_label: Some("Cancel".to_string()),
        }
    }
}

#[derive(Debug, Type)]
#[zvariant(signature = "s")]
// Possible values for PromptReady reply parameter
pub enum Reply {
    No,
    Yes,
    Empty,
}

const NO: &str = "no";
const YES: &str = "yes";
const EMPTY: &str = "";

impl Serialize for Reply {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::No => str::serialize(NO, serializer),
            Self::Yes => str::serialize(YES, serializer),
            Self::Empty => str::serialize(EMPTY, serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Reply {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            NO => Ok(Self::No),
            YES => Ok(Self::Yes),
            EMPTY => Ok(Self::Empty),
            err => Err(serde::de::Error::custom(format!("Invalid reply {err}"))),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Type)]
#[serde(rename_all = "lowercase")]
#[zvariant(signature = "s")]
// Possible values for PerformPrompt type parameter
pub enum PromptType {
    Confirm,
    Password,
}

// org.gnome.keyring.internal.Prompter

#[proxy(
    default_service = "org.gnome.keyring.SystemPrompter",
    interface = "org.gnome.keyring.internal.Prompter",
    default_path = "/org/gnome/keyring/Prompter"
)]
pub trait Prompter {
    fn begin_prompting(&self, callback: &OwnedObjectPath) -> Result<(), ServiceError>;

    fn perform_prompt(
        &self,
        callback: OwnedObjectPath,
        type_: PromptType,
        properties: Properties,
        exchange: &str,
    ) -> Result<(), ServiceError>;

    fn stop_prompting(&self, callback: OwnedObjectPath) -> Result<(), ServiceError>;
}

// org.gnome.keyring.internal.Prompter.Callback

pub struct PrompterCallback {
    service: Service,
    path: OwnedObjectPath,
}

#[interface(name = "org.gnome.keyring.internal.Prompter.Callback")]
impl PrompterCallback {
    pub async fn prompt_ready(
        &self,
        reply: Reply,
        _properties: Properties,
        exchange: &str,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> Result<(), ServiceError> {
        let Some(prompt) = self.service.prompt().await else {
            return Err(ServiceError::NoSuchObject(
                "Prompt does not exist.".to_string(),
            ));
        };

        match prompt.role() {
            PromptRole::Lock => {
                match reply {
                    Reply::Empty => {
                        // First PromptReady call
                        let secret_exchange = SecretExchange::new().map_err(|err| {
                            ServiceError::ZBus(zbus::Error::FDO(Box::new(
                                zbus::fdo::Error::Failed(format!(
                                    "Failed to generate SecretExchange {err}."
                                )),
                            )))
                        })?;
                        let exchange = secret_exchange.begin();

                        let properties = Properties::for_lock();
                        let path = self.path.clone();

                        tokio::spawn(PrompterCallback::perform_prompt(
                            connection.clone(),
                            path,
                            PromptType::Confirm,
                            properties,
                            exchange,
                        ));
                    }
                    Reply::No => {
                        // Second PromptReady call and the prompt is dismissed
                        tracing::debug!("Prompt is being dismissed.");

                        tokio::spawn(PrompterCallback::stop_prompting(
                            connection.clone(),
                            self.path.clone(),
                        ));

                        let signal_emitter = self.service.signal_emitter(prompt.path().clone())?;
                        let result = Value::new::<Vec<OwnedObjectPath>>(vec![])
                            .try_to_owned()
                            .unwrap();

                        tokio::spawn(PrompterCallback::prompt_completed(
                            signal_emitter,
                            true,
                            result,
                        ));
                    }
                    Reply::Yes => {
                        // Second PromptReady call with the final exchange
                        let service = self.service.clone();
                        let objects = prompt.objects().clone();
                        let result = Value::new(&objects).try_to_owned().unwrap();

                        tokio::spawn(async move {
                            let _ = service.set_locked(true, &objects, true).await;
                        });

                        tokio::spawn(PrompterCallback::stop_prompting(
                            connection.clone(),
                            self.path.clone(),
                        ));

                        let signal_emitter = self.service.signal_emitter(prompt.path().clone())?;

                        tokio::spawn(PrompterCallback::prompt_completed(
                            signal_emitter,
                            false,
                            result,
                        ));
                    }
                }
            }
            PromptRole::Unlock => todo!(),
            PromptRole::CreateCollection => todo!(),
        };

        Ok(())
    }

    pub async fn prompt_done(
        &self,
        #[zbus(object_server)] object_server: &zbus::ObjectServer,
    ) -> Result<(), ServiceError> {
        if let Some(prompt) = self.service.prompt().await {
            object_server.remove::<Prompt, _>(prompt.path()).await?;
            self.service.remove_prompt().await;
        }
        object_server.remove::<Self, _>(&self.path).await?;

        Ok(())
    }
}

impl PrompterCallback {
    pub async fn new(service: Service) -> Self {
        let index = service.prompt_index().await;
        Self {
            path: OwnedObjectPath::try_from(format!("/org/gnome/keyring/Prompt/p{index}")).unwrap(),
            service,
        }
    }

    pub fn path(&self) -> &OwnedObjectPath {
        &self.path
    }

    pub async fn perform_prompt(
        connection: zbus::Connection,
        path: OwnedObjectPath,
        prompt_type: PromptType,
        properties: Properties,
        exchange: String,
    ) -> Result<(), ServiceError> {
        let prompter = PrompterProxy::new(&connection).await?;
        prompter
            .perform_prompt(path, prompt_type, properties, &exchange)
            .await?;

        Ok(())
    }

    pub async fn stop_prompting(
        connection: zbus::Connection,
        path: OwnedObjectPath,
    ) -> Result<(), ServiceError> {
        let prompter = PrompterProxy::new(&connection).await?;
        prompter.stop_prompting(path).await?;

        Ok(())
    }

    pub async fn prompt_completed(
        signal_emitter: zbus::object_server::SignalEmitter<'_>,
        dismissed: bool,
        result: OwnedValue,
    ) -> Result<(), ServiceError> {
        Prompt::completed(&signal_emitter, dismissed, result).await?;
        tracing::debug!("Prompt completed.");

        Ok(())
    }
}
