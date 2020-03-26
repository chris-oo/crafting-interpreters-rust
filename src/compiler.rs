use crate::scanner;

pub fn compile(source: &String) {
    let scanner = scanner::Scanner::new(source);
    let mut line = -1;

    loop {
        let token = scanner.scan_token();

        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("   | ");
        }

        // print!("{:2} '{}'\n", token.type, token.string);

        if token == Token::TokenEof {
            break;
        }
    }
}
