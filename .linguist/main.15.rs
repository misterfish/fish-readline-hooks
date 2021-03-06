use std::env;
use std::process::Command;
use std::io;
use std::fs::File;
use std::io::Read;
use std::io::BufRead;
use std::io::Write;
use std::slice;
use std::borrow::Cow;

extern crate regex;
extern crate shlex;
extern crate libc;

use libc::c_int;
use libc::size_t;

use regex::Regex;

// --- (output, should_rewind, should_quote) / empty error.
type DispatchCommandResults = Result<(String, bool, bool), ()>;

// --- (cdata, num, dir, cmd).
//type CommandParseParsed<'a> = (&'a str, &'a str, &'a str, &'a str);
type CommandParseParsed<'a> = (String, String, String, String);

enum CommandParseResult<'a> {
    Found( CommandParseParsed<'a> ),
    NotFound,
    Err(&'a str),
}

struct Config {
    history_file_path: &'static str,
}

#[allow(non_upper_case_globals)]
const config: Config = Config {
    history_file_path: "/tmp/history.txt",
};

struct Main {
    // --- storing a box is probably not really better than just storing the struct.
    dispatchers: Vec<Box<Dispatcher>>,

    parse_results: ParseResults,
}

// --- where the ffi parser stuffs results as it goes.
//
// the strings get dup'ed so the ffi parser can free them.
struct ParseResults {
    cdata:  String,
    num:    String,
    dir:    String,
    cmd:    String,
}

impl Main {
    fn add_dispatcher(&mut self, re: &str, func: fn(DispatchData) -> DispatchCommandResults) {
        let re_full = "(?x)".to_string() + re;
        let dispatcher = Dispatcher {
            re: Regex::new(&re_full)
                .unwrap_or_else(|e| { panic!("{}", e) }),
            cb: func,
        };
        self.dispatchers.push(Box::new(dispatcher));
    }
    
    fn parse_event_cdata(&mut self, cdata: &str) {
        self.parse_results.cdata = cdata.to_string();
    }
}

struct DispatchData {
    first_part: String,
}

struct Dispatcher {
    re: Regex,
    cb: fn(DispatchData) -> DispatchCommandResults,
}

struct ReadlineState {
    line: String,
    point: String,
}

// --- the ref before the array on both sides makes it so you don't need to give the size.
//
// otherwise you need [T; n] notation.

const DISPATCH: &'static [(&'static str, fn(DispatchData) -> DispatchCommandResults)] = &[
    (r" ^ g     $", handle_g),
    (r" ^ t     $", handle_t),
    (r" ^ tr    $", handle_tr),
    (r" ^ l     $", handle_l),
    (r" ^ lr    $", handle_lr),
    (r" ^ T     $", handle_T),
    (r" ^ z     $", handle_z),
];

fn main() {
    // --- stores global parse data and command dispatch table.
    let main = get_main();

    // to practice XX
    let mut my_results = ParseResults {
        cdata: "".to_string(),
        cmd: "".to_string(),
        num: "".to_string(),
        dir: "".to_string(),
    };
    let mut results_ptr = Box::new(my_results);

    register_ffi();

    let readline_state_in = get_readline_state();

    //let (cdata, num, dir, command) = parse(&main.parse_results, &readline_state_in.line)
    //let (cdata, num, dir, command) = parse(&mut results_ptr, &readline_state_in.line)
    parse(&mut results_ptr, &readline_state_in.line)
        .unwrap_or_else(|e| { panic!("{}", e) });

    let cdata = &results_ptr.cdata;
    let dir = &results_ptr.dir;
    let num = &results_ptr.num;
    let cmd = &results_ptr.cmd;

    for dispatcher in &main.dispatchers {
        let ref re = (*dispatcher).re;
        if re.is_match(cmd) {
            match process(cdata, dispatcher, &readline_state_in) {
                Ok(readline_state_out) => {
                    store_history(cdata);
                    output(&readline_state_out);
                },
                // --- do nothing if process failed.
                //_   => {},
                _   => return,
            }
            // --- done.
            break;
        }
    }
}

