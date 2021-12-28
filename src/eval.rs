use crate::parse::*;
use std::collections::HashMap;

#[derive(Debug)]
enum Instruction {
    DupPlusSP(i32),
    MoveMinusSP(usize, i32),
    MovePlusSP(usize),
    Store(i32),
    Return,
    JumpIfNotZero(String),
    Jump(String),
    Call(String, usize),
    Add,
    Subtract,
    LessThan,
}

#[derive(Debug)]
struct Symbol {
    location: i32,
    narguments: usize,
    nlocals: usize,
}

#[derive(Debug)]
pub struct Program {
    syms: HashMap<String, Symbol>,
    instructions: Vec<Instruction>,
}

fn compile_binary_operation(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    bop: BinaryOperation,
) {
    compile_expression(pgrm, raw, locals, *bop.left);
    compile_expression(pgrm, raw, locals, *bop.right);
    match bop.operator.value.as_str() {
        "+" => {
            pgrm.instructions.push(Instruction::Add);
        }
        "-" => {
            pgrm.instructions.push(Instruction::Subtract);
        }

        "<" => {
            pgrm.instructions.push(Instruction::LessThan);
        }
        _ => panic!(
            "{}",
            bop.operator
                .loc
                .debug(raw, "Unable to compile binary operation:")
        ),
    }
}

fn compile_function_call(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    fc: FunctionCall,
) {
    let len = fc.arguments.len();
    for arg in fc.arguments {
        compile_expression(pgrm, raw, locals, arg);
    }

    pgrm.instructions
        .push(Instruction::Call(fc.name.value, len));
}

fn compile_literal(
    pgrm: &mut Program,
    _: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    lit: Literal,
) {
    match lit {
        Literal::Number(i) => {
            let n = i.value.parse::<i32>().unwrap();
            pgrm.instructions.push(Instruction::Store(n));
        }
        Literal::Identifier(ident) => {
            pgrm.instructions
                .push(Instruction::DupPlusSP(locals[&ident.value]));
        }
    }
}

fn compile_expression(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    exp: Expression,
) {
    match exp {
        Expression::BinaryOperation(bop) => {
            compile_binary_operation(pgrm, raw, locals, bop);
        }
        Expression::FunctionCall(fc) => {
            compile_function_call(pgrm, raw, locals, fc);
        }
        Expression::Literal(lit) => {
            compile_literal(pgrm, raw, locals, lit);
        }
    }
}

fn compile_declaration(
    pgrm: &mut Program,
    raw: &Vec<char>,
    _: &mut HashMap<String, i32>,
    fd: FunctionDeclaration,
) {
    // Jump to end of function to guard top-level
    let done_label = format!("function_done_{}", pgrm.instructions.len());
    pgrm.instructions
        .push(Instruction::Jump(done_label.clone()));

    let mut new_locals = HashMap::<String, i32>::new();

    let function_index = pgrm.instructions.len() as i32;
    let narguments = fd.parameters.len();
    for (i, param) in fd.parameters.iter().enumerate() {
        pgrm.instructions.push(Instruction::MoveMinusSP(
            i,
            narguments as i32 - (i as i32 + 1),
        ));
        new_locals.insert(param.value.clone(), i as i32);
    }

    for stmt in fd.body {
        compile_statement(pgrm, raw, &mut new_locals, stmt);
    }

    // Overwrite function lookup with total number of locals
    pgrm.syms.insert(
        fd.name.value,
        Symbol {
            location: function_index as i32,
            narguments: narguments,
            nlocals: new_locals.keys().len(),
        },
    );

    pgrm.syms.insert(
        done_label,
        Symbol {
            location: pgrm.instructions.len() as i32,
            narguments: 0,
            nlocals: 0,
        },
    );
}

fn compile_return(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    ret: Return,
) {
    compile_expression(pgrm, raw, locals, ret.expression);
    pgrm.instructions.push(Instruction::Return);
}

fn compile_if(pgrm: &mut Program, raw: &Vec<char>, locals: &mut HashMap<String, i32>, if_: If) {
    compile_expression(pgrm, raw, locals, if_.test);
    let done_label = format!("if_else_{}", pgrm.instructions.len());
    pgrm.instructions
        .push(Instruction::JumpIfNotZero(done_label.clone()));
    for stmt in if_.body {
        compile_statement(pgrm, raw, locals, stmt);
    }
    pgrm.syms.insert(
        done_label,
        Symbol {
            location: pgrm.instructions.len() as i32 - 1,
            nlocals: 0,
            narguments: 0,
        },
    );
}

