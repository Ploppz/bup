use chrono::Duration;
use lazy_static::lazy_static;
use slog::*;
use slog_async::*;
use slog_term::*;
use std::{
    io::{self, Write},
    path::PathBuf,
};

pub fn logger() -> Logger {
    let decorator = TermDecorator::new().build();
    let drain = FullFormat::new(decorator)
        .use_custom_header_print(print_msg_header)
        .build()
        .fuse();
    let drain = Filter::new(drain, |record| record.tag().is_empty()).fuse();
    let drain = Async::new(drain).build().fuse();
    Logger::root(drain, o!())
}
pub fn print_msg_header(
    fn_timestamp: &dyn ThreadSafeTimestampFn<Output = io::Result<()>>,
    mut rd: &mut dyn RecordDecorator,
    record: &Record,
    use_file_location: bool,
) -> io::Result<bool> {
    rd.start_timestamp()?;
    fn_timestamp(&mut rd)?;

    rd.start_whitespace()?;
    write!(rd, " [")?;
    rd.start_location()?;
    write!(rd, "{}", record.tag())?;
    rd.start_whitespace()?;
    write!(rd, "] ")?;

    rd.start_level()?;
    write!(rd, "{}", record.level().as_short_str())?;

    if use_file_location {
        rd.start_location()?;
        write!(
            rd,
            "[{}:{}:{}]",
            record.location().file,
            record.location().line,
            record.location().column
        )?;
    }

    rd.start_whitespace()?;
    write!(rd, " ")?;

    rd.start_msg()?;
    let mut count_rd = CountingWriter::new(&mut rd);
    write!(count_rd, "{}", record.msg())?;
    Ok(count_rd.count() != 0)
}
