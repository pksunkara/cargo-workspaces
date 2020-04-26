use atty::{is, Stream};
use std::io::{Result, Write};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

fn is_a_tty(use_stderr: bool) -> bool {
    let stream = if use_stderr {
        Stream::Stderr
    } else {
        Stream::Stdout
    };

    is(stream)
}

macro_rules! color {
    ($_self:ident, $name:ident, $c:expr) => {
        pub fn $name(&mut $_self, msg: &str) -> Result<()> {
            $_self.stream.set_color($c)?;
            $_self.stream.write_all(msg.as_bytes())?;
            $_self.stream.reset()
        }
    };
}

pub struct Writer {
    stream: StandardStream,
}

impl Writer {
    pub fn new(use_stderr: bool) -> Self {
        let choice = if is_a_tty(use_stderr) {
            ColorChoice::Auto
        } else {
            ColorChoice::Never
        };

        let stream = if use_stderr {
            StandardStream::stderr(choice)
        } else {
            StandardStream::stdout(choice)
        };

        Self { stream }
    }

    pub fn none(&mut self, msg: &str) -> Result<()> {
        self.stream.write_all(msg.as_bytes())
    }

    color!(self, red, ColorSpec::new().set_fg(Some(Color::Red)));
    color!(self, green, ColorSpec::new().set_fg(Some(Color::Green)));
    color!(self, yellow, ColorSpec::new().set_fg(Some(Color::Yellow)));
    color!(self, cyan, ColorSpec::new().set_fg(Some(Color::Cyan)));
    color!(self, magenta, ColorSpec::new().set_fg(Some(Color::Magenta)));
    color!(
        self,
        b_red,
        ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true)
    );
    color!(
        self,
        br_black,
        ColorSpec::new()
            .set_fg(Some(Color::Black))
            .set_intense(true)
    );
}
