use iced::{
    Alignment, Application, Command, Element, Length, Settings, Subscription,
    executor,
    widget::{Column, Container, Row, Scrollable, Text},
};
use std::time::Duration;
use sysinfo::{CpuExt, ProcessExt, System, SystemExt};

pub fn main() -> iced::Result {
    SystemMonitor::run(Settings::default())
}

struct SystemMonitor {
    system: System,
    cpu_usage: f32,
    memory_usage_mb: f64,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
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
        let memory_usage_mb = system.used_memory() as f64 / 100000.0;

        (
            SystemMonitor {
                system,
                cpu_usage,
                memory_usage_mb,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Rust System Monitor".into()
    }

    fn update(&mut self, _msg: Self::Message) -> Command<Self::Message> {
        self.system.refresh_all();
        self.cpu_usage = self.system.global_cpu_info().cpu_usage();
        self.memory_usage_mb = self.system.used_memory() as f64 / 1000000.0;
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let cpu_count = self.system.cpus().len() as f32;

        let header_info = Text::new(format!(
            "CPU Usage: {:.2}% | Memory Usage: {:.2} MB",
            self.cpu_usage, self.memory_usage_mb
        ))
        .size(22);

        let header_row = Row::new()
            .push(Text::new("Process").width(Length::FillPortion(4)).size(18))
            .push(Text::new("CPU %").width(Length::FillPortion(1)).size(18))
            .push(
                Text::new("Memory (MB)")
                    .width(Length::FillPortion(2))
                    .size(18),
            )
            .align_items(Alignment::Center);

        let mut processes: Vec<_> = self.system.processes().values().collect();
        processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());

        let mut rows = Column::new().spacing(0).align_items(Alignment::Start);

        for process in processes.iter().take(30) {
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

            let row_container = Container::new(row_content).padding([6, 8]);
            rows = rows.push(row_container);
        }

        let scrollable = Scrollable::new(rows)
            .height(Length::Fill)
            .width(Length::Fill);

        let content = Column::new()
            .push(header_info)
            .push(Text::new(
                "-----------------------------------------------------------",
            ))
            .push(header_row)
            .push(scrollable)
            .spacing(8)
            .align_items(Alignment::Start);

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
    }
    //nigga
}
