// TODO: The entire program needs to be rewritten.
// We should only take stdin but do everything on the cursor instead
use std::alloc::System;
use std::process::Child;

#[global_allocator]
static A: System = System;
mod config;
mod error;
use config::Conf;
use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::terminal::ClearType;
use crossterm::{execute, terminal};
use error::Error;
use nix::sys::wait::waitpid;
use nix::sys::wait::WaitPidFlag;
use nix::sys::wait::WaitStatus;
use nix::unistd::Pid;
use std::fs::OpenOptions;
const PROMPT_LENGTH: usize = 2;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::io::{stdout, Cursor};
use std::process::Command;
use std::process::Stdio;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use toml::toml;
//use tosh_classifier::{classify, prime};
type Result<T> = std::result::Result<T, error::Error>;

fn main() -> Result<()> {
    let term = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term))
        .map_err(Error::Signal)?;
    signal_hook::flag::register(signal_hook::consts::SIGQUIT, Arc::clone(&term))
        .map_err(Error::Signal)?;
    signal_hook::flag::register(signal_hook::consts::SIGTSTP, Arc::clone(&term))
        .map_err(Error::Signal)?;
    //let mut stdout = stdout();
    terminal::enable_raw_mode().map_err(Error::Term)?;
    let mut history: Vec<String> = vec![];
    populate_history(&mut history)?;
    println!("{}", history.len());
    let mut fg_list: Vec<Child> = Vec::with_capacity(32);
    let mut history_index = history.len();
    let mut curse: Cursor<String> = Cursor::new(String::new());
    let mut config = Conf::load_conf().unwrap_or_else(|_| Conf::default());
    shell_return();
    stdout().flush().map_err(Error::Inout)?;
    let mut start = crossterm::cursor::position().map_err(Error::Term)?;
    loop {
        execute!(stdout(), crossterm::terminal::EnableLineWrap).map_err(Error::Term)?;
        terminal::enable_raw_mode().map_err(Error::Term)?;
        if event::poll(Duration::from_millis(100)).map_err(Error::Term)? {
            if let Event::Key(event) = event::read().expect("Failed to read line") {
                match event {
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: event::KeyModifiers::CONTROL,
                    } => break,
                    KeyEvent {
                        code: KeyCode::Char('w'),
                        modifiers: event::KeyModifiers::CONTROL,
                    } => {
                        history_index = history.len();
                        // ctrl+w will delete the last word. It's the same as backspace, but we use the last occuring space to remove the entire word.
                        // Note: If we only have one word, remove everything.
                        let current_letter = curse.position();
                        let cmd = curse.get_mut();
                        let last_space = cmd[..current_letter as usize].rfind(' ').unwrap_or(0);
                        if !cmd.is_empty() {
                            if current_letter as usize == cmd.len() {
                                cmd.truncate(last_space);
                            } else {
                                cmd.replace_range(last_space..current_letter as usize, "");
                                //print!("{last_space}")
                                //last_space = 0;
                                //println!("\n\rDEBUG: {cmd}");
                            }
                            for _ in last_space..(current_letter as usize) {
                                print!("\u{0008}");
                            }
                            print!("{}", crossterm::cursor::SavePosition);

                            execute!(
                                stdout(),
                                crossterm::terminal::Clear(ClearType::UntilNewLine)
                            )
                            .map_err(Error::Term)?;
                            let rest = cmd[last_space..].to_owned();
                            print!("{}", rest);
                            print!("{}", crossterm::cursor::RestorePosition);
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

                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        history_index = history.len();
                        let current_letter = curse.position();
                        let cmd = curse.get_mut();

                        if current_letter != 0 {
                            //last_space = cmd.rfind(' ').unwrap_or(0);
                            if current_letter as usize == cmd.len() {
                                cmd.pop();
                                //println!("\n\rDEBUG: {cmd}");
                            } else {
                                cmd.remove(current_letter as usize - 1);
                                //last_space = 0;
                                //print!("\n\rDEBUG: {cmd}");
                            }
                            let term_curse_pos =
                                crossterm::cursor::position().map_err(Error::Term)?;
                            let term_size = crossterm::terminal::size().map_err(Error::Term)?;

                            if term_curse_pos.0 == 0 {
                                print!("{}", crossterm::cursor::MoveUp(1));
                                print!("{}", crossterm::cursor::MoveRight(term_size.0));
                            } else {
                                print!("\u{0008}");
                            }
                            print!("{}", crossterm::cursor::SavePosition);

                            execute!(
                                stdout(),
                                crossterm::terminal::Clear(ClearType::FromCursorDown)
                            )
                            .map_err(Error::Term)?;
                            let rest = cmd[current_letter as usize - 1..].to_owned();
                            print!("{}", rest);
                            print!("{}", crossterm::cursor::RestorePosition);
                            //println!("\n\rDEBUG: {cmd}, REST: {rest}");
                            curse.seek(SeekFrom::Current(-1)).map_err(Error::Signal)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('u'),
                        modifiers: event::KeyModifiers::CONTROL,
                    } => {
                        history_index = history.len();
                        execute!(stdout(), crossterm::terminal::Clear(ClearType::CurrentLine))
                            .map_err(Error::Term)?;
                        curse = Cursor::new(String::new());
                        shell_no_return();
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        let string = curse.get_ref();
                        crossterm::terminal::disable_raw_mode().map_err(Error::Term)?;
                        if string.is_empty() {
                            continue;
                        }
                        if let Some(t) = history.last() {
                            if t != string {
                                history.push(string.to_owned());
                            }
                        }
                        if string.trim() == "exit" {
                            print!("\n\rBye!!!!!!!!!!!!!!!!!!!\r");
                            break;
                        }
                        process_command(string, &mut config, &mut fg_list)?;
                        crossterm::terminal::enable_raw_mode().map_err(Error::Term)?;
                        curse.set_position(0);
                        curse = Cursor::new(String::new());
                        history_index = history.len();
                    }
                    KeyEvent {
                        code: KeyCode::Char(k),
                        modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
                    } => {
                        // reset history cursor to the end of the history

                        curse.seek(SeekFrom::Current(1)).map_err(Error::Term)?;

                        history_index = 0;
                        let cur_pos = curse.position() as usize;
                        let cmd = curse.get_mut();
                        let term_curse_pos = crossterm::cursor::position().map_err(Error::Term)?;
                        let term_size = crossterm::terminal::size().map_err(Error::Term)?;

                        if cur_pos < cmd.len() && !cmd.is_empty() {
                            if (cmd.len() + PROMPT_LENGTH) % (term_size.0 as usize) == 0
                                && (start.1 as usize + ((cmd.len() + 2) / (term_size.0 as usize))
                                    == term_size.1 as usize
                                    || start.1 == term_size.1 - 1)
                            {
                                print!("\x1b[1S");
                                print!("{}", crossterm::cursor::MoveUp(1));
                            }
                            cmd.insert(cur_pos - 1, k);

                            print!("{}", crossterm::cursor::SavePosition);
                            print!("{}", &cmd[cur_pos - 1..]);
                            print!("{}", crossterm::cursor::RestorePosition);
                            if term_curse_pos.0 == term_size.0 - 1 {
                                print!("{}", crossterm::cursor::MoveDown(1));
                                print!("\r");
                            } else {
                                print!("{}", crossterm::cursor::MoveRight(1));
                            }
                        } else {
                            cmd.push(k);
                            print!("{k}");
                            if term_curse_pos.0 + 1 == term_size.0 {
                                print!("\n\r");
                            }
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        history_index = history.len();
                        if curse.position() != 0 {
                            let term_curse_pos =
                                crossterm::cursor::position().map_err(Error::Term)?;
                            let term_size = crossterm::terminal::size().map_err(Error::Term)?;
                            if term_curse_pos.0 == 0 {
                                print!("{}", crossterm::cursor::MoveUp(1));
                                print!("{}", crossterm::cursor::MoveRight(term_size.0));
                            } else {
                                print!("{}", crossterm::cursor::MoveLeft(1));
                            }
                            curse
                                .seek(std::io::SeekFrom::Current(-1))
                                .map_err(Error::Term)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        history_index = history.len();
                        if (curse.position() as usize) < curse.get_ref().len() {
                            let term_curse_pos =
                                crossterm::cursor::position().map_err(Error::Term)?;

                            let term_size = crossterm::terminal::size().map_err(Error::Term)?;
                            if term_curse_pos.0 == term_size.0 - 1 {
                                print!("{}", crossterm::cursor::MoveDown(1));
                                print!("{}", crossterm::cursor::MoveLeft(term_size.0));
                            } else {
                                print!("{}", crossterm::cursor::MoveRight(1));
                            }

                            curse
                                .seek(std::io::SeekFrom::Current(1))
                                .map_err(Error::Term)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        if !history.is_empty() && history_index > 0 {
                            history_index -= 1;
                            let cmd = curse.get_mut();
                            cmd.clear();
                            cmd.push_str(&history[history_index]);
                            execute!(stdout(), crossterm::terminal::Clear(ClearType::CurrentLine))
                                .map_err(Error::Term)?;
                            print!("\r⡢ {cmd}");
                            curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        if !history.is_empty() && history_index != history.len() {
                            if history_index == history.len() - 1 {
                                history_index += 1;
                                let cmd = curse.get_mut();
                                cmd.clear();
                                execute!(
                                    stdout(),
                                    crossterm::terminal::Clear(ClearType::CurrentLine)
                                )
                                .map_err(Error::Term)?;
                                print!("\r⡢ {cmd}");
                                curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                            } else {
                                history_index += 1;
                                let cmd = curse.get_mut();
                                cmd.clear();
                                cmd.push_str(&history[history_index]);
                                execute!(
                                    stdout(),
                                    crossterm::terminal::Clear(ClearType::UntilNewLine)
                                )
                                .map_err(Error::Term)?;
                                print!("\r⡢ {cmd}");
                                curse.seek(SeekFrom::End(0)).map_err(Error::Term)?;
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Tab,
                        modifiers: event::KeyModifiers::NONE,
                    } => {
                        history_index = 0;
                        tab_completion(&mut curse)?;
                    }
                    _ => {
                        history_index = history.len();
                        curse = Cursor::new(String::new());
                        shell_return();
                    }
                }
                if curse.get_ref().is_empty() {
                    start = crossterm::cursor::position().map_err(Error::Term)?;
                }
                //terminal::disable_raw_mode().map_err(Error::Term)?;
                stdout().flush().map_err(Error::Inout)?;

                //}
            }
        }
    }
    save_history(history)?;
    config.save_conf()?;
    terminal::disable_raw_mode().map_err(Error::Term)?;
    Ok(())
}

fn process_command(input: &str, conf: &mut Conf, fg_list: &mut Vec<Child>) -> Result<()> {
    let t = input.strip_prefix('\n').unwrap_or(input);
    // get args

    let args: Vec<&str> = t.split_whitespace().collect();

    let arguments = args.iter().skip(1);
    if args.is_empty() {
        return Ok(());
    }
    let cmd = args[0].to_owned();
    println!("\r\n");
    match cmd.as_str() {
        "cd" => {
            crossterm::terminal::disable_raw_mode().map_err(Error::Term)?;
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
                conf.add_env_var((args[1].to_string(), args[2].to_string()))?;
                conf.save_conf().unwrap();
                shell_return();
                return Ok(());
            }
        }
        "fg" => {
            //crossterm::terminal::disable_raw_mode().map_err(Error::Term)?;
            if let Some(fg_prg) = dbg!(fg_list).pop() {
                nix::sys::signal::kill(
                    Pid::from_raw(fg_prg.id() as i32),
                    nix::sys::signal::Signal::SIGCONT,
                )
                .unwrap();
                let pid = Pid::from_raw(fg_prg.id() as i32);
                //TODO: Fix this to be similar to the one before
                waitpid(pid, Some(WaitPidFlag::WUNTRACED)).unwrap();
                shell_return();
            } else {
                println!("NO PID FOUND");
                shell_return();
            }
            return Ok(());
        }

        _ => {}
    };
    if args.len() == 1 {
        crossterm::terminal::disable_raw_mode().map_err(Error::Term)?;
        // Change to Error::Jc
        let mut jc_output = Command::new("jc")
            .arg("-r")
            .arg(&cmd)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .output()
            .map_err(Error::Term)?;
        if let Some(jc_code) = jc_output.status.code() {
            if jc_code < 100 {
                let json_value = serde_json::from_slice::<toml::Value>(&jc_output.stdout).unwrap();

                println!("{}", json_value);
                shell_return();
                return Ok(());
            }
        }
        let mut output = Command::new(cmd);
        if let Ok(process) = output.spawn() {
            let pid_u32 = process.id();

            let pid = Pid::from_raw(pid_u32.try_into().unwrap());
            if let Ok(x) = waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
                match x {
                    WaitStatus::Stopped(p, nix::sys::signal::Signal::SIGTSTP) => {
                        fg_list.push(process);
                    }
                    _ => {}
                }
            }
            //TODO: Tomlize the ouput
            // tomlize_stdout(&process.stdout);
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
        crossterm::terminal::disable_raw_mode().map_err(Error::Term)?;
        let mut output = Command::new(cmd);
        output.args(arguments);
        if let Ok(process) = output.spawn() {
            let pid = process.id();
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
        //TODO: Implement fg for returning these processes
    }

    shell_return();
    Ok(())
}

fn tab_completion(cmd: &mut Cursor<String>) -> Result<()> {
    use tosh::trie::Trie;
    let mut obj = Trie::default();
    let key = "PATH";
    match std::env::var_os(key) {
        Some(paths) => {
            for path in std::env::split_paths(&paths) {
                if let Ok(t) = path.read_dir() {
                    t.for_each(|x| obj.insert(x.unwrap().file_name().into_string().unwrap()));
                }
                //println!("{path:?}");
            }
        }
        None => return Ok(()),
    }
    if let Some(matches) = obj.root.collect_all_matches(cmd.get_ref()) {
        if !matches.is_empty() && matches.len() == 1 && matches[0].len() > cmd.get_ref().len() {
            let cmdd = cmd.get_mut();
            if cmdd.trim() != matches[0].trim() {
                let from = cmdd.len();
                let to = matches[0].len();
                *cmdd = matches[0].to_string();
                print!("{}", &cmdd[from..]);
                cmd.seek(SeekFrom::Current((to - from) as i64))
                    .map_err(Error::Term)?;
            }
        }
    }
    Ok(())
}

fn shell_return() {
    print!("\r\n⡢ ");
}

fn shell_no_return() {
    print!("\r⡢ ");
}

fn save_history(history: Vec<String>) -> Result<()> {
    let fd = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dirs_next::home_dir().unwrap().join("history.tosh"))
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
        .open(dirs_next::home_dir().unwrap().join("history.tosh"))
        .map_err(Error::File)?;

    let f = std::io::BufReader::new(fd);
    f.lines().for_each(|l| history.push(l.unwrap()));
    Ok(())
}

fn process_output(out: String) {
    todo!()
}

fn classifier_initialization() -> Result<()> {
    todo!()
}
