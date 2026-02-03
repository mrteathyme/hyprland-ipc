use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use crate::{params::Window, Error};

pub struct Dispatcher {
    socket_path: String,
}

impl Dispatcher {
    pub fn new() -> Result<Self, Error> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_err| Error::NoInstanceSignature)?;
        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
            .map_err(|_| Error::NoInstanceSignature)?;
        Ok(Self {
            socket_path: format!("{xdg_runtime}/hypr/{his}/.socket.sock"),
        })
    }

    pub async fn call_command(&self, command: &str) -> Result<String, Error> {
        let mut socket = UnixStream::connect(&self.socket_path).await?;

        socket.write_all(command.as_bytes()).await?;

        let mut response = String::new();

        socket.read_to_string(&mut response).await?;

        if response == "ok" {
            Ok(response)
        } else {
            Err(Error::NotOkResponse(response))
        }
    }


    pub async fn call_command_json<T: DeserializeOwned>(&self, command: &str) -> Result<T, Error> {
        let mut socket = UnixStream::connect(&self.socket_path).await?;

        socket.write_all(command.as_bytes()).await?;

        let mut response = String::new();
        socket.read_to_string(&mut response).await?;

        serde_json::from_str(&response).map_err(|_err| Error::MalformedInput)
    }


    pub async fn clients(&self) -> Result<Vec<Client>, Error> {
        self.call_command_json("j/clients").await
    }

    pub async fn toggle_floating(&self, window: Option<Window>) -> Result<String, Error> {
        if let Some(window) = window {
            self.call_command(&format!("/dispatch togglefloating {window}"))
                .await
        } else {
            self.call_command("/dispatch togglefloating").await
        }
    }

    pub async fn pin(&self, window: Option<Window>) -> Result<String, Error> {
        if let Some(window) = window {
            self.call_command(&format!("/dispatch pin {window}")).await
        } else {
            self.call_command("/dispatch pin").await
        }
    }

    pub async fn moveoutofgroup(&self, window: Option<Window>) -> Result<String, Error> {
        if let Some(window) = window {
            self.call_command(&format!("/dispatch moveoutofgroup {window}"))
                .await
        } else {
            self.call_command("/dispatch moveoutofgroup").await
        }
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Client {
    pub address: String,
    pub mapped: bool,
    pub hidden: bool,
    pub at: [u32; 2],
    pub size: [u32; 2],
    pub workspace: Workspace,
    pub floating: bool,
    pub pseudo: bool,
    pub monitor: i64,
    pub class: String,
    pub title: String,
    #[serde(rename = "initialClass")]
    pub initial_class: String,
    #[serde(rename = "initialTitle")]
    pub initial_title: String,
    pub pid: i32,
    pub xwayland: bool,
    pub pinned: bool,
    pub fullscreen: i64,
    #[serde(rename = "fullscreenClient")]
    pub fullscreen_client: i64,
    pub grouped: Vec<String>,
    pub tags: Vec<String>,
    pub swallowing: String,
    #[serde(rename = "focusHistoryID")]
    pub focus_history_id: i64,
    #[serde(rename = "inhibitingIdle")]
    pub inhibiting_idle: bool,
    #[serde(rename = "xdgTag")]
    pub xdg_tag: String,
    #[serde(rename = "xdgDescription")]
    pub xdg_description: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}
