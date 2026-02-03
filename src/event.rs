use futures_core::Stream;
use pin_project_lite::pin_project;
use tokio::net::UnixStream;
use tokio_util::codec::{Decoder, FramedRead, LinesCodec};

use crate::Error;

#[derive(Debug)]
pub enum Event {
    Workspace {
        workspace_name: String,
    },
    FocusedMon {
        mon_name: String,
        workspace_name: String,
    },
    ActiveWindow {
        window_class: String,
        window_title: String,
    },
    ActiveWindowV2 {
        window_address: String,
    },
    Fullscreen(bool),
    MonitorRemoved {
        monitor_name: String,
    },
    MonitorAdded {
        monitor_name: String,
    },
    CreateWorkspace {
        workspace_name: String,
    },
    DestroyWorkspace {
        workspace_name: String,
    },
    MoveWorkspace {
        workspace_name: String,
        mon_name: String,
    },
    RenameWorkspace {
        workspace_id: String,
        new_name: String,
    },
    ActiveSpecial {
        workspace_name: String,
        mon_name: String,
    },
    ActiveSpecialV2 {
        workspace_id: String,
        workspace_name: String,
        mon_name: String
    },
    ActiveLayout {
        keyboard_name: String,
        layout_name: String,
    },
    OpenWindow {
        window_address: String,
        workspace_name: String,
        window_class: String,
        window_title: String,
    },
    CloseWindow {
        window_address: String,
    },
    MoveWindow {
        window_address: String,
        workspace_name: String,
    },
    OpenLayer {
        namespace: String,
    },
    CloseLayer {
        namespace: String,
    },
    Submap {
        submap_name: String,
    },
    ChangeFloatingMode {
        window_address: String,
        floating: bool,
    },
    Urgent {
        window_address: String,
    },
    Minimize {
        window_address: String,
        minimized: bool,
    },

    Screencast {
        state: String, // FIXME: this are probrably bools but the documentation isn't very clear
        owner: String,
    },
    WindowTitle {
        window_address: String,
    },
    WindowTitleV2 {
        window_address: String,
        window_title: String
    },
    IgnoreGroupLock(bool),
    LockGroups(bool),
}

