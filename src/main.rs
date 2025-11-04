use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription, Color,
    executor,
    widget::{Column, Container, Row, Scrollable, Text, Button, canvas, radio},
    widget::container,
};
use iced::widget::canvas::{Canvas, Stroke, Frame, Path};
use sysinfo::{CpuExt, System, SystemExt, ProcessExt, Pid};
use std::time::Duration;

// --- THEME DEFINITIONS ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Theme { Light, Dark }

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

// --- STYLING IMPLEMENTATIONS ---

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
struct RowContainerStyle(Color);
impl container::StyleSheet for RowContainerStyle {
    type Style = iced::Theme;
    fn appearance(&self, _style: &Self::Style) -> container::Appearance {
        container::Appearance { background: Some(self.0.into()), ..Default::default() }
    }
}

// --- MAIN STRUCTS AND ENUMS ---

pub fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

struct SystemMonitor {
    system: System,
    cpu_usage: f32,
    memory_usage_mb: f64,
    screen: Screen,
    cpu_history: Vec<f32>,
    memory_history: Vec<f32>,
    current_theme: Theme,
}

#[derive(Debug, Clone)]
enum Screen { Main, Graph }

#[derive(Debug, Clone)]
enum Message {
    Tick,
    GoToGraph,
    BackToMain,
    ThemeChanged(Theme),
    EndTask(Pid),
}

// --- GRAPH STRUCTS ---

struct CpuGraph {
    history: Vec<f32>,
    current: f32,
    theme: Theme,
}

impl<Message> canvas::Program<Message> for CpuGraph {
    type State = ();
    fn draw(&self, _: &Self::State, renderer: &iced::Renderer, _: &iced::Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor)
        -> Vec<canvas::Geometry> {
        let palette = self.theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let top_offset = 30.0;
        let bottom_margin = 1.0; 
        let chart_height = h - top_offset - bottom_margin;
        let len = (self.history.len().max(1)) as f32;
        let step = w / len;
        
        // Draw the outer border
        frame.stroke(
            &Path::rectangle([0.0, 0.0].into(), bounds.size()),
            Stroke::default().with_width(1.0).with_color(palette.line_separator),
        );
        
        // Draw the baseline (X-Axis) at the bottom of the graph area
        frame.stroke(
            &Path::line(
                [0.0, h - bottom_margin].into(), 
                [w, h - bottom_margin].into()
            ),
            Stroke::default().with_width(1.0).with_color(palette.line_separator),
        );
        
        frame.fill_text(canvas::Text {
            content: format!("CPU: {:.2}%", self.current),
            position: [10.0, 20.0].into(),
            size: iced::Pixels(16.0),
            color: palette.foreground,
            ..Default::default()
        });
        
        // Draw the CPU usage line
        let path = Path::new(|b| {
            for (i, v) in self.history.iter().enumerate() {
                let x = i as f32 * step;
                // Vertical calculation: y = top_offset + (chart_height - (value / max_scale * chart_height))
                // Max scale for CPU is 100.0
                let y = top_offset + (chart_height - (v / 100.0 * chart_height));
                
                if i == 0 { b.move_to([x, y].into()); } else { b.line_to([x, y].into()); }
            }
        });
        frame.stroke(&path, Stroke::default().with_width(2.0).with_color(palette.accent));
        vec![frame.into_geometry()]
    }
}

struct MemGraph {
    history: Vec<f32>,
    current: f32,
    theme: Theme,
}

impl<Message> canvas::Program<Message> for MemGraph {
    type State = ();
    fn draw(&self, _: &Self::State, renderer: &iced::Renderer, _: &iced::Theme, bounds: iced::Rectangle, _: iced::mouse::Cursor)
        -> Vec<canvas::Geometry> {
        let palette = self.theme.palette();
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;
        let top_offset = 30.0;
        let bottom_margin = 1.0; 
        let chart_height = h - top_offset - bottom_margin;
        let max_val = self.history.iter().cloned().fold(1.0_f32, f32::max);
        let len = (self.history.len().max(1)) as f32;
        let step = w / len;
        
        // Draw the outer border
        frame.stroke(
            &Path::rectangle([0.0, 0.0].into(), bounds.size()),
            Stroke::default().with_width(1.0).with_color(palette.line_separator),
        );
        
        // Draw the baseline (X-Axis) at the bottom of the graph area
        frame.stroke(
            &Path::line(
                [0.0, h - bottom_margin].into(), 
                [w, h - bottom_margin].into()
            ),
            Stroke::default().with_width(1.0).with_color(palette.line_separator),
        );
        
        // --- MODIFIED: Removed Max Scale from title content ---
        frame.fill_text(canvas::Text {
            content: format!("Memory: {:.2} MB", self.current),
            position: [10.0, 20.0].into(),
            size: iced::Pixels(16.0),
            color: palette.foreground,
            ..Default::default()
        });
        
        // Draw the Memory usage line
        let path = Path::new(|b| {
            for (i, v) in self.history.iter().enumerate() {
                let x = i as f32 * step;
                // Vertical calculation: y = top_offset + (chart_height - (value / max_scale * chart_height))
                let y = top_offset + (chart_height - (v / max_val * chart_height));
                
                if i == 0 { b.move_to([x, y].into()); } else { b.line_to([x, y].into()); }
            }
        });
        frame.stroke(&path, Stroke::default().with_width(2.0).with_color(palette.accent));
        vec![frame.into_geometry()]
    }
}

