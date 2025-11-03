use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    executor,
    widget::{Column, Container, Row, Scrollable, Text, Button, canvas}
};
use iced::widget::canvas::{Stroke,Frame};
use iced::Color;
use sysinfo::{CpuExt, System, SystemExt, ProcessExt};
use std::time::Duration;

pub fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

struct SystemMonitor {
    system: System,
    cpu_usage: f32,
    memory_usage_mb: f64,
    screen: Screen,
    cpu_history: Vec<f32>,
}

#[derive(Debug, Clone)]
enum Screen {
    Main,
    Graph,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
    GoToGraph,
    BackToMain,
}

impl Application for SystemMonitor {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut system = System::new_all();
        system.refresh_all();
        (
            SystemMonitor {
                system,
                cpu_usage: 0.0,
                memory_usage_mb: 0.0,
                screen: Screen::Main,
                cpu_history: vec![0.0; 100],
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
                self.memory_usage_mb = self.system.used_memory() as f64 / 1_000_000.0;
                self.cpu_history.push(self.cpu_usage);
                if self.cpu_history.len() > 100 {
                    self.cpu_history.remove(0);
                }
            }
            Message::GoToGraph => self.screen = Screen::Graph,
            Message::BackToMain => self.screen = Screen::Main,
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

impl SystemMonitor {
    fn main_view(&self) -> Element<Message> {
        let header = Text::new(format!(
            "CPU: {:.2}%  |  Memory: {:.2} MB",
            self.cpu_usage, self.memory_usage_mb
        )).size(22);

        let button = Button::new(Text::new("View CPU Graph"))
            .on_press(Message::GoToGraph)
            .padding(10);

        let header_row = Row::new()
            .push(Text::new("Process").width(Length::FillPortion(4)))
            .push(Text::new("CPU %").width(Length::FillPortion(1)))
            .push(Text::new("Memory (MB)").width(Length::FillPortion(2)))
            .align_items(Alignment::Center);

        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));

        let mut rows = Column::new().spacing(0);
        let cpu_count = self.system.cpus().len() as f32;

        for process in processes.iter().take(30) {
            let row = Row::new()
                .push(Text::new(process.name().to_string()).width(Length::FillPortion(4)))
                .push(Text::new(format!("{:.2}", process.cpu_usage() / cpu_count)).width(Length::FillPortion(1)))
                .push(Text::new(format!("{:.2}", process.memory() as f64 / 1_000_000.0)).width(Length::FillPortion(2)))
                .align_items(Alignment::Center);
            rows = rows.push(Container::new(row).padding([6, 8]));
        }

        let scroll = Scrollable::new(rows).height(Length::Fill);

        let content = Column::new()
            .push(header)
            .push(button)
            .push(Text::new("----------------------------------------------------"))
            .push(header_row)
            .push(scroll)
            .spacing(8);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    fn graph_view(&self) -> Element<Message> {
        let back = Button::new(Text::new("Back"))
            .on_press(Message::BackToMain)
            .padding(10);

        let graph = canvas::Canvas::new(CpuGraph { 
            history: self.cpu_history.clone(),
            current: self.cpu_usage,  
        })
        .width(Length::Fill)
        .height(Length::Fixed(250.0));


        let content = Column::new()
            .push(Text::new("CPU Usage Graph").size(28))
            .push(graph)
            .push(back)
            .align_items(Alignment::Center)
            .spacing(20);

        Container::new(content)
            .center_x()
            .center_y()
            .into()
    }
}

struct CpuGraph {
    history: Vec<f32>,
    current: f32,
}

impl<Message> canvas::Program<Message> for CpuGraph {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let w = bounds.width;
        let h = bounds.height;

        let border = canvas::Path::rectangle([0.0, 0.0].into(), bounds.size());
        frame.stroke(
            &border,
            Stroke::default()
                .with_width(2.0)
                .with_color(Color::from_rgb(0.6, 0.6, 0.6)),
        );

        frame.fill_text(canvas::Text {
            content: format!("CPU Usage: {:.2}%", self.current),
            position: [10.0, 20.0].into(),
            size: iced::Pixels(18.0),
            color: Color::BLACK,
            ..Default::default()
        });

        let top_offset = 30.0;
        let graph_height = h - top_offset;
        let step = w / self.history.len() as f32;

        let path = canvas::Path::new(|b| {
            for (i, v) in self.history.iter().enumerate() {
                let x = i as f32 * step;
                let y = top_offset + (graph_height - (v / 100.0 * graph_height));

                if i == 0 {
                    b.move_to([x, y].into());
                } else {
                    b.line_to([x, y].into());
                }
            }
        });

        frame.stroke(
            &path,
            Stroke::default()
                .with_width(2.0)
                .with_color(Color::from_rgb(0.0, 0.0, 0.9)),
        );

        vec![frame.into_geometry()]
    }
}


