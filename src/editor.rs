use super::*;

#[derive(Debug, Clone)]
pub enum EditorMessage {
    SetName(String),

    NewSource,
    Source(usize, path::Message),
    DelSource(usize),

    NewExclude,
    SetExclude(usize, String),
    DelExclude(usize),

    // Meant for outside
    /// Save button pressed
    Save,
    /// Cancel button pressed
    Cancel,
}

#[derive(Default)]
pub struct Editor {
    pub target: Target,
    pub error: Option<String>,

    s_name: text_input::State,
    s_new_source: button::State,
    s_new_exclude: button::State,
    s_save_button: button::State,
    s_cancel_button: button::State,

    s_exclude: Vec<text_input::State>,
    s_delete_exclude_button: Vec<button::State>,

    s_source: Vec<FilePicker>,
    s_delete_source_button: Vec<button::State>,

    s_scrollable: scrollable::State,
}
impl Editor {
    pub fn with_target(target: Target) -> Self {
        Self {
            // Review; One must manually make sure that the lists of states have the same length as
            // thet lists of values (or other state lists)
            s_exclude: vec![Default::default(); target.excludes.len()],
            s_delete_exclude_button: vec![Default::default(); target.excludes.len()],
            s_source: vec![Default::default(); target.sources.len()],
            s_delete_source_button: vec![Default::default(); target.sources.len()],
            target,
            ..Default::default()
        }
    }
    pub fn view(&mut self) -> Element<'_, EditorMessage> {
        let mut x = Column::new()
            .padding(20)
            .spacing(20)
            // .align_items(Align::Center)
            .push(
                Row::new().spacing(8).push(Icon::Folder.h3()).push(
                    TextInput::new(
                        &mut self.s_name,
                        "Name",
                        &self.target.name,
                        EditorMessage::SetName,
                    )
                    .style(style::TextInput)
                    .size(H3_SIZE),
                ),
            )
            // Sources
            .push(
                Container::new({
                    let mut col = Column::new().push(
                        Row::new().spacing(20).push(h3("Sources")).push(
                            // TODO: icon button
                            Button::new(&mut self.s_new_source, Icon::New.text())
                                .padding(4)
                                .style(style::Button::Icon {
                                    hover_color: Color::WHITE,
                                })
                                .on_press(EditorMessage::NewSource),
                        ),
                    );
                    for (i, (source, del_button, file_picker)) in izip!(
                        &self.target.sources,
                        &mut self.s_delete_source_button,
                        &mut self.s_source
                    )
                    .enumerate()
                    {
                        col = col.push(
                            Row::new()
                                .push(
                                    file_picker
                                        .view(source.as_ref().map(|x| x.as_path()), TEXT_SIZE)
                                        .map(move |msg| EditorMessage::Source(i, msg)),
                                )
                                .push(
                                    Button::new(del_button, Icon::Delete.text())
                                        .on_press(EditorMessage::DelSource(i))
                                        .padding(0)
                                        .style(style::Button::Icon {
                                            hover_color: Color::from_rgb(0.7, 0.2, 0.2),
                                        }),
                                ),
                        );
                    }
                    col
                })
                .width(Length::FillPortion(1)),
            )
            // Excludes
            .push(
                Container::new(
                    Column::new()
                        .push(
                            Row::new().spacing(20).push(h3("Excludes")).push(
                                Button::new(&mut self.s_new_exclude, Icon::New.text())
                                    .style(style::Button::Icon {
                                        hover_color: Color::WHITE,
                                    })
                                    .padding(BUTTON_PAD)
                                    .on_press(EditorMessage::NewExclude),
                            ),
                        )
                        .push(
                            self.target
                                .excludes
                                .iter_mut()
                                .zip(self.s_exclude.iter_mut())
                                .zip(self.s_delete_exclude_button.iter_mut())
                                .enumerate()
                                .fold(
                                    Column::new(),
                                    |column, (i, ((exclude, state), del_button))| {
                                        column.push(
                                            Row::new()
                                                .push(
                                                    TextInput::new(
                                                        state,
                                                        "Exclude string",
                                                        exclude,
                                                        move |s| EditorMessage::SetExclude(i, s),
                                                    )
                                                    .style(style::TextInput)
                                                    .size(TEXT_SIZE),
                                                )
                                                .push(
                                                    Button::new(del_button, Icon::Delete.text())
                                                        .on_press(EditorMessage::DelExclude(i))
                                                        .padding(0)
                                                        .style(style::Button::Icon {
                                                            hover_color: Color::from_rgb(
                                                                0.7, 0.2, 0.2,
                                                            ),
                                                        }),
                                                ),
                                        )
                                    },
                                ),
                        ),
                )
                .width(Length::FillPortion(1)),
            )
            .push(
                Container::new(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                &mut self.s_cancel_button,
                                Text::new("CANCEL").size(TEXT_SIZE - 4),
                            )
                            .padding(8)
                            .style(style::Button::Text)
                            .on_press(EditorMessage::Cancel),
                        )
                        .push(
                            Button::new(
                                &mut self.s_save_button,
                                Text::new("SAVE").size(TEXT_SIZE - 4),
                            )
                            .padding(8)
                            .style(style::Button::Primary)
                            .on_press(EditorMessage::Save),
                        ),
                )
                .width(Length::Fill)
                .align_x(Align::End),
            );
        if let Some(ref error) = self.error {
            x = x.push(Text::new(error).color(Color::from_rgb(0.5, 0.0, 0.0)))
        }
        let x = Container::new(x)
            .style(style::EditorContainer)
            .width(Length::Fill)
            .max_width(1000)
            .height(Length::Shrink);
        let x = Scrollable::new(&mut self.s_scrollable).push(x);
        x.into()
    }
    pub fn update(&mut self, message: EditorMessage) -> Command<EditorMessage> {
        match message {
            EditorMessage::SetName(name) => self.target.name = name,
            EditorMessage::NewSource => {
                self.target.sources.push(Default::default());
                self.s_delete_source_button.push(Default::default());
                // Review; I forgot once to put the following line here
                // Makes the UI malfunction due to how I izip! the iterators
                self.s_source.push(Default::default());
            }
            EditorMessage::Source(i, msg) => {
                if let path::Message::Path(ref path) = msg {
                    self.target.sources[i] = Some(path.clone());
                }
                return self.s_source[i]
                    .update(msg)
                    .map(move |msg| EditorMessage::Source(i, msg));
            }
            EditorMessage::DelSource(i) => {
                self.target.sources.remove(i);
            }
            EditorMessage::NewExclude => {
                self.target.excludes.push(Default::default());
                self.s_exclude.push(Default::default());
                self.s_delete_exclude_button.push(Default::default());
            }
            EditorMessage::SetExclude(i, exclude) => self.target.excludes[i] = exclude,
            EditorMessage::DelExclude(i) => {
                self.target.excludes.remove(i);
            }
            EditorMessage::Save => {
                // Show eventual error message
                if let Err(error) = verify_target(&self.target) {
                    self.error = Some(error);
                }
            }
            EditorMessage::Cancel => (),
        }
        Command::none()
    }
}
