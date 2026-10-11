//! File drops arrive asynchronously in winit 0.31, potentially after `DragLeft`.
use std::{collections::BTreeMap, path::PathBuf};

use winit::{
    data_transfer::{DataTransferId, TypeHint},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, AsyncRequestSerial, DndAction},
};

#[derive(Default)]
pub(crate) struct DragDrop {
    transfers: BTreeMap<DataTransferId, Transfer>,
    hovering: Option<DataTransferId>,
}

struct Transfer {
    serial: AsyncRequestSerial,
    paths: Option<Vec<PathBuf>>,
    dropped: bool,
}

impl DragDrop {
    pub fn on_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: &WindowEvent,
        input: &mut egui::RawInput,
    ) {
        match event {
            WindowEvent::DragEntered { id, .. } => {
                input.hovered_files.clear();
                self.hovering = Some(*id);
                if event_loop
                    .data_transfer(*id)
                    .is_ok_and(|data| data.has_type(&TypeHint::UriList))
                {
                    match event_loop.fetch_data_transfer(*id, &TypeHint::UriList) {
                        Ok(serial) => {
                            self.transfers.insert(
                                *id,
                                Transfer {
                                    serial,
                                    paths: None,
                                    dropped: false,
                                },
                            );
                            if let Err(err) =
                                event_loop.set_valid_dnd_actions(*id, &[DndAction::Copy])
                            {
                                log::debug!("Cannot accept file drag: {err}");
                            }
                        }
                        Err(err) => log::debug!("Cannot fetch dragged files: {err}"),
                    }
                }
            }
            _ => self.on_transfer_event(event, input),
        }
    }

    fn on_transfer_event(&mut self, event: &WindowEvent, input: &mut egui::RawInput) {
        match event {
            WindowEvent::DragDropped { id, .. } => {
                input.hovered_files.clear();
                if let Some(transfer) = self.transfers.get_mut(id) {
                    transfer.dropped = true;
                    self.finish_drop(*id, input);
                }
            }
            WindowEvent::DragLeft { id } => {
                if self.hovering == Some(*id) {
                    self.hovering = None;
                    input.hovered_files.clear();
                }
                // An accepted drop still needs its asynchronous data, even after leaving.
                if self
                    .transfers
                    .get(id)
                    .is_some_and(|transfer| !transfer.dropped)
                {
                    self.transfers.remove(id);
                }
            }
            WindowEvent::DataTransferReceived { id, serial, value } => {
                self.receive_uris(*id, *serial, value.try_as_uris(), input);
            }
            _ => {}
        }
    }

    fn receive_uris(
        &mut self,
        id: DataTransferId,
        serial: AsyncRequestSerial,
        uris: std::io::Result<Vec<String>>,
        input: &mut egui::RawInput,
    ) {
        if let Some(transfer) = self.transfers.get_mut(&id)
            && transfer.serial == serial
        {
            match uris {
                Ok(uris) => {
                    let paths: Vec<_> = uris
                        .iter()
                        .filter_map(|uri| {
                            #[cfg(target_env = "ohos")]
                            {
                                winit::platform::ohos::drag_drop::file_path(uri)
                            }
                            #[cfg(not(target_env = "ohos"))]
                            {
                                url::Url::parse(uri).ok()?.to_file_path().ok()
                            }
                        })
                        .collect();
                    if self.hovering == Some(id) && !transfer.dropped {
                        input.hovered_files = paths
                            .iter()
                            .map(|path| egui::HoveredFile {
                                path: Some(path.clone()),
                                ..Default::default()
                            })
                            .collect();
                    }
                    transfer.paths = Some(paths);
                    self.finish_drop(id, input);
                }
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Deadlock
                    ) => {}
                Err(err) => {
                    log::debug!("Cannot read dragged files: {err}");
                    self.transfers.remove(&id);
                }
            }
        }
    }

    fn finish_drop(&mut self, id: DataTransferId, input: &mut egui::RawInput) {
        if self
            .transfers
            .get(&id)
            .is_some_and(|transfer| transfer.dropped && transfer.paths.is_some())
            && let Some(Transfer {
                paths: Some(paths), ..
            }) = self.transfers.remove(&id)
        {
            input
                .dropped_files
                .extend(paths.into_iter().map(|path| egui::DroppedFile {
                    path: Some(path),
                    ..Default::default()
                }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending() -> (DragDrop, DataTransferId, AsyncRequestSerial) {
        let id = DataTransferId::from_raw(1);
        let serial = AsyncRequestSerial::get();
        let mut state = DragDrop {
            hovering: Some(id),
            ..Default::default()
        };
        state.transfers.insert(
            id,
            Transfer {
                serial,
                paths: None,
                dropped: false,
            },
        );
        (state, id, serial)
    }

    #[test]
    fn drop_data_can_arrive_after_leaving_and_is_delivered_once() {
        let (mut state, id, serial) = pending();
        let mut input = egui::RawInput::default();
        state.on_transfer_event(
            &WindowEvent::DragDropped {
                id,
                proposed_action: None,
            },
            &mut input,
        );
        state.on_transfer_event(&WindowEvent::DragLeft { id }, &mut input);
        let uris = vec![
            "file:///tmp/a%20b.txt".to_owned(),
            "https://example.com/not-a-file".to_owned(),
        ];
        state.receive_uris(id, serial, Ok(uris.clone()), &mut input);
        state.receive_uris(id, serial, Ok(uris), &mut input);
        assert_eq!(input.dropped_files.len(), 1);
        assert_eq!(
            input.dropped_files[0]
                .path
                .as_ref()
                .and_then(|p| p.file_name()),
            Some(std::ffi::OsStr::new("a b.txt"))
        );
        assert!(input.hovered_files.is_empty());
    }

    #[test]
    fn cancelled_drag_discards_late_data() {
        let (mut state, id, serial) = pending();
        let mut input = egui::RawInput::default();
        state.on_transfer_event(&WindowEvent::DragLeft { id }, &mut input);
        state.receive_uris(
            id,
            serial,
            Ok(vec!["file:///tmp/a.txt".to_owned()]),
            &mut input,
        );
        assert!(input.hovered_files.is_empty());
        assert!(input.dropped_files.is_empty());
        assert!(state.transfers.is_empty());
    }
}