// -- END.



// --- main logic.

fn get_main() -> Main {
    let mut main = Main {
        dispatchers: Vec::new(),
        parse_results: ParseResults {
            cdata: "".to_string(),
            num: "".to_string(),
            dir: "".to_string(),
            cmd: "".to_string(),
        },
    };

    for n in 0..DISPATCH.len() {
        let pair = DISPATCH[n];
        let (re, cb) = pair;
        main.add_dispatcher(re, cb);
    }

    main
}

//fn parse<'a, 'b>(parse_results: &'b ParseResults, line: &'a str) -> Result<CommandParseParsed<'a>, &'a str> {
//fn parse<'a, 'b>(parse_results: &'b mut Box<ParseResults>, line: &'a str) -> Result<CommandParseParsed<'a>, &'a str> {

fn parse<'a, 'b>(parse_results: &'b mut Box<ParseResults>, line: &'a str) -> Result<(), &'a str> {

    let input = line.to_string();
    unsafe {
        rh_parse_init(&mut **parse_results);
        rh_parse_set_input((input + "\0").as_bytes().as_ptr());
        rh_parse_start();
    }

    // --- no real Err case for the enum currently.

    match parse_results.cmd.len() {
        // --- parse error / command not found
        0   => {
            parse_results.cdata = line.to_string();
            parse_results.dir = "".to_string();
            parse_results.num = "".to_string();
            parse_results.cmd = "".to_string();
        },
        _   => {},
    };

    Ok(())
}

fn process(first_part: &str, dispatcher: &Dispatcher, readline_state: &ReadlineState) ->
    Result<ReadlineState, ()> {

    let ref cb = dispatcher.cb;

    let data = DispatchData {
        first_part: first_part.to_string(),
    };

    let (output, should_rewind, should_quote) = match cb(data) {
        Ok(r)   => r,
        _       => return Err(()),
    };

    let output_quoted = match should_quote {
        true    => shell_quote(&output),
        _       => output,
    };

    Ok(
        get_output(&readline_state.point, first_part.to_string(), output_quoted, should_rewind)
    )
}

fn get_readline_state() -> ReadlineState {
    let mut line: String = get_env("READLINE_LINE");
    let mut point: String = get_env("READLINE_POINT");

    if point.len() == 0 {
        warn("READLINE_POINT not set, running with test data.".to_string());
        warn("".to_string());
        line = "mv -iv = /tmp 2 t".to_string();
        // ?
        point = format!("{}", line.len());
    }
    
    ReadlineState {
        line: line,
        point: point,
    }
}

fn get_output(point_in: &str, first_part: String, output: String, should_rewind: bool) -> ReadlineState {
    let first_bit = match should_rewind {
        true    => "".to_string(),
        false   => first_part,
    };

    let point_in_u32 = point_in.parse::<u32>()
        .unwrap_or_else(|e| { panic!("{}", e) });

    let line: String = vec![first_bit, output, " ".to_string()]
        .join("");
    let point: String = (point_in_u32 as u32 + line.len() as u32 + 1)
        .to_string();

    ReadlineState {
        line: line,
        point: point,
    }
}

fn output(state: &ReadlineState) {
    let line = &state.line;
    let point = &state.point;

    let line_quoted = shell_quote(line);

    println!(
        "READLINE_LINE={}; READLINE_POINT={}",
        line_quoted,
        point,
     );
}

fn get_history() -> Result<String, ()> {
    // --- has to be mut to read it into string, not sure why.
    let mut history_file = match File::open(config.history_file_path) {
        Ok(f)   => f,
        _       => {
            warn("Can't open history file for reading".to_string());
            return Err(())
        },
    };

    let mut s = String::new();
    match history_file.read_to_string(&mut s) {
        Ok(_)   => Ok(s),
        _       => {
            warn("Can't read history file".to_string());
            Err(())
        },
    }
}

fn store_history(line: &str) {
    let mut history_file = match File::create(config.history_file_path) {
        Ok(f)   => f,
        _       => {
            warn("Can't open history file for writing".to_string());
            return;
        },
    };
    match history_file.write_all(line.as_bytes()) {
        Ok(_)   => {},
        _       => warn("Can't write to history file".to_string()),
    }
}

