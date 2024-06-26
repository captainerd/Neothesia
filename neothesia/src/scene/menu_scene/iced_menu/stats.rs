use crate::{context::Context, scene::menu_scene::icons, song::Song};
use chrono::{DateTime, Local};
use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_widget::{column as col, container, row, vertical_space};
use neothesia_core::gamesave::SavedStats;
use neothesia_iced_widgets::{BarLayout, Element, Layout, NeoBtn};

use super::{
    centered_text,
    page::{Page, PageMessage},
    Data, Message,
};

#[derive(Debug, Clone)]
pub enum Event {
    GoBack,
    Play,
}

use crate::menu_scene::Step;
pub struct StatsPage;

impl Page for StatsPage {
    type Event = Event;

    fn update(data: &mut Data, event: Event, ctx: &mut Context) -> PageMessage {
        match event {
            Event::GoBack => return PageMessage::go_to_page(Step::Main),
            Event::Play => {
                super::play(data, ctx);
            }
        }

        PageMessage::none()
    }

    fn view<'a>(_data: &'a Data, ctx: &Context) -> Element<'a, Event> {
        let mut songhistory = Vec::new();

        let mut songname = String::new();
        if let Some(song) = ctx.song.as_ref() {
            songname = Song::get_clean_songname(song.file.name.clone())
        }

        // Load saved stats for the current song
        let sorted_stats = SavedStats::load_for_song(songname.clone());

        // Populate data into tracks
        for (index, stats) in sorted_stats.iter().enumerate() {
            let scores = SavedStats::score_cooking(stats);

            //Sort by score, higher score first

            let datetime: DateTime<Local> = stats.date.into();
            let score = (index + 1) as u32;
            let trophy_image = if score <= 3 { score } else { 0 };
            let card = neothesia_iced_widgets::StatsContainer::new()
                .image(trophy_image)
                .date(datetime.format("%d/%m/%y %H:%M:%S").to_string())
                .place(&(index + 1).to_string()) // Index starts from 1
                .score(scores)
                .notes_hits(stats.notes_hit)
                .notes_missed(stats.notes_missed)
                .wrong_notes(stats.wrong_notes)
                .correct_notes_duration(stats.correct_note_times);
            songhistory.push(Vec::<neothesia_iced_widgets::Element<Event>>::from(card));
        }

        //   Collect all elements from songhistory into a single Vec
        let mut all_elements = Vec::new();

        for children in songhistory {
            all_elements.extend(children);
        }

        // Create a Scrollable widget with the collected elements
        let scrollable = iced_widget::Scrollable::new(
            iced_widget::Column::with_children(all_elements)
                .spacing(10)
                .align_items(Alignment::Start),
        )
        .height(ctx.window_state.logical_size.height as u16 - 250);

        let mut elements = Vec::new();
        let scrollable_element: Element<'_, Event> = scrollable.into();

        // Insert the header of the "List" and the Scrollable element "items" into the elements vector for a more list-like responsive UI
        let mut songhistory_header = Vec::new();
        let first_place_card = neothesia_iced_widgets::StatsContainer::new()
            .image(0)
            .date("Date")
            .place("Place")
            .score("Score")
            .notes_hits("Good Hits")
            .notes_missed("Slow Hits")
            .wrong_notes("Wrong notes")
            .correct_notes_duration("Good Durations")
            .header(true);
        songhistory_header.push(Vec::<neothesia_iced_widgets::Element<Event>>::from(
            first_place_card,
        ));

        for children in songhistory_header {
            elements.extend(children);
        }
        elements.push(scrollable_element.into());

        let column = iced_widget::scrollable(iced_widget::column(elements));

        let mut elements = Vec::new();

        let center_text = centered_text(songname)
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let center_text_container = container(center_text)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 25.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });
        elements.push(center_text_container.into());

        elements.push(
            col![vertical_space().height(Length::Fixed(10.0)), column]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .into(),
        );

        let column = iced_widget::scrollable(iced_widget::column(elements));

        let right = {
            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Play);

            if ctx.song.is_some() {
                row![play]
            } else {
                row![]
            }
            .spacing(10)
            .width(Length::Shrink)
            .align_items(Alignment::Center)
        };

        let right = container(right)
            .width(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        let left = {
            let back = NeoBtn::new(
                icons::left_arrow_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::GoBack);

            row![back].align_items(Alignment::Start)
        };

        let left = container(left)
            .width(Length::Fill)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            });

        let center = {
            container(
                centered_text("Hit enter to play again")
                    .size(20)
                    .width(Length::Fill)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            })
        };

        Layout::new()
            .body(column)
            .bottom(BarLayout::new().left(left).center(center).right(right))
            .into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Enter => Some(Message::StatsPage(self::Event::Play)),
                Named::Escape => Some(Message::GoToPage(Step::Main)),
                _ => None,
            },
            _ => None,
        }
    }
}
