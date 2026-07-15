use crate::persistence::database::{Collection, CollectionFolder, CollectionRequest};
use crate::ui::theme;
use iced::{
    widget::{button, column, container, row, scrollable, text, text_input},
    Alignment, Color, Element, Length, Renderer, Theme,
};
use iced_aw::ContextMenu;
use iced_fonts::lucide;

const INDENT_SIZE: f32 = 16.0;

#[derive(Debug, Clone)]
pub enum Message {
    ToggleCollection(usize),
    ToggleFolder(i32),
    NewCollectionNameChanged(String),
    CreateCollection,
    StartRenameCollection(usize),
    RenameCollectionValueChanged(String),
    ConfirmRenameCollection,
    CancelRenameCollection,
    RequestDeleteCollection(usize),
    ConfirmDeleteCollection(usize),
    CancelDeleteCollection,

    ShowNewFolderInput(i32, Option<i32>),
    HideNewFolderInput,
    NewFolderNameChanged(String),
    CreateFolder,

    StartRenameFolder(i32),
    RenameFolderValueChanged(String),
    ConfirmRenameFolder,
    CancelRenameFolder,
    RequestDeleteFolder(i32),
    ConfirmDeleteFolder(i32),
    CancelDeleteFolder,

    StartRenameRequest(i32),
    RenameRequestValueChanged(String),
    ConfirmRenameRequest,
    CancelRenameRequest,
    RequestDeleteRequest(i32),
    ConfirmDeleteRequest(i32),
    CancelDeleteRequest,
    LoadRequest(i32),

    MoveRequestUp(i32),
    MoveRequestDown(i32),
    StartMoveToFolder(i32),
    MoveToFolder(i32, Option<i32>),
    CancelMoveToFolder,

