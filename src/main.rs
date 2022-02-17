// TODO: The entire program needs to be rewritten.
// We should only take stdin but do everything on the cursor instead
mod config;
mod error;
use config::Conf;
use crossterm::terminal;
use error::Error;
use nix::sys::wait::*;
use nix::unistd::Pid;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{stdin, stdout, Cursor};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::style;
use tokio::process::Command;
use toml::toml;
struct CleanUp;
impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
async fn main() -> Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))
        .map_err(Error::Signal)?;
    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term))
        .map_err(Error::Signal)?;
    signal_hook::flag::register(signal_hook::consts::SIGTSTP, Arc::clone(&term))
        .map_err(Error::Signal)?;

    let mut stdout = stdout().into_raw_mode().map_err(Error::Term)?;
    let mut history: Vec<String> = vec![];
    populate_history(&mut history)?;
    println!("{}", history.len());
    let mut history_index = history.len();
    let stdin = stdin();
    let mut curse: Cursor<String> = Cursor::new(String::new());

    let mut config = Conf::load_conf().unwrap_or_else(|_| Conf::default());

    shell_return();
    stdout.flush().map_err(Error::Inout)?;

    for c in stdin.keys() {
        match c.as_ref().expect("ERROR FETCHING") {
            Key::Ctrl('q') => {
                //stdout.activate_raw_mode()?;
                println!("\r\nQUITTING TOSH, Take Care <3\r");
                break;
            }
            Key::Ctrl('w') => {
                history_index = history.len();
                // ctrl+w will delete the last word. It's the same as backspace, but we use the last occuring space to remove the entire word.
                // Note: If we only have one word, remove everything.
                let current_letter = curse.position();
                let cmd = curse.get_mut();
                let mut last_space = cmd.rfind(' ').unwrap_or(0);
                if !cmd.is_empty() {
                    if current_letter as usize == cmd.len() {
                        cmd.truncate(last_space);
                    } else {
                        cmd.replace_range(
                            current_letter as usize - last_space..current_letter as usize,
                            "",
                        );
                        last_space = 0;
                        //println!("\n\rDEBUG: {cmd}");
                    }
                    for _ in 0..current_letter as usize - last_space {
                        print!("\u{0008}");
                    }
                    print!("{}", termion::cursor::Save);

                    print!("{}", termion::clear::AfterCursor);
                    let rest = cmd[last_space..].to_owned();
                    print!("{}", rest);
                    print!("{}", termion::cursor::Restore);
                    //println!("\n\rDEBUG: {cmd}, REST: {rest}");
                    curse
                        .seek(SeekFrom::Current(
                            (-(current_letter as isize - last_space as isize))
                                .try_into()
                                .unwrap(),
                        ))
                        .map_err(Error::Term)?;
                }
            }
            Key::Char(k) => {
                // reset history cursor to the end of the history

                curse.seek(SeekFrom::Current(1)).map_err(Error::Term)?;
                if *k == '\n' {
                    // handle exit only, i don't like how it's handled now

                    let string = curse.get_ref();
                    //stdout.suspend_raw_mode()?;
                    if string.is_empty() {
                        continue;
                    }
                    history.push(string.to_owned());
                    if string.trim() == "exit" {
                        print!("\n\rBye!!!!!!!!!!!!!!!!!!!\r");
                        break;
                    }
                    process_command(string, &mut stdout, &mut config).await?;
                    curse.set_position(0);
                    curse = Cursor::new(String::new());
                    history_index = history.len();
                    //
                } else if *k == '\t' {
                    history_index = 0;
                    tab_completion()
                } else {
                    history_index = 0;
                    let cmd = curse.get_mut();
                    cmd.push(*k);
                    write!(stdout, "{}", k).map_err(Error::Inout)?;
                    //print!("{}", cursor::Right(1));
                    //print!("{}", *k);
                }
            }
            Key::BackTab => tab_completion(),
            Key::Backspace => {
                history_index = history.len();
                let current_letter = curse.position();
                let cmd = curse.get_mut();
                if current_letter != 0 {
                    //last_space = cmd.rfind(' ').unwrap_or(0);
                    if current_letter as usize == cmd.len() {
                        cmd.pop();
                        //println!("\n\rDEBUG: {cmd}");
                    } else {
                        cmd.remove(current_letter as usize);
                        //last_space = 0;
                        //print!("\n\rDEBUG: {cmd}");
                    }

                    print!("\u{0008}");
                    print!("{}", termion::cursor::Save);

                    print!("{}", termion::clear::AfterCursor);
                    let rest = cmd[current_letter as usize - 1..].to_owned();
                    print!("{}", rest);
                    print!("{}", termion::cursor::Restore);
                    //println!("\n\rDEBUG: {cmd}, REST: {rest}");
                    curse.seek(SeekFrom::Current(-1)).map_err(Error::Signal)?;
                }
            }
            Key::Ctrl('u') => {
                history_index = history.len();
                print!("{}", termion::clear::CurrentLine);
                curse = Cursor::new(String::new());
                print!("\r> ");
            }
            Key::Left => {
                history_index = history.len();
                if curse.position() != 0 {
                    let term_curse_pos = termion::cursor::DetectCursorPos::cursor_pos(&mut stdout)
                        .map_err(Error::Term)?;
                    let term_size = termion::terminal_size().map_err(Error::Term)?;
                    if term_curse_pos.0 == 1 {
                        print!("{}", termion::cursor::Up(1));
                        print!("{}", termion::cursor::Right(term_size.0));
                    } else {
                        print!("{}", termion::cursor::Left(1));
                    }
                    curse
                        .seek(std::io::SeekFrom::Current(-1))
                        .map_err(Error::Term)?;
                }
            }
            Key::Right => {
                history_index = history.len();
                if (curse.position() as usize) < curse.get_ref().len() {
                    let term_curse_pos = termion::cursor::DetectCursorPos::cursor_pos(&mut stdout)
                        .map_err(Error::Term)?;
                    let term_size = termion::terminal_size().map_err(Error::Term)?;
                    if term_curse_pos.0 == term_size.0 {
                        print!("{}", termion::cursor::Down(1));
                        print!("{}", termion::cursor::Left(term_size.0));
                    } else {
                        print!("{}", termion::cursor::Right(1));
                    }

                    curse
                        .seek(std::io::SeekFrom::Current(1))
                        .map_err(Error::Term)?;
                }
            }
            Key::Up => {
                if !history.is_empty() && history_index > 0 {
                    history_index -= 1;
                    let cmd = curse.get_mut();
                    cmd.clear();
                    cmd.push_str(&history[history_index]);
                    print!("{}", termion::clear::CurrentLine);
                    print!("\r⡢ {cmd}");
                    curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                }
            }
            Key::Down => {
                if !history.is_empty() && history_index != history.len() {
                    if history_index == history.len() - 1 {
                        history_index += 1;
                        let cmd = curse.get_mut();
                        cmd.clear();
                        print!("{}", termion::clear::CurrentLine);
                        print!("\r⡢ {cmd}");
                        curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                    } else {
                        history_index += 1;
                        let cmd = curse.get_mut();
                        cmd.clear();
                        cmd.push_str(&history[history_index]);
                        print!("{}", termion::clear::CurrentLine);
                        print!("\r⡢ {cmd}");
                        curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                    }
                }
            }
            _ => {
                history_index = history.len();
                curse = Cursor::new(String::new());
                shell_return();
            }
        }
        stdout.activate_raw_mode().map_err(Error::Term)?;
        stdout.flush().map_err(Error::Inout)?;

        //}
    }
    save_history(history)?;
    Ok(())
}

