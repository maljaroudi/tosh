use nix::sys::wait::*;
use nix::unistd::Pid;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{stdin, stdout, Cursor};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use termion::clear;
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
                println!("QUITTING TOSH, LOVE YOU <3");
                break;
            }
            Key::Char(k) => {
                curse.seek(SeekFrom::Current(1))?;
                if *k == '\n' {
                    //
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
                if curse.position() > 0 {
                    //print!("\u{0008} \u{0008}");
                    let pos = curse.position();
                    curse.seek(SeekFrom::Current(-1))?;
                    let c = cursor::Save;
                    let poss = DetectCursorPos::cursor_pos(&mut stdout)?;
                    if poss.0 == 1 && !curse.get_ref().is_empty() {
                        print!("{}", cursor::Up(1));
                        print!("{}", cursor::Right(pos as u16 + 1));
                    } else {
                        print!("{}", cursor::Left(1));
                    }
                    let cmd = curse.get_mut();
                    cmd.remove((pos - 1) as usize);

                    print!("{}", cursor::Save);
                    //remove the current char
                    print!("\u{0008} \u{0008}");
                    print!("{}", termion::clear::AfterCursor);

                    if poss.0 < (2) as u16 {
                        print!("\r> ");
                        print!("{}", cmd);
                        print!("{}", cursor::Restore);
                    }
                    //otherwise print the command with index between the cursor and the cursor
                    else {
                        print!(
                            "\r{}",
                            cmd[(pos - poss.0 as u64 + 1) as usize..].to_string()
                        );
                        print!("{}", cursor::Restore);
                    }
                    //print!("{}{}", poss.0, pos);

                    //print!("{}", cursor::Goto(poss.0 - 1, poss.1));
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
