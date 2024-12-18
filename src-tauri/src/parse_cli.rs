use anyhow::{bail, Result};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ParsedArguments {
    pub command: MainCommand,
    pub handle: Option<usize>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MainCommand {
    Show,
    Preview,
    Config,
}

// Here goes the specification. We need to support one of the following arguments:
// /p:handle -- show the preview into given handle
// /s        -- show the main event fullscreen
// /s:handle -- show the main event into a given handle
// /c        -- show the configuration window
// /c:handle -- no idea
//
// Now, the trouble part. The arguments may be either in a lower or upper case.
// The handle may be separated by either ":" or space.
// Another day with the great standartisation by Micro$oft.

///Parses command line arfuments. Returns Err() if arguments are malformed.
pub fn parse_args(args_in: &[String]) -> Result<ParsedArguments> {
    //Lowercasing everything:
    let args: Vec<String> = args_in.iter().map(|s| s.to_lowercase()).collect();

    //If parsing goes wrong, just return Err;
    //With no args or wrong amount of args, abort parsing.
    if args.len() > 3 || args.is_empty() {
        bail!("Wrong number of arguments");
    }
    //No arguments at all is acceptable
    if args.len() == 1 {
        return Ok(ParsedArguments {
            command: MainCommand::Show,
            handle: None,
        });
    }

    //If argument 2 exists, parse it to handle
    let handle1: Option<usize> = match args.get(2) {
        Some(s) => match s.parse() {
            Ok(0_usize) | Err(_) => bail!("Can't parse argument 2"),
            Ok(n) => Some(n),
        },
        None => None,
    };

    //Parse argument 1
    let command_chars: Vec<char> = match args.get(1) {
        Some(s) => s.chars().collect(),
        None => bail!("Can't prepare argument 1"),
    };

    if command_chars.is_empty() || command_chars[0] != '/' {
        bail!("Argument 1 bad form: {command_chars:?}");
    }
    //Check if argument 1 has a handle provided after :
    let handle2;
    if command_chars.len() > 2 {
        if command_chars[2] != ':' {
            bail!("Argument 1 bad form: {command_chars:?}");
        } else {
            let s: String = command_chars[3..].iter().collect();
            handle2 = match s.parse() {
                Ok(0_usize) | Err(_) => {
                    bail!("Can't parse handle: {command_chars:?}")
                }
                Ok(n) => Some(n),
            }
        }
    } else {
        handle2 = None;
    }
    //If both handles are set, it is a parse error
    if handle1.is_some() && handle2.is_some() {
        bail!("Provided 2 handles together");
    }
    let handle = handle1.or(handle2);
    //Parse second letter of arg1 to determine command
    let command = match command_chars[1] {
        'c' => MainCommand::Config,
        's' => MainCommand::Show,
        'p' => MainCommand::Preview,
        _ => bail!("Unknown command letter"),
    };

    //Preview should not be passed without a handle
    if let MainCommand::Preview = command {
        if handle.is_none() {
            bail!("Preview should not be passed with a handle");
        }
    }

    Ok(ParsedArguments { command, handle })
}