async fn process_command(
    input: &str,
    out: &mut termion::raw::RawTerminal<std::io::Stdout>,
    conf: &mut Conf,
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
    match cmd.as_str() {
        "cd" => {
            out.suspend_raw_mode().map_err(Error::Term)?;
            if arguments.clone().count() == 0 {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/".to_owned());
                std::env::set_current_dir(home).map_err(Error::Cd)?;
            } else if let Err(e) = std::env::set_current_dir(args[1]).map_err(Error::Cd) {
                eprintln!("{}", toml::to_string(&e).map_err(Error::Parse)?)
            }
            shell_return();
            return Ok(());
        }
        "add_to_env" => {
            if arguments.clone().count() < 2 {
                eprintln!("Invalid Number of Args");
            } else {
                conf.add_env_var((args[1].to_string(), args[2].to_string()));
                conf.save_conf().unwrap();
                shell_return();
                return Ok(());
            }
        }
        _ => {}
    };
    if args.len() == 1 {
        out.suspend_raw_mode().map_err(Error::Term)?;
        let mut output = Command::new(cmd);
        if let Ok(process) = output.spawn() {
            let pid = process.id().unwrap();
            let pid = Pid::from_raw(pid.try_into().unwrap());
            waitpid(pid, Some(WaitPidFlag::WUNTRACED)).unwrap();
        } else {
            let cmd_str = format!("Command Not Found {}", args[0]);

            eprint!(
                "{}",
                toml! {
                    [Error]
                    Source = cmd_str
                }
            );
            shell_return();
            return Ok(());
        }
    } else {
        out.suspend_raw_mode().map_err(Error::Inout)?;
        let mut output = Command::new(cmd);
        output.args(arguments);
        let pid = output.spawn().unwrap().id().unwrap();
        let pid = Pid::from_raw(pid.try_into().unwrap());
        waitpid(pid, Some(WaitPidFlag::WUNTRACED)).unwrap();
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
    print!("\r\n{}⡢ {}", style::Bold, style::Reset);
}
fn save_history(history: Vec<String>) -> Result<()> {
    let fd = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dirs::home_dir().unwrap().join("history.tosh"))
        .map_err(Error::File)?;
    let mut f = std::io::BufWriter::new(fd);
    writeln!(f, "{}", history.join("\n")).map_err(Error::File)?;
    Ok(())
}
fn populate_history(history: &mut Vec<String>) -> Result<()> {
    use std::io::BufRead;
    let fd = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dirs::home_dir().unwrap().join("history.tosh"))
        .map_err(Error::File)?;

    let f = std::io::BufReader::new(fd);
    f.lines().for_each(|l| history.push(l.unwrap()));
    Ok(())
}