impl Event {
    pub(crate) fn parse(event: &str, data: &str) -> Result<Self, Error> {
        match event {
            "workspace" => Ok(Self::Workspace {
                workspace_name: data.to_string(),
            }),
            "focusedmon" => {
                let (mon_name, workspace_name) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::FocusedMon {
                    mon_name: mon_name.to_string(),
                    workspace_name: workspace_name.to_string(),
                })
            }
            "activewindow" => {
                let (window_class, window_title) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::ActiveWindow {
                    window_class: window_class.to_string(),
                    window_title: window_title.to_string(),
                })
            }
            "activewindowv2" => Ok(Self::ActiveWindowV2 {
                window_address: data.to_string(),
            }),
            "fullscreen" => Ok(Self::Fullscreen(
                data.parse::<u8>().map_err(|_err| Error::MalformedInput)? != 0,
            )),
            "monitorremoved" => Ok(Self::MonitorRemoved {
                monitor_name: data.to_string(),
            }),
            "monitoradded" => Ok(Self::MonitorAdded {
                monitor_name: data.to_string(),
            }),
            "createworkspace" => Ok(Self::CreateWorkspace {
                workspace_name: data.to_string(),
            }),
            "destroyworkspace" => Ok(Self::DestroyWorkspace {
                workspace_name: data.to_string(),
            }),
            "moveworkspace" => {
                let (workspace_name, mon_name) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::MoveWorkspace {
                    workspace_name: workspace_name.to_string(),
                    mon_name: mon_name.to_string(),
                })
            }
            "renameworkspace" => {
                let (workspace_id, new_name) = data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::RenameWorkspace {
                    workspace_id: workspace_id.to_string(),
                    new_name: new_name.to_string(),
                })
            }
            "activespecial" => {
                let (workspace_name, mon_name) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::ActiveSpecial {
                    workspace_name: workspace_name.to_string(),
                    mon_name: mon_name.to_string(),
                })
            }

            "activespecialv2" => {
                let parts: Vec<&str> = data.split(',').collect();

                let [workspaceid, workspace_name, mon_name] = parts.as_slice()
                    else { return Err(Error::MalformedInput); };

                Ok(Self::ActiveSpecialV2 {
                    workspace_id: workspaceid.to_string(),
                    workspace_name: workspace_name.to_string(),
                    mon_name: mon_name.to_string(),
                })
            }

            "activelayout" => {
                let (keyboard_name, layout_name) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::ActiveLayout {
                    keyboard_name: keyboard_name.to_string(),
                    layout_name: layout_name.to_string(),
                })
            }
            "openwindow" => {
                let data: Vec<&str> = data.splitn(4, ',').collect();

                if data.len() != 4 {
                    Err(Error::MalformedInput)
                } else {
                    unsafe {
                        Ok(Self::OpenWindow {
                            window_address: data.get_unchecked(0).to_string(),
                            workspace_name: data.get_unchecked(1).to_string(),
                            window_class: data.get_unchecked(2).to_string(),
                            window_title: data.get_unchecked(3).to_string(),
                        })
                    }
                }
            }
            "closewindow" => Ok(Self::CloseWindow {
                window_address: data.to_string(),
            }),
            "movewindow" => {
                let (window_address, workspace_name) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::MoveWindow {
                    window_address: window_address.to_string(),
                    workspace_name: workspace_name.to_string(),
                })
            }
            "openlayer" => Ok(Self::OpenLayer {
                namespace: data.to_string(),
            }),
            "closelayer" => Ok(Self::CloseLayer {
                namespace: data.to_string(),
            }),
            "submap" => Ok(Self::Submap {
                submap_name: data.to_string(),
            }),
            "changefloatingmode" => {
                let (window_address, floating) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::ChangeFloatingMode {
                    window_address: window_address.to_string(),
                    floating: floating
                        .parse::<u8>()
                        .map_err(|_err| Error::MalformedInput)?
                        != 0,
                })
            }
            "urgent" => Ok(Self::Urgent {
                window_address: data.to_string(),
            }),
            "minimize" => {
                let (window_address, minimized) =
                    data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::Minimize {
                    window_address: window_address.to_string(),
                    minimized: minimized
                        .parse::<u8>()
                        .map_err(|_err| Error::MalformedInput)?
                        != 0,
                })
            }
            "screencast" => {
                let (state, owner) = data.split_once(',').ok_or(Error::MalformedInput)?;

                Ok(Self::Screencast {
                    state: state.to_string(),
                    owner: owner.to_string(),
                })
            }
            "windowtitle" => Ok(Self::WindowTitle {
                window_address: data.to_string(),
            }),
            "windowtitlev2" => {
                let mut parts = data.splitn(2, ',');

                let window_address = parts.next().ok_or(Error::MalformedInput)?;
                let window_title   = parts.next().ok_or(Error::MalformedInput)?;

                Ok(Self::WindowTitleV2 {
                    window_address: window_address.to_string(),
                    window_title: window_title.to_string(),
                })
            }
                "ignoregrouplock" => Ok(Self::IgnoreGroupLock(
                data.parse::<u8>().map_err(|_err| Error::MalformedInput)? != 0u8,
            )),
            "lockgroups" => Ok(Self::LockGroups(
                data.parse::<u8>().map_err(|_err| Error::MalformedInput)? != 0u8,
            )),
            _ => Err(Error::UnknownEvent(event.to_string(), data.to_string())),
        }
    }
}

struct EventDecoder {
    inner: LinesCodec,
}

impl EventDecoder {
    pub fn new() -> Self {
        Self {
            inner: LinesCodec::new(),
        }
    }
}

impl Decoder for EventDecoder {
    type Item = Event;

    type Error = Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if let Some(line) = self.inner.decode(src)? {
            let (event, data) = line.split_once(">>").ok_or(Error::MalformedInput)?;

            Event::parse(event, data).map(Some)
        } else {
            Ok(None)
        }
    }
}

pin_project! {
    pub struct EventListener {
        #[pin]
        inner: FramedRead<UnixStream, EventDecoder>,
    }
}

impl EventListener {
    pub async fn new() -> Result<Self, Error> {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .map_err(|_err| Error::NoInstanceSignature)?;
        let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
            .map_err(|_| Error::NoInstanceSignature)?;
        let socket2 = UnixStream::connect(format!("{xdg_runtime}/hypr/{his}/.socket2.sock")).await?;

        Ok(Self {
            inner: FramedRead::new(socket2, EventDecoder::new()),
        })
    }
}

impl Stream for EventListener {
    type Item = Result<Event, Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.project().inner.poll_next(cx)
    }
}