    ImportCollection,
    ImportCollectionData(Option<String>),
    ImportOpenApi,
    ImportOpenApiData(Option<String>),
    ExportCollection(usize),
    ExportCollectionData(()),
    SaveCurrentRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeItemId {
    Collection(usize),
    Folder(i32),
    Request(i32),
}

#[derive(Debug, Default)]
pub struct CollectionView {
    pub collections: Vec<Collection>,
    pub folders: Vec<CollectionFolder>,
    pub requests: Vec<CollectionRequest>,
    pub expanded_collections: Vec<bool>,
    pub expanded_folders: Vec<bool>,
    pub selected_item: Option<TreeItemId>,
    pub hovered_item: Option<TreeItemId>,
    pub new_collection_name: String,
    pub new_folder_name: String,
    pub new_folder_target: Option<i32>,
    pub new_folder_parent: Option<i32>,
    pub renaming_collection: Option<usize>,
    pub rename_collection_value: String,
    pub renaming_folder: Option<i32>,
    pub rename_folder_value: String,
    pub renaming_request: Option<i32>,
    pub rename_request_value: String,
    pub pending_delete_collection: Option<usize>,
    pub pending_delete_folder: Option<i32>,
    pub pending_delete_request: Option<i32>,
    pub moving_request: Option<i32>,
    pub moving_collection_id: Option<i32>,
}

impl Clone for CollectionView {
    fn clone(&self) -> Self {
        Self {
            collections: self.collections.clone(),
            folders: self.folders.clone(),
            requests: self.requests.clone(),
            expanded_collections: self.expanded_collections.clone(),
            expanded_folders: self.expanded_folders.clone(),
            selected_item: self.selected_item.clone(),
            hovered_item: self.hovered_item.clone(),
            new_collection_name: self.new_collection_name.clone(),
            new_folder_name: self.new_folder_name.clone(),
            new_folder_target: self.new_folder_target,
            new_folder_parent: self.new_folder_parent,
            renaming_collection: self.renaming_collection,
            rename_collection_value: self.rename_collection_value.clone(),
            renaming_folder: self.renaming_folder,
            rename_folder_value: self.rename_folder_value.clone(),
            renaming_request: self.renaming_request,
            rename_request_value: self.rename_request_value.clone(),
            pending_delete_collection: self.pending_delete_collection,
            pending_delete_folder: self.pending_delete_folder,
            pending_delete_request: self.pending_delete_request,
            moving_request: self.moving_request,
            moving_collection_id: self.moving_collection_id,
        }
    }
}

impl CollectionView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, message: Message) -> Option<i32> {
        match message {
            Message::ToggleCollection(idx) => {
                if let Some(expanded) = self.expanded_collections.get_mut(idx) {
                    *expanded = !*expanded;
                }
                self.selected_item = Some(TreeItemId::Collection(idx));
                None
            }
            Message::ToggleFolder(folder_id) => {
                if let Some(f_idx) = self.folders.iter().position(|f| f.id == folder_id) {
                    if let Some(expanded) = self.expanded_folders.get_mut(f_idx) {
                        *expanded = !*expanded;
                    }
                }
                self.selected_item = Some(TreeItemId::Folder(folder_id));
                None
            }
            Message::NewCollectionNameChanged(name) => {
                self.new_collection_name = name;
                None
            }
            Message::CreateCollection => None,
            Message::ShowNewFolderInput(col_id, parent_id) => {
                self.new_folder_target = Some(col_id);
                self.new_folder_parent = parent_id;
                self.new_folder_name.clear();
                None
            }
            Message::HideNewFolderInput => {
                self.new_folder_target = None;
                self.new_folder_parent = None;
                self.new_folder_name.clear();
                None
            }
            Message::NewFolderNameChanged(name) => {
                self.new_folder_name = name;
                None
            }
            Message::CreateFolder => None,
            Message::RequestDeleteCollection(idx) => {
                self.pending_delete_collection = Some(idx);
                None
            }
            Message::ConfirmDeleteCollection(_) => {
                self.pending_delete_collection = None;
                None
            }
            Message::CancelDeleteCollection => {
                self.pending_delete_collection = None;
                None
            }
            Message::RequestDeleteFolder(folder_id) => {
                self.pending_delete_folder = Some(folder_id);
                None
            }
            Message::ConfirmDeleteFolder(_) => {
                self.pending_delete_folder = None;
                None
            }
            Message::CancelDeleteFolder => {
                self.pending_delete_folder = None;
                None
            }
            Message::RequestDeleteRequest(req_id) => {
                self.pending_delete_request = Some(req_id);
                None
            }
            Message::ConfirmDeleteRequest(_) => {
                self.pending_delete_request = None;
                None
            }
            Message::CancelDeleteRequest => {
                self.pending_delete_request = None;
                None
            }
            Message::ImportCollection => None,
            Message::ImportCollectionData(_) => None,
            Message::ImportOpenApi => None,
            Message::ImportOpenApiData(_) => None,
            Message::ExportCollection(_) => None,
            Message::ExportCollectionData(_) => None,
            Message::SaveCurrentRequest => None,
            Message::LoadRequest(req_id) => {
                self.selected_item = Some(TreeItemId::Request(req_id));
                Some(req_id)
            }
            Message::StartRenameCollection(idx) => {
                if let Some(col) = self.collections.get(idx) {
                    self.renaming_collection = Some(idx);
                    self.rename_collection_value = col.name.clone();
                }
                None
            }
            Message::RenameCollectionValueChanged(value) => {
                self.rename_collection_value = value;
                None
            }
            Message::ConfirmRenameCollection => {
                self.renaming_collection = None;
                None
            }
            Message::CancelRenameCollection => {
                self.renaming_collection = None;
                None
            }
            Message::StartRenameFolder(folder_id) => {
                if let Some(folder) = self.folders.iter().find(|f| f.id == folder_id) {
                    self.renaming_folder = Some(folder_id);
                    self.rename_folder_value = folder.name.clone();
                }
                None
            }
            Message::RenameFolderValueChanged(value) => {
                self.rename_folder_value = value;
                None
            }
            Message::ConfirmRenameFolder => {
                self.renaming_folder = None;
                None
            }
            Message::CancelRenameFolder => {
                self.renaming_folder = None;
                None
            }
            Message::StartRenameRequest(req_id) => {
                if let Some(req) = self.requests.iter().find(|r| r.id == req_id) {
                    self.renaming_request = Some(req_id);
                    self.rename_request_value = req.name.clone();
                }
                None
            }
            Message::RenameRequestValueChanged(value) => {
                self.rename_request_value = value;
                None
            }
            Message::ConfirmRenameRequest => {
                self.renaming_request = None;
                None
            }
            Message::CancelRenameRequest => {
                self.renaming_request = None;
                None
            }
            Message::MoveRequestUp(_req_id) => None,
            Message::MoveRequestDown(_req_id) => None,
            Message::StartMoveToFolder(req_id) => {
                if let Some(req) = self.requests.iter().find(|r| r.id == req_id) {
                    self.moving_request = Some(req_id);
                    self.moving_collection_id = Some(req.collection_id);
                }
                None
            }
            Message::MoveToFolder(_req_id, _target_folder_id) => {
                self.moving_request = None;
                self.moving_collection_id = None;
                None
            }
            Message::CancelMoveToFolder => {
                self.moving_request = None;
                self.moving_collection_id = None;
                None
            }
        }
    }

    pub fn sync_collections(&mut self, collections: &[Collection]) {
        self.collections = collections.to_vec();
        if self.expanded_collections.len() < collections.len() {
            self.expanded_collections.resize(collections.len(), false);
        } else {
            self.expanded_collections.truncate(collections.len());
        }
    }

    pub fn sync_folders_for_collection(&mut self, col_id: i32, folders: &[CollectionFolder]) {
        self.folders.retain(|f| f.collection_id != col_id);
        self.folders.extend_from_slice(folders);
        self.folders.sort_by_key(|f| {
            (
                f.collection_id,
                f.parent_folder_id.unwrap_or(0),
                f.sort_order,
            )
        });
        self.expanded_folders.resize(self.folders.len(), false);
    }

    pub fn sync_requests_for_collection(&mut self, col_id: i32, requests: &[CollectionRequest]) {
        self.requests.retain(|r| r.collection_id != col_id);
        self.requests.extend_from_slice(requests);
        self.requests
            .sort_by_key(|r| (r.collection_id, r.folder_id.unwrap_or(0), r.sort_order));
    }

    pub fn folder_index(&self, folder_id: i32) -> Option<usize> {
        self.folders.iter().position(|f| f.id == folder_id)
    }

    pub fn view(&self) -> Element<'_, Message, Theme, Renderer> {
        let header = row![
            text("Collections").size(15),
            button(lucide::plus().size(13)).on_press(Message::CreateCollection),
            button(row![lucide::upload().size(13), text(" Import")].spacing(4))
                .on_press(Message::ImportCollection),
            button(row![lucide::file_code().size(13), text(" OpenAPI")].spacing(4))
                .on_press(Message::ImportOpenApi),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let new_collection_input = text_input("New collection...", &self.new_collection_name)
            .on_input(Message::NewCollectionNameChanged)
            .size(12)
            .padding(4);

        let save_button: Element<'_, Message, Theme, Renderer> = if self.collections.is_empty() {
            button(row![lucide::save().size(13), text(" Save Request").size(12)].spacing(4)).into()
        } else {
            button(row![lucide::save().size(13), text(" Save Request").size(12)].spacing(4))
                .on_press(Message::SaveCurrentRequest)
                .into()
        };

        let mut tree = column![].spacing(1);

        for (col_idx, col) in self.collections.iter().enumerate() {
            let is_expanded = self
                .expanded_collections
                .get(col_idx)
                .copied()
                .unwrap_or(false);
            tree = self.render_collection(tree, col_idx, col, is_expanded);
        }

        if self.collections.is_empty() {
            tree = tree.push(
                text("No collections yet.")
                    .size(12)
                    .color(Color::from_rgb(0.5, 0.5, 0.5)),
            );
        }

        let content = if let Some(moving_req_id) = self.moving_request {
            let col_id = self.moving_collection_id.unwrap_or(0);
            let mut folder_buttons = column![].spacing(2);
            folder_buttons = folder_buttons.push(
                button(
                    row![
                        lucide::corner_down_left().size(11),
                        text(" Root (no folder)").size(11)
                    ]
                    .spacing(4),
                )
                .width(Length::Fill)
                .on_press(Message::MoveToFolder(moving_req_id, None)),
            );
            for folder in &self.folders {
                if folder.collection_id == col_id {
                    let fid = folder.id;
                    let fname = folder.name.clone();
                    let depth = if folder.parent_folder_id.is_some() {
                        "  └ "
                    } else {
                        ""
                    };
                    folder_buttons = folder_buttons.push(
                        button(
                            row![
                                text(depth).size(10),
                                lucide::folder_open().size(11),
                                text(fname).size(11)
                            ]
                            .spacing(4),
                        )
                        .width(Length::Fill)
                        .on_press(Message::MoveToFolder(moving_req_id, Some(fid))),
                    );
                }
            }
            folder_buttons = folder_buttons.push(
                button(
                    text("Cancel")
                        .size(11)
                        .color(Color::from_rgb(0.6, 0.6, 0.6)),
                )
                .width(Length::Fill)
                .on_press(Message::CancelMoveToFolder),
            );
            let picker = container(
                column![
                    text("Move to folder:").size(12),
                    scrollable(folder_buttons).height(Length::Fixed(200.0)),
                ]
                .spacing(4)
                .padding(8),
            )
            .style(|_theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(Color::from_rgb(0.14, 0.14, 0.18))),
                border: iced::Border::default().rounded(4),
                ..iced::widget::container::Style::default()
            })
            .width(Length::Fill);

            column![
                header,
                new_collection_input,
                save_button,
                picker,
                scrollable(tree).height(Length::Fill),
            ]
            .spacing(6)
            .padding(8)
        } else {
            column![
                header,
                new_collection_input,
                save_button,
                scrollable(tree).height(Length::Fill),
            ]
            .spacing(6)
            .padding(8)
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn indent(depth: usize) -> Element<'static, Message, Theme, Renderer> {
        container(text("").width(Length::Fixed((depth as f32) * INDENT_SIZE))).into()
    }

    fn render_collection<'a>(
        &'a self,
        mut tree: iced::widget::Column<'a, Message, Theme, Renderer>,
        col_idx: usize,
        col: &'a Collection,
        is_expanded: bool,
    ) -> iced::widget::Column<'a, Message, Theme, Renderer> {
        let is_renaming = self.renaming_collection == Some(col_idx);
        let is_pending_delete = self.pending_delete_collection == Some(col_idx);

        if is_renaming {
            let rename_row = row![
                text_input("Rename...", &self.rename_collection_value)
                    .on_input(Message::RenameCollectionValueChanged)
                    .size(12)
                    .padding(3),
                button(lucide::check().size(11)).on_press(Message::ConfirmRenameCollection),
                button(lucide::x().size(11)).on_press(Message::CancelRenameCollection),
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            tree = tree.push(rename_row);
        } else {
            let expand_icon: Element<'_, Message, Theme, Renderer> = if is_expanded {
                lucide::chevron_down().size(12).into()
            } else {
                lucide::chevron_right().size(12).into()
            };

            let col_content = row![
                expand_icon,
                lucide::folder().size(12),
                text(&col.name).size(12)
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            let mut actions = row![].spacing(2);
            if is_pending_delete {
                actions = actions
                    .push(
                        button(
                            text("Delete?")
                                .size(10)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .on_press(Message::ConfirmDeleteCollection(col_idx)),
                    )
                    .push(button(lucide::x().size(10)).on_press(Message::CancelDeleteCollection));
            } else {
                actions = actions
                    .push(
                        button(lucide::pencil().size(10))
                            .on_press(Message::StartRenameCollection(col_idx)),
                    )
                    .push(
                        button(lucide::download().size(10))
                            .on_press(Message::ExportCollection(col_idx)),
                    )
                    .push(
                        button(
                            lucide::trash()
                                .size(10)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .on_press(Message::RequestDeleteCollection(col_idx)),
                    );
            }

            let full_row = row![col_content, actions]
                .spacing(4)
                .align_y(Alignment::Center);

            let row_button = button(full_row)
                .width(Length::Fill)
                .on_press(Message::ToggleCollection(col_idx));

            let col_id = col.id;
            let context_menu = ContextMenu::new(row_button, move || {
                container(
                    column![
                        button(
                            text("New Folder")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::ShowNewFolderInput(col_id, None)),
                        button(
                            text("Rename")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::StartRenameCollection(col_idx)),
                        button(
                            text("Export")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::ExportCollection(col_idx)),
                        button(
                            text("Delete")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.85, 0.25, 0.25)))
                                }
                                _ => {
                                    Some(iced::Background::Color(Color::from_rgb(0.70, 0.18, 0.18)))
                                }
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::RequestDeleteCollection(col_idx)),
                    ]
                    .spacing(2)
                    .padding(4),
                )
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.16, 0.16, 0.20))),
                    border: iced::Border::default()
                        .rounded(6)
                        .width(1)
                        .color(Color::from_rgb(0.5, 0.5, 0.9)),
                    ..iced::widget::container::Style::default()
                })
                .width(Length::Fixed(170.0))
                .into()
            });

            tree = tree.push(context_menu);
        }

        if is_expanded {
            // Show new folder input if this collection is the target
            if self.new_folder_target == Some(col.id) && self.new_folder_parent.is_none() {
                let input_row = row![
                    Self::indent(1),
                    text_input("Folder name...", &self.new_folder_name)
                        .on_input(Message::NewFolderNameChanged)
                        .size(12)
                        .padding(3)
                        .width(Length::Fill),
                    button(lucide::check().size(11)).on_press(Message::CreateFolder),
                    button(lucide::x().size(11)).on_press(Message::HideNewFolderInput),
                ]
                .spacing(4)
                .align_y(Alignment::Center);
                tree = tree.push(input_row);
            }

            // Render root folders for this collection
            let col_folders: Vec<(usize, &CollectionFolder)> = self
                .folders
                .iter()
                .enumerate()
                .filter(|(_, f)| f.collection_id == col.id && f.parent_folder_id.is_none())
                .collect();

            for (f_idx, folder) in col_folders {
                tree = self.render_folder_recursive(tree, f_idx, folder, 1);
            }

            // Render root requests for this collection
            let root_requests: Vec<&CollectionRequest> = self
                .requests
                .iter()
                .filter(|r| r.collection_id == col.id && r.folder_id.is_none())
                .collect();

            for req in root_requests {
                tree = self.render_request_item(tree, req, 1);
            }
        }

        tree
    }

    fn render_folder_recursive<'a>(
        &'a self,
        mut tree: iced::widget::Column<'a, Message, Theme, Renderer>,
        f_idx: usize,
        folder: &'a CollectionFolder,
        depth: usize,
    ) -> iced::widget::Column<'a, Message, Theme, Renderer> {
        let is_expanded = self.expanded_folders.get(f_idx).copied().unwrap_or(false);
        let is_renaming = self.renaming_folder == Some(folder.id);
        let is_pending_delete = self.pending_delete_folder == Some(folder.id);

        if is_renaming {
            let rename_row = row![
                Self::indent(depth),
                text_input("Rename...", &self.rename_folder_value)
                    .on_input(Message::RenameFolderValueChanged)
                    .size(12)
                    .padding(3),
                button(lucide::check().size(11)).on_press(Message::ConfirmRenameFolder),
                button(lucide::x().size(11)).on_press(Message::CancelRenameFolder),
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            tree = tree.push(rename_row);
        } else {
            let expand_icon: Element<'_, Message, Theme, Renderer> = if is_expanded {
                lucide::chevron_down().size(11).into()
            } else {
                lucide::chevron_right().size(11).into()
            };

            let folder_content = row![
                Self::indent(depth),
                expand_icon,
                lucide::folder_open().size(11),
                text(&folder.name).size(12),
            ]
            .spacing(3)
            .align_y(Alignment::Center);

            let mut actions = row![].spacing(2);
            if is_pending_delete {
                actions = actions
                    .push(
                        button(
                            text("Delete?")
                                .size(10)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .on_press(Message::ConfirmDeleteFolder(folder.id)),
                    )
                    .push(button(lucide::x().size(10)).on_press(Message::CancelDeleteFolder));
            } else {
                actions = actions
                    .push(
                        button(lucide::pencil().size(10))
                            .on_press(Message::StartRenameFolder(folder.id)),
                    )
                    .push(
                        button(
                            lucide::trash()
                                .size(10)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .on_press(Message::RequestDeleteFolder(folder.id)),
                    );
            }

            let full_row = row![folder_content, actions]
                .spacing(4)
                .align_y(Alignment::Center);

            let row_button = button(full_row)
                .width(Length::Fill)
                .on_press(Message::ToggleFolder(folder.id));

            let folder_id = folder.id;
            let col_id = folder.collection_id;
            let context_menu = ContextMenu::new(row_button, move || {
                container(
                    column![
                        button(
                            text("New Sub-folder")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::ShowNewFolderInput(col_id, Some(folder_id))),
                        button(
                            text("Rename")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::StartRenameFolder(folder_id)),
                        button(
                            text("Delete")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.85, 0.25, 0.25)))
                                }
                                _ => {
                                    Some(iced::Background::Color(Color::from_rgb(0.70, 0.18, 0.18)))
                                }
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::RequestDeleteFolder(folder_id)),
                    ]
                    .spacing(2)
                    .padding(4),
                )
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.16, 0.16, 0.20))),
                    border: iced::Border::default()
                        .rounded(6)
                        .width(1)
                        .color(Color::from_rgb(0.5, 0.5, 0.9)),
                    ..iced::widget::container::Style::default()
                })
                .width(Length::Fixed(170.0))
                .into()
            });

            tree = tree.push(context_menu);
        }

        if is_expanded {
            // Show new sub-folder input if this folder is the target
            if self.new_folder_target == Some(folder.collection_id)
                && self.new_folder_parent == Some(folder.id)
            {
                let input_row = row![
                    Self::indent(depth + 1),
                    text_input("Sub-folder name...", &self.new_folder_name)
                        .on_input(Message::NewFolderNameChanged)
                        .size(12)
                        .padding(3)
                        .width(Length::Fill),
                    button(lucide::check().size(11)).on_press(Message::CreateFolder),
                    button(lucide::x().size(11)).on_press(Message::HideNewFolderInput),
                ]
                .spacing(4)
                .align_y(Alignment::Center);
                tree = tree.push(input_row);
            }

            // Render sub-folders
            let sub_folders: Vec<(usize, &CollectionFolder)> = self
                .folders
                .iter()
                .enumerate()
                .filter(|(_, f)| f.parent_folder_id == Some(folder.id))
                .collect();

            for (sub_f_idx, sub_folder) in sub_folders {
                tree = self.render_folder_recursive(tree, sub_f_idx, sub_folder, depth + 1);
            }

            // Render requests in this folder
            let folder_requests: Vec<&CollectionRequest> = self
                .requests
                .iter()
                .filter(|r| r.folder_id == Some(folder.id))
                .collect();

            for req in folder_requests {
                tree = self.render_request_item(tree, req, depth + 1);
            }
        }

        tree
    }

    fn render_request_item<'a>(
        &'a self,
        mut tree: iced::widget::Column<'a, Message, Theme, Renderer>,
        req: &'a CollectionRequest,
        depth: usize,
    ) -> iced::widget::Column<'a, Message, Theme, Renderer> {
        let method_color = theme::method_color(&req.method);
        let is_renaming = self.renaming_request == Some(req.id);
        let is_pending_delete = self.pending_delete_request == Some(req.id);

        if is_renaming {
            let rename_row = row![
                Self::indent(depth),
                text_input("Rename...", &self.rename_request_value)
                    .on_input(Message::RenameRequestValueChanged)
                    .size(11)
                    .padding(2),
                button(lucide::check().size(10)).on_press(Message::ConfirmRenameRequest),
                button(lucide::x().size(10)).on_press(Message::CancelRenameRequest),
            ]
            .spacing(4)
            .align_y(Alignment::Center);
            tree = tree.push(rename_row);
        } else {
            let req_content = row![
                Self::indent(depth),
                text(&req.method)
                    .size(10)
                    .color(method_color)
                    .font(iced::font::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::font::Font::default()
                    }),
                text(&req.name)
                    .size(11)
                    .color(Color::from_rgb(0.6, 0.75, 1.0)),
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            let mut actions = row![].spacing(2);
            if is_pending_delete {
                actions = actions
                    .push(
                        button(
                            text("Delete?")
                                .size(9)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .style(|_theme, _status| button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb(
                                0.12, 0.12, 0.16,
                            ))),
                            ..button::Style::default()
                        })
                        .on_press(Message::ConfirmDeleteRequest(req.id)),
                    )
                    .push(
                        button(lucide::x().size(9))
                            .style(|_theme, _status| button::Style {
                                background: Some(iced::Background::Color(Color::from_rgb(
                                    0.12, 0.12, 0.16,
                                ))),
                                ..button::Style::default()
                            })
                            .on_press(Message::CancelDeleteRequest),
                    );
            } else {
                actions = actions
                    .push(
                        button(
                            lucide::pencil()
                                .size(9)
                                .color(Color::from_rgb(0.6, 0.75, 1.0)),
                        )
                        .style(|_theme, _status| button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb(
                                0.12, 0.12, 0.16,
                            ))),
                            ..button::Style::default()
                        })
                        .on_press(Message::StartRenameRequest(req.id)),
                    )
                    .push(
                        button(
                            lucide::trash()
                                .size(9)
                                .color(Color::from_rgb(0.8, 0.2, 0.2)),
                        )
                        .style(|_theme, _status| button::Style {
                            background: Some(iced::Background::Color(Color::from_rgb(
                                0.12, 0.12, 0.16,
                            ))),
                            ..button::Style::default()
                        })
                        .on_press(Message::RequestDeleteRequest(req.id)),
                    );
            }

            let full_row = row![req_content, actions]
                .spacing(4)
                .align_y(Alignment::Center);

            let row_button = button(full_row)
                .width(Length::Fill)
                .on_press(Message::LoadRequest(req.id))
                .style(|_theme, _status| button::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.12, 0.12, 0.16))),
                    ..button::Style::default()
                });

            let req_id = req.id;
            let context_menu = ContextMenu::new(row_button, move || {
                container(
                    column![
                        button(
                            text("Move Up")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::MoveRequestUp(req_id)),
                        button(
                            text("Move Down")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::MoveRequestDown(req_id)),
                        button(
                            text("Move to Folder...")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::StartMoveToFolder(req_id)),
                        container(text("").height(Length::Fixed(1.0)))
                            .width(Length::Fill)
                            .style(|_theme| iced::widget::container::Style {
                                background: Some(iced::Background::Color(Color::from_rgb(
                                    0.25, 0.45, 0.80,
                                ))),
                                ..iced::widget::container::Style::default()
                            }),
                        button(
                            text("Rename")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.6, 0.6, 1.0)))
                                }
                                _ => Some(iced::Background::Color(Color::from_rgb(0.5, 0.5, 0.9))),
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::StartRenameRequest(req_id)),
                        button(
                            text("Delete")
                                .size(11)
                                .color(Color::from_rgb(1.0, 1.0, 1.0))
                        )
                        .width(Length::Fill)
                        .style(|_theme, status| {
                            let bg = match status {
                                button::Status::Hovered => {
                                    Some(iced::Background::Color(Color::from_rgb(0.85, 0.25, 0.25)))
                                }
                                _ => {
                                    Some(iced::Background::Color(Color::from_rgb(0.70, 0.18, 0.18)))
                                }
                            };
                            button::Style {
                                background: bg,
                                text_color: Color::from_rgb(1.0, 1.0, 1.0),
                                ..button::Style::default()
                            }
                        })
                        .on_press(Message::RequestDeleteRequest(req_id)),
                    ]
                    .spacing(2)
                    .padding(4),
                )
                .style(|_theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb(0.16, 0.16, 0.20))),
                    border: iced::Border::default()
                        .rounded(6)
                        .width(1)
                        .color(Color::from_rgb(0.5, 0.5, 0.9)),
                    ..iced::widget::container::Style::default()
                })
                .width(Length::Fixed(170.0))
                .into()
            });

            tree = tree.push(context_menu);
        }

        tree
    }
}