// --- APPLICATION IMPLEMENTATION ---

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        system.refresh_all();
        (Self {
            system,
            cpu_usage: 0.0,
            memory_usage_mb: 0.0,
            current_theme: Theme::Dark,
            screen: Screen::Main,
            cpu_history: vec![0.0; 100],
            memory_history: vec![0.0; 100],
        }, Command::none())
    }

    fn title(&self) -> String { "Rust System Monitor".into() }

    fn update(&mut self, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            Message::Tick => {
                self.system.refresh_all();
                self.cpu_usage = self.system.global_cpu_info().cpu_usage();
                self.memory_usage_mb = self.system.used_memory() as f64 / 1000000.0;
                self.cpu_history.push(self.cpu_usage);
                if self.cpu_history.len() > 100 { self.cpu_history.remove(0); }
                self.memory_history.push(self.memory_usage_mb as f32);
                if self.memory_history.len() > 100 { self.memory_history.remove(0); }
            }
            Message::GoToGraph => self.screen = Screen::Graph,
            Message::BackToMain => self.screen = Screen::Main,
            Message::ThemeChanged(theme) => self.current_theme = theme,
            Message::EndTask(pid) => { if let Some(p) = self.system.process(pid) { p.kill(); } }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self.screen {
            Screen::Main => self.main_view(),
            Screen::Graph => self.graph_view(),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
}

// --- VIEW METHODS ---

impl SystemMonitor {
    fn main_view(&self) -> Element<Message> {
        let palette = self.current_theme.palette();
        let cpu_count = self.system.cpus().len() as f32;

        let theme_selection = Row::new()
            .push(Text::new("Theme:").style(palette.foreground))
            .push(radio::Radio::new("Light", Theme::Light, Some(self.current_theme), Message::ThemeChanged))
            .push(radio::Radio::new("Dark", Theme::Dark, Some(self.current_theme), Message::ThemeChanged))
            .spacing(10);

        let header_info = Text::new(
            format!("CPU Usage: {:.2}% | Memory: {:.2} MB", self.cpu_usage, self.memory_usage_mb)
        ).style(palette.accent);

        let button = Button::new(Text::new("View CPU & Memory Graphs")).on_press(Message::GoToGraph);

        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        let mut rows = Column::new().spacing(0);
        let [r, g, b, _] = palette.header_bg.into_rgba8();
        
        // Add Header Row
        let header_row = Row::new()
            .push(Text::new("Action").width(Length::Shrink))
            .push(Text::new("Process Name").width(Length::FillPortion(3)))
            .push(Text::new("CPU (%)").width(Length::FillPortion(1)))
            .push(Text::new("Memory (MB)").width(Length::FillPortion(2)))
            .spacing(10)
            .align_items(Alignment::Center);
        rows = rows.push(
            Container::new(header_row)
            .style(iced::theme::Container::Custom(Box::new(RowContainerStyle(palette.header_bg))))
            .padding(5)
        );
        
        // Add Process Rows
        for (i, process) in processes.iter().take(30).enumerate() {
            let row = Row::new()
                .push(Button::new(Text::new("End")).on_press(Message::EndTask(process.pid())).width(Length::Shrink))
                .push(Text::new(process.name()).width(Length::FillPortion(3)))
                .push(Text::new(format!("{:.2}", process.cpu_usage() / cpu_count)).width(Length::FillPortion(1)))
                .push(Text::new(format!("{:.2}", process.memory() as f64 / 1000000.0)).width(Length::FillPortion(2)))
                .spacing(10)
                .align_items(Alignment::Center);
            
            let bg = if i % 2 == 0 {
                Color::from_rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 0.1) // Slightly lighter background
            } else { Color::TRANSPARENT };
            
            rows = rows.push(
                Container::new(row)
                .style(iced::theme::Container::Custom(Box::new(RowContainerStyle(bg))))
                .padding(5)
            );
        }

        Container::new(
            Column::new()
                .push(theme_selection)
                .push(header_info)
                .push(button)
                .push(Scrollable::new(rows).height(Length::Fill))
                .spacing(10)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(iced::theme::Container::Custom(Box::new(CustomContainerStyle(self.current_theme))))
        .into()
    }

    fn graph_view(&self) -> Element<Message> {
        let palette = self.current_theme.palette();
        let cpu_graph = Canvas::new(CpuGraph {
            history: self.cpu_history.clone(),
            current: self.cpu_usage,
            theme: self.current_theme,
        }).width(Length::Fill).height(Length::Fixed(220.0));

        let mem_graph = Canvas::new(MemGraph {
            history: self.memory_history.clone(),
            current: self.memory_usage_mb as f32,
            theme: self.current_theme,
        }).width(Length::Fill).height(Length::Fixed(220.0));

        let back = Button::new(Text::new("Back")).on_press(Message::BackToMain);

        Container::new(
            Column::new()
                .spacing(20) 
                .align_items(Alignment::Center) 
                .push(Text::new("CPU Usage Graph").size(20).style(palette.accent))
                .push(cpu_graph)
                .push(Text::new("Memory Usage Graph").size(20).style(palette.accent))
                .push(mem_graph)
                .push(
                    Container::new(back)
                        .width(Length::Shrink)
                        .padding(10)
                )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20)
        .style(iced::theme::Container::Custom(Box::new(CustomContainerStyle(self.current_theme))))
        .into()
    }
}