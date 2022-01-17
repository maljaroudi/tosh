// TODO: The entire program needs to be rewritten.
// We should only take stdin but do everything on the cursor instead

use nix::sys::wait::*;
use nix::unistd::Pid;
use std::io::BufRead;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{stdin, stdout, Cursor};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use termion::cursor;
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::process::Command;
#[tokio::main]
async fn main() -> Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term))?;
    signal_hook::flag::register(signal_hook::consts::SIGTSTP, Arc::clone(&term))?;

    let mut stdout = stdout().into_raw_mode()?;

    let stdin = stdin();
    let mut curse: Cursor<String> = Cursor::new(String::new());

    shell_return();
    stdout.flush()?;

    for c in stdin.keys() {
        match c.as_ref().expect("ERROR FETCHING") {
            Key::Ctrl('q') => {
                //stdout.activate_raw_mode()?;
                println!("\r\nQUITTING TOSH, LOVE YOU <3\r");
                break;
            }
            Key::Char(k) => {
                curse.seek(SeekFrom::Current(1))?;
                if *k == '\n' {
                    let string = curse.get_ref();
                    //stdout.suspend_raw_mode()?;
                    process_command(string, &mut stdout).await?;
                    curse.set_position(0);
                    curse = Cursor::new(String::new());
                    //
                } else if *k == '\t' {
                    tab_completion()
                } else {
                    let cmd = curse.get_mut();
                    cmd.push(*k);
                    write!(stdout, "{}", k)?;
                    //print!("{}", cursor::Right(1));
                    //print!("{}", *k);
                }
            }
            Key::BackTab => tab_completion(),
            Key::Backspace => {
                let current_letter = curse.position();
                let cmd = curse.get_mut();
                if !cmd.is_empty() {
                    if current_letter as usize == cmd.len() {
                        cmd.pop();
                    } else {
                        cmd.remove(current_letter as usize - 1);
                    }
                    print!("\u{0008}");
                    print!("{}", termion::cursor::Save);

                    print!("{}", termion::clear::AfterCursor);
                    let rest = cmd[current_letter as usize - 1..].to_owned();
                    print!("{}", rest);
                    print!("{}", termion::cursor::Restore);
                    curse.seek(SeekFrom::Current(-1))?;
                }
            }
            Key::Ctrl('u') => {
                print!("{}", termion::clear::CurrentLine);
                curse = Cursor::new(String::new());
                print!("\r> ");
            }
            Key::Left => {
                if curse.position() > 0 {
                    print!("{}", termion::cursor::Left(1));
                    curse.seek(std::io::SeekFrom::Current(-1))?;
                }
            }
            Key::Right => {
                if (curse.position() as usize) < curse.get_ref().len() {
                    curse.seek(std::io::SeekFrom::Current(1))?;
                    print!("\x1b[C")
                }
            }
            _ => {
                curse = Cursor::new(String::new());
                shell_return();
            }
        }
        stdout.activate_raw_mode()?;
        stdout.flush()?;
        //}
    }
    Ok(())
}

async fn process_command(
    input: &str,
    out: &mut termion::raw::RawTerminal<std::io::Stdout>,
) -> Result<()> {
    let t = input.strip_prefix('\n').unwrap_or(input);
    // get args

    let args: Vec<&str> = t.split_whitespace().collect();

    let arguments = args.iter().skip(1);
    if args.is_empty() {
        return Ok(());
    }
    let cmd = args[0].to_owned();
    println!("\r");
    if args.len() == 1 {
        out.suspend_raw_mode()?;
        let mut output = Command::new(cmd);
        // get pid of process
        let pid = output.spawn()?.id().unwrap();
        let pid = Pid::from_raw(pid.try_into().unwrap());
        waitpid(pid, Some(WaitPidFlag::WUNTRACED))?;
    } else {
        out.suspend_raw_mode()?;
        let mut output = Command::new(cmd);
        output.args(arguments);
        let pid = output.spawn()?.id().unwrap();
        let pid = Pid::from_raw(pid.try_into().unwrap());
        waitpid(pid, Some(WaitPidFlag::WUNTRACED))?;
        //TODO: Implement fg for returning these processes
    }

    shell_return();
    Ok(())
}

fn tab_completion() {
    print!("TAB COMPLETION");
    shell_return();
}

fn shell_return() {
    print!("\r\n> ");
}
