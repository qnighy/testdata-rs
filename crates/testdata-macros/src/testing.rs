use std::io::{self, BufWriter, Write};
use std::process::{Command, Stdio};

pub(crate) fn rustfmt(s: &str) -> io::Result<String> {
    let mut rustfmt = Command::new("rustfmt")
        .args(&["--edition", "2021"])
        .args(&["--color", "never"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let rustfmt_stdin = rustfmt.stdin.take().unwrap();
    let mut rustfmt_stdin = BufWriter::new(rustfmt_stdin);
    writeln!(rustfmt_stdin, "{}\n", s)?;
    rustfmt_stdin.flush()?;
    drop(rustfmt_stdin);

    let rustfmt_output = rustfmt.wait_with_output()?;
    if !rustfmt_output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("rustfmt failed: {:?}", rustfmt_output),
        ));
    }
    let src = String::from_utf8_lossy(&rustfmt_output.stdout).into_owned();
    Ok(src)
}

#[macro_export]
macro_rules! assert_ts_eq {
    ($lhs:expr, $rhs:expr) => {
        match ($lhs, $rhs) {
            (lhs, rhs) => {
                let lhs = lhs.to_string();
                let rhs = rhs.to_string();
                if lhs != rhs {
                    if let (Ok(lhs), Ok(rhs)) = (
                        $crate::testing::rustfmt(&lhs),
                        $crate::testing::rustfmt(&rhs),
                    ) {
                        pretty_assertions::assert_eq!(lhs, rhs);
                    }
                    pretty_assertions::assert_eq!(lhs, rhs);
                    unreachable!();
                }
            }
        }
    };
}
