use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription, Color, Border,
    executor,
    widget::{Column, Container, Row, Scrollable, Text, radio},
    widget::container,
};
use std::time::Duration;
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme {
    Light,
    Dark,
}

struct ThemePalette {
    background: Color,
    foreground: Color,
    header_bg: Color,
    accent: Color,
    line_separator: Color,
}

impl Theme {
    fn palette(&self) -> ThemePalette {
        match self {
            Theme::Light => ThemePalette {
                background: Color::from_rgb(0.95, 0.95, 0.95),
                foreground: Color::BLACK,
                header_bg: Color::from_rgb(0.80, 0.80, 0.80),
                accent: Color::from_rgb(0.1, 0.5, 0.8),
                line_separator: Color::from_rgb(0.6, 0.6, 0.6),
            },
            Theme::Dark => ThemePalette {
                background: Color::from_rgb(0.15, 0.15, 0.15),
                foreground: Color::WHITE,
                header_bg: Color::from_rgb(0.25, 0.25, 0.25),
                accent: Color::from_rgb(0.0, 0.7, 1.0),
                line_separator: Color::from_rgb(0.4, 0.4, 0.4),
            },
        }
    }
}

struct CustomContainerStyle(Theme);

impl container::StyleSheet for CustomContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let palette = self.0.palette();
        container::Appearance {
            background: Some(palette.background.into()),
            text_color: Some(palette.foreground),
            ..Default::default()
        }
    }
}

struct HeaderContainerStyle(Theme);

impl container::StyleSheet for HeaderContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        let palette = self.0.palette();
        container::Appearance {
            background: Some(palette.header_bg.into()),
            border: Border::with_radius(4.0),
            ..Default::default()
        }
    }
}

struct RowContainerStyle(Color);

impl container::StyleSheet for RowContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(self.0.into()),
            ..Default::default()
        }
    }
}

pub fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

struct SystemMonitor {
    system: System,
    cpu_usage: f32,
    memory_usage_mb: f64,
    current_theme: Theme,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    ThemeChanged(Theme),
}

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage();
        let memory_usage_mb = system.used_memory() as f64 / 1000000.0;

        (
            SystemMonitor {
                system,
                cpu_usage,
                memory_usage_mb,
                current_theme: Theme::Dark,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Rust System Monitor".into()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            Message::Tick => {
                self.system.refresh_all();
                self.cpu_usage = self.system.global_cpu_info().cpu_usage();
                self.memory_usage_mb = self.system.used_memory() as f64 / 1000000.0;
            }
            Message::ThemeChanged(new_theme) => {
                self.current_theme = new_theme;
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let palette = self.current_theme.palette();
        let cpu_count = self.system.cpus().len() as f32;

        let theme_selection = Row::with_children(vec![
            Text::new("Theme:").size(16).style(palette.foreground).into(),
            radio::Radio::new(
                "Light",
                Theme::Light,
                Some(self.current_theme),
                Message::ThemeChanged,
            ).into(),
            radio::Radio::new(
                "Dark",
                Theme::Dark,
                Some(self.current_theme),
                Message::ThemeChanged,
            ).into(),
        ])
        .spacing(15)
        .padding([0, 0, 15, 0])
        .align_items(Alignment::Center);

        let header_info = Text::new(format!(
            "CPU Usage: {:.2}% | Memory Usage: {:.2} MB",
            self.cpu_usage, self.memory_usage_mb
        ))
        .size(24)
        .style(palette.accent);

        let header_row = Container::new(
            Row::new()
                .push(Text::new("Process").width(Length::FillPortion(4)).size(18).style(palette.foreground))
                .push(Text::new("CPU %").width(Length::FillPortion(1)).size(18).style(palette.foreground))
                .push(
                    Text::new("Memory (MB)")
                        .width(Length::FillPortion(2))
                        .size(18)
                        .style(palette.foreground),
                )
                .align_items(Alignment::Center)
        )
        .padding(8)
        .style(iced::theme::Container::Custom(Box::new(HeaderContainerStyle(self.current_theme))));


        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        let mut rows = Column::new().spacing(0).align_items(Alignment::Start);
        
        let [header_r_u8, header_g_u8, header_b_u8, _] = palette.header_bg.into_rgba8();

        let header_r = header_r_u8 as f32 / 255.0;
        let header_g = header_g_u8 as f32 / 255.0;
        let header_b = header_b_u8 as f32 / 255.0;

        for (i, process) in processes.iter().take(30).enumerate() {
            let name = process.name();
            let cpu = process.cpu_usage() / cpu_count; 
            let mem_mb = process.memory() as f64 / 1000000.0;

            let row_content = Row::new()
                .push(
                    Text::new(name.to_string())
                        .width(Length::FillPortion(4))
                        .size(16),
                )
                .push(
                    Text::new(format!("{:.2}", cpu))
                        .width(Length::FillPortion(1))
                        .size(16),
                )
                .push(
                    Text::new(format!("{:.2}", mem_mb))
                        .width(Length::FillPortion(2))
                        .size(16),
                )
                .align_items(Alignment::Center);

            let row_bg_color = if i % 2 == 0 {
                Color::from_rgba(header_r, header_g, header_b, 0.3)
            } else {
                Color::TRANSPARENT
            };
            
            let row_container = Container::new(row_content)
                .padding([4, 8])
                .style(iced::theme::Container::Custom(Box::new(RowContainerStyle(row_bg_color))));
                
            rows = rows.push(row_container);
        }

        let scrollable = Scrollable::new(rows)
            .height(Length::Fill)
            .width(Length::Fill);

        let content = Column::new()
            .push(theme_selection)
            .push(header_info)
            .push(Text::new("-----------------------------------------------------------").style(palette.line_separator))
            .push(header_row)
            .push(scrollable)
            .spacing(10)
            .align_items(Alignment::Start);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(CustomContainerStyle(self.current_theme))))
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}