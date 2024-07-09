use boba_script::{
    core::{engine::Value, Engine},
    lexer::{BobaCache, Lexer},
    parser::{parsers::statement, Token, TokenStream},
};
use boba_script_ariadne::ToAriadne;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

pub fn session() {
    let mut editor = Reedline::create();
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic(format!("boba ")),
        DefaultPromptSegment::Empty,
    );

    let mut engine = Engine::new();
    let mut cache = BobaCache::new();
    loop {
        let data = match editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => cache.store("shell", buffer),
            Ok(Signal::CtrlD) => {
                println!("Closing Shell...");
                return;
            }
            Ok(Signal::CtrlC) => {
                println!("Aborting...");
                return;
            }
            Err(e) => {
                eprintln!("Input Error: {e}");
                continue;
            }
        };

        let mut parser = Lexer::new(data).stream_parser();
        let _indent = match parser.peek() {
            // if there are no tokens, then do nothing and try again
            None => continue,
            // if we find an error, print the error and try again
            Some(Err(error)) => {
                error.to_ariadne().eprint(&mut cache).unwrap();
                continue;
            }
            // if the first token is an indent, then get the indent level
            Some(Ok(Token::Indent)) => parser.stream().indent_level(),
            // if the first token is something else, then the indent level is 0
            Some(_) => 0,
        };

        match statement::parse(&mut parser) {
            Ok(statement) => match engine.eval(statement) {
                Ok(Value::None) => continue, // do nothing with none
                Ok(value) => println!("{value}"),
                Err(error) => error.to_ariadne().eprint(&mut cache).unwrap(),
            },
            Err(errors) => {
                for error in errors {
                    error.to_ariadne().eprint(&mut cache).unwrap()
                }
            }
        }
    }
}