// --- commands / handlers.

fn git_commit() -> DispatchCommandResults {
    let output = match cmd("it", vec!["branch"]) {
        Ok(o)   => o,
        _       => return Err(()),
    };
    let mut branch = "UNKNOWN BRANCH".to_string();
    // --- take first with a star, assume only one.
    for line in output.lines() {
        match line.find('*') {
            Some(0) => {
                let branch_line = line.to_string();
                if branch_line.len() >= 3 {
                    branch = String::from_utf8(branch_line
                        // --- vec, assume narrow bytes. XX
                        .into_bytes()
                        .split_off(2)
                    )
                    .unwrap_or_else(|e| { panic!("{}", e) });
                }
            },
            _   => {},
        }
    }
    let output = format!("gpcm '{} ", branch);
    Ok( (output.to_string(), true, false) )
}

fn ls_last_priv(arg: &str, dir: &str) -> DispatchCommandResults {
    let mut arg_vec = match arg.len() {
        0   => vec![],
        _   => vec![arg]
    };
    if dir.len() != 0 {
        arg_vec.push(dir);
    }
    let result = match cmd("ls", arg_vec) {
        Ok(o)   => o,
        _       => return Err(()),
    };

    // --- empty dir: return "".
    let mut last = "";

    match result.lines().last() {
        Some(l)     => last = l,
        _           => {},
    }
    let ret = match dir.len() {
        0   => last.to_string(),
        _   => {
            let re = Regex::new(r"(?x) /$ $ ")
                .unwrap_or_else(|e| { panic!("{}", e) });
            let base = re.replace_all(last, "");
            format!("{}/{}", dir, base)
        },
    };
    Ok( (ret, false, true) )
}

fn handle_g(_: DispatchData) -> DispatchCommandResults {
    git_commit()
}

// --- t and tr switched on purpose.
fn handle_t(_: DispatchData) -> DispatchCommandResults {
    ls_last_priv("-tr", "")
}
fn handle_tr(_: DispatchData) -> DispatchCommandResults {
    ls_last_priv("-t", "")
}
fn handle_l(_: DispatchData) -> DispatchCommandResults {
    ls_last_priv("", "")
}
fn handle_lr(_: DispatchData) -> DispatchCommandResults {
    ls_last_priv("-r", "")
}
#[allow(non_snake_case)]
fn handle_T(data: DispatchData) -> DispatchCommandResults {
    let (first_words, dir) = get_words_partition_right(&data.first_part);

    // --- String.
    let (file, _, _) = match ls_last_priv("-tr", &dir) {
        Ok(r)   => r,
        _       => return Err(()),
    };

    let file_quoted = shell_quote(&file);

    let out = vec![first_words.to_string(), file_quoted]
        .join(" ");

    Ok( (out, true, false) )
}
fn handle_z(_: DispatchData) -> DispatchCommandResults {
    let out = match get_history() {
        Ok(o)   => o,
        _       => return Err(()),
    };
    Ok( (out, true, false) )
}

// --- general.

fn get_words_partition_right(input: &str) -> (String, String) {
    let mut words: Vec<String> = get_words(input);
    let last_word = match words.pop() {
        None    => return ("".to_string(), "".to_string()),
        Some(s) => s,
    };
    let first_words = words.join(" ");
    (first_words, last_word)
}

fn get_words(input: &str) -> Vec<String> {
    match shlex::split(input) {
        None    => panic!("couldn't shlex"),
        Some(s) => s
    }
}

// --- util.

#[allow(dead_code)]
fn warn(w: String) {
    io::stderr().write(w.as_bytes())
        .unwrap_or_else(|e| { panic!("failed to write: {}", e) });
    io::stderr().write("\n".as_bytes())
        .unwrap_or_else(|e| { panic!("failed to write: {}", e) });
}

fn shell_quote(input: &str) -> String {
    match shlex::quote(input) {
        Cow::Borrowed(b) => b.to_string(),
        Cow::Owned(o)    => o,
    }
}

