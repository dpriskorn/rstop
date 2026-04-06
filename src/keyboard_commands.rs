use crate::color::Colors;

pub struct KeyboardCommands;

impl KeyboardCommands {
    pub fn new() -> Self {
        Self
    }

    pub fn print_line(&self, interval: f64) {
        print!(
            "\nq=quit | p=pause | a=advanced | h=help | m=mem | r=renice | k=kill | interval={:.1}s{}",
            interval,
            Colors::RESET
        );
    }

    fn build_markers(
        &self,
        advanced: bool,
        help: bool,
        paused: bool,
        renice: bool,
        kill: bool,
    ) -> String {
        let mut markers = String::new();
        if advanced {
            markers.push_str(&format!(" {}ADVANCED{}", Colors::CYAN, Colors::RESET));
        }
        if help {
            markers.push_str(&format!(" {}HELP{}", Colors::CYAN, Colors::RESET));
        }
        if paused {
            markers.push_str(&format!(" {}PAUSED{}", Colors::YELLOW, Colors::RESET));
        }
        if renice {
            markers.push_str(&format!(" {}RENICE{}", Colors::YELLOW, Colors::RESET));
        }
        if kill {
            markers.push_str(&format!(" {}KILL{}", Colors::RED, Colors::RESET));
        }
        markers
    }

    pub fn print(
        &self,
        interval: f64,
        advanced: bool,
        help: bool,
        paused: bool,
        renice: bool,
        kill: bool,
    ) {
        self.print_line(interval);
        print!(
            "{}",
            self.build_markers(advanced, help, paused, renice, kill)
        );
    }
}

impl Default for KeyboardCommands {
    fn default() -> Self {
        Self::new()
    }
}
