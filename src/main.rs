extern crate num;
#[macro_use]
extern crate num_derive;

mod bytecode;
mod chunk;
mod compiler;
mod scanner;
mod vm;

use std::env;
use std::fs;
use std::io;
use std::process;

fn repl() {
    let mut vm = vm::VM::new();

    let mut line = String::new();

    loop {
        print!("> ");

        match io::stdin().read_line(&mut line) {
            Ok(n) => {
                if n == 0 {
                    println!("EOF, exiting...");
                    process::exit(0);
                }

                vm.interpret(&line);

                line.clear();
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

fn run_file(filename: &String) {
    let mut vm = vm::VM::new();

    let file = fs::read_to_string(filename).expect("Error reading file");

    match vm.interpret(&file) {
        vm::InterpretResult::InterpretOk => {}
        vm::InterpretResult::InterpretCompileError => {
            eprintln!("Compiler error reading file!");
            process::exit(65);
        }
        vm::InterpretResult::InterpretRuntimeError => {
            eprintln!("Runtime error executing file!");
            process::exit(70);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            repl();
        }
        2 => {
            run_file(&args[1]);
        }
        _ => {
            eprintln!("Usage: clox [path]");
            process::exit(64);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_test() {
        assert_eq!(2 + 2, 4);
    }
}
