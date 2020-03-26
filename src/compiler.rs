use crate::scanner;

pub fn compile(source: &String) {
    let mut scanner = scanner::Scanner::new(source);
    let mut line = -1;

    loop {
        let token = scanner.scan_token();

        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        print!("{:?} '{}'\n", token.token_type, token.string);

        // TODO - Is there any way to not use match here?
        match token.token_type {
            scanner::TokenType::TokenEof => {
                return;
            }
            _ => (),
        }
    }
}