// --- dies if the command couldn't be run; returns 
fn cmd(bin: &str, args: Vec<&str>) -> Result<String, ()> {
    let mut command = Command::new(&bin);
    for arg in &args {
        command.arg(&arg);
    }
    // quote args XX
    let full = vec![bin, args.join(" ").as_ref()].join(" ");
    // --- Vec<u8>.
    let output = match command.output() {
        Ok(o)   => o,
        Err(e)  => {
            warn(format!("Couldn't run cmd «{}»: {}", full, e));
            return Err(());
        }
    };
    if ! output.status.success() {
        warn (format!("Command «{}» unsuccessful.", full));
        return Err(());
    }
    // from_utf8_lossy? XX

    Ok(
        String::from_utf8(output.stdout)
            .unwrap_or_else(|e| { panic!("failed to unwrap output: {}", e) })
    )
}

// --- treat non-existent as "" (like perl, shell etc.)
fn get_env(key: &str) -> String {
    let line: String;
    match env::var(key) {
        Ok(_line)    => { line = _line; }
        Err(_)      => { line = "".to_string(); }
    }
    line
}

#[allow(dead_code)]
fn set_env(key: &'static str, val: String) {
    // --- doesn't tell you if it worked.
    env::set_var(key, &val);
}





#[link(name="rh-parse", kind="static")]
extern {
    // --- some types:
    //
    // u8 -> char.
    // c_int -> int
    // *const u8 -> char*

    fn rh_parse_init(parse_results: *mut ParseResults);
    fn rh_parse_set_input(_: *const u8);
    fn rh_parse_start() -> c_int;

    fn rh_parse_register_cb_store_num(cb: extern fn(_: *mut ParseResults, _: i32));
    fn rh_parse_register_cb_store_cdata(cb: extern fn(_: *mut ParseResults, _: *const u8, _: size_t));
    fn rh_parse_register_cb_store_dir(cb: extern fn(_: *mut ParseResults, _: *const u8, _: size_t));
    fn rh_parse_register_cb_store_command(cb: extern fn(_: *mut ParseResults, _: *const u8, _: size_t));
}

// --- called from c /
extern fn parse_store_cdata(parse_results: *mut ParseResults, data: *const u8, len: size_t) {
    unsafe {
        let slice = slice::from_raw_parts(data, len - 1);
        let thestr = std::str::from_utf8(slice)
            .unwrap_or_else(|e| { panic!("{}", e); });

        //println!("ping cdata! {}", thestr);

        let ref cur_cdata = (*parse_results).cdata;
        // dangle ... xx??
        (*parse_results).cdata =
            // gobbles spaces XX
            format!("{} {}", cur_cdata, thestr);
    };
}
extern fn parse_store_command(parse_results: *mut ParseResults, data: *const u8, len: size_t) {
    unsafe {
        let slice = slice::from_raw_parts(data, len - 1);
        let thestr = std::str::from_utf8(slice)
            .unwrap_or_else(|e| { panic!("{}", e); });

        (*parse_results).cmd = thestr.to_string();
    };
    //println!("ping command! {}", thestr);
}
extern fn parse_store_dir(parse_results: *mut ParseResults, data: *const u8, len: size_t) {
    unsafe {
        let slice = slice::from_raw_parts(data, len - 1);
        let thestr = std::str::from_utf8(slice)
            .unwrap_or_else(|e| { panic!("{}", e); });

        (*parse_results).dir = thestr.to_string();
    };
    //println!("ping dir! {}", thestr);
}
extern fn parse_store_num(parse_results: *mut ParseResults, num: i32) {
    unsafe {
        (*parse_results).num = format!("{}", num);
    }
}

// /.

fn register_ffi() {
    unsafe {
        rh_parse_register_cb_store_num(parse_store_num);
        rh_parse_register_cb_store_command(parse_store_command);
        rh_parse_register_cb_store_dir(parse_store_dir);
        rh_parse_register_cb_store_cdata(parse_store_cdata);
    }
}