fn compile_local(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    local: Local,
) {
    let index = locals.keys().len();
    locals.insert(local.name.value, index as i32);
    compile_expression(pgrm, raw, locals, local.expression);
    pgrm.instructions.push(Instruction::MovePlusSP(index));
}

fn compile_statement(
    pgrm: &mut Program,
    raw: &Vec<char>,
    locals: &mut HashMap<String, i32>,
    stmt: Statement,
) {
    match stmt {
        Statement::FunctionDeclaration(fd) => compile_declaration(pgrm, raw, locals, fd),
        Statement::Return(r) => compile_return(pgrm, raw, locals, r),
        Statement::If(if_) => compile_if(pgrm, raw, locals, if_),
        Statement::Local(loc) => compile_local(pgrm, raw, locals, loc),
        Statement::Expression(e) => compile_expression(pgrm, raw, locals, e),
    }
}

pub fn compile(raw: &Vec<char>, ast: AST) -> Program {
    let mut locals: HashMap<String, i32> = HashMap::new();
    let mut pgrm = Program {
        syms: HashMap::new(),
        instructions: Vec::new(),
    };
    for stmt in ast {
        compile_statement(&mut pgrm, raw, &mut locals, stmt);
    }

    pgrm
}

pub fn eval(pgrm: Program) {
    let mut pc: i32 = 0;
    let mut sp: i32 = 0;
    let mut data: Vec<i32> = vec![];

    while pc < pgrm.instructions.len() as i32 {
        match &pgrm.instructions[pc as usize] {
            Instruction::DupPlusSP(i) => {
                data.push(data[(sp + i) as usize]);
                pc += 1;
            }
            Instruction::MoveMinusSP(local_offset, sp_offset) => {
                data[sp as usize + local_offset] = data[(sp - (sp_offset + 4)) as usize];
                pc += 1;
            }
            Instruction::MovePlusSP(i) => {
                let val = data.pop().unwrap();
                let index = sp as usize + *i;
                // Accounts for top-level locals
                while index >= data.len() {
                    data.push(0);
                }
                data[index] = val;
                pc += 1;
            }
            Instruction::JumpIfNotZero(label) => {
                let top = data.pop().unwrap();
                if top == 0 {
                    pc = pgrm.syms[label].location;
                }
                pc += 1;
            }
            Instruction::Jump(label) => {
                pc = pgrm.syms[label].location;
            }
            Instruction::Return => {
                let ret = data.pop().unwrap();

                // Clean up the local stack
                while sp < data.len() as i32 {
                    data.pop();
                }

                // Restore pc and sp
                let mut narguments = data.pop().unwrap();
                pc = data.pop().unwrap();
                sp = data.pop().unwrap();

                // Clean up arguments
                while narguments > 0 {
                    data.pop();
                    narguments -= 1;
                }

                // Add back return value
                data.push(ret);
            }
            Instruction::Call(label, narguments) => {
                // Handle builtin functions
                if label == "print" {
                    for _ in 0..*narguments {
                        print!("{}", data.pop().unwrap());
                        print!(" ");
                    }
                    println!("");
                    pc += 1;
                    continue;
                }

                data.push(sp);
                data.push(pc + 1);
                data.push(pgrm.syms[label].narguments as i32);
                pc = pgrm.syms[label].location;
                sp = data.len() as i32;

                // Set up space for all arguments/locals
                let mut nlocals = pgrm.syms[label].nlocals;
                while nlocals > 0 {
                    data.push(0);
                    nlocals -= 1;
                }
            }
            Instruction::Add => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left + right);
                pc += 1;
            }
            Instruction::Subtract => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(left - right);
                pc += 1;
            }
            Instruction::LessThan => {
                let right = data.pop().unwrap();
                let left = data.pop().unwrap();
                data.push(if left < right { 1 } else { 0 });
                pc += 1;
            }
            Instruction::Store(n) => {
                data.push(*n);
                pc += 1;
            }
        }
    }
}
