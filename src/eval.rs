use crate::parse::*;
use std::collections::HashMap;

enum Instruction {
    DupMinusSP(i32),
    DupPlusSP(i32),
    Store(i32),
    Return,
    JumpIfZero(String),
    Jump(String),
    Call(String),
    Add,
    Subtract,
    LessThan,
}

struct Program {
    syms: HashMap<String, i32>,
    instructions: Vec<Instruction>,
}

fn compile_binary_operation(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, bop: BinaryOperation) {
    compile_expression(pgrm, raw, locals, *bop.left);
    compile_expression(pgrm, raw, locals, *bop.right);
    match bop.operator.value.as_str() {
	"+" => {
	    pgrm.instructions.push(Instruction::Add);
	},
	"-" => {
	    pgrm.instructions.push(Instruction::Subtract);
	},

	"<" => {
	    pgrm.instructions.push(Instruction::LessThan);
	},
	_ => panic!("{}", bop.operator.loc.debug(*raw, "Unable to compile binary operation:")),
    }
}

fn compile_function_call(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, fc: FunctionCall) {
    for arg in fc.arguments {
	compile_expression(pgrm, raw, locals, arg);
    }

    pgrm.instructions.push(Instruction::Call(fc.name.value));
}

fn compile_literal(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, lit: Literal) {
    match lit {
	Literal::Number(i) => {
	    let int = i.value.parse::<i32>().unwrap();
	    pgrm.instructions.push(Instruction::Store(int));
	},
	Literal::Identifier(ident) => {
	    pgrm.instructions.push(Instruction::DupPlusSP(locals[&ident.value]));
	},
    }
}

fn compile_expression(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, exp: Expression) {
    match exp {
	Expression::BinaryOperation(bop) => {
	    compile_binary_operation(pgrm, raw, locals, bop);
	},
	Expression::FunctionCall(fc) => {
	    compile_function_call(pgrm, raw, locals, fc);
	},
	Expression::Literal(lit) => {
	    compile_literal(pgrm, raw, locals, lit);
	},
    }
}

fn compile_declaration(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, fd: FunctionDeclaration) {
    // Jump to end of function to guard top-level
    let done_label = format!("function_done_{}", pgrm.instructions.len());
    pgrm.instructions.push(Instruction::Jump(done_label));

    pgrm.syms[&fd.name.value] = pgrm.instructions.len() as i32;
    for (i, param) in fd.parameters.iter().enumerate() {
	pgrm.instructions.push(Instruction::DupMinusSP(fd.parameters.len() as i32 - (i as i32 + 1)));
	locals.insert(param.value, i as i32);
    }

    for stmt in fd.body {
	compile_statement(pgrm, raw, locals, stmt);
    }

    pgrm.syms[&done_label] = pgrm.instructions.len() as i32;
}

fn compile_return(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, ret: Return) {
    pgrm.instructions.push(Instruction::Return);
}

fn compile_if(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, if_: If) {
    compile_expression(pgrm, raw, locals, if_.test);
    let done_label = format!("if_else_{}", pgrm.instructions.len());
    pgrm.instructions.push(Instruction::JumpIfZero(done_label));
    for stmt in if_.body {
	compile_statement(pgrm, raw, locals, stmt);
    }
    pgrm.syms[&done_label] = pgrm.instructions.len() as i32;
}

fn compile_local(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, local: Local) {
    locals.insert(local.name.value, pgrm.instructions.len() as i32);
    compile_expression(pgrm, raw, locals, local.expression);
}

fn compile_statement(pgrm: &Program, raw: &Vec<char>, locals: &HashMap<String, i32>, stmt: Statement) {
    match stmt {
	Statement::FunctionDeclaration(fd) => compile_declaration(pgrm, raw, locals, fd),
	Statement::Return(r) => compile_return(pgrm, raw, locals, r),
	Statement::If(if_) => compile_if(pgrm, raw, locals, if_),
	Statement::Local(loc) => compile_local(pgrm, raw, locals, loc),
	Statement::Expression(e) => compile_expression(pgrm, raw, locals, e),
    }
}

pub fn compile(raw: &Vec<char>, ast: AST) -> Program {
    let locals: HashMap<String, i32> = HashMap::new();
    let pgrm = Program{
	syms: HashMap::new(),
	instructions: Vec::new(),
    };
    for stmt in ast {
	compile_statement(&pgrm, raw, &locals, stmt);
    }

    pgrm
}

pub fn eval(pgrm: Program) {
    let mut pc: i32 = 0;
    let mut sp: i32 = 0;
    let mut calls: Vec<i32> = vec![];
    let mut data: Vec<i32> = vec![];

    while pc < pgrm.instructions.len() as i32 {
	match &pgrm.instructions[pc as usize] {
	    Instruction::DupMinusSP(i) => data.push(data[(sp - i) as usize]),
	    Instruction::DupPlusSP(i) => data.push(data[(sp + i) as usize]),
	    Instruction::Return => {
		pc = calls.pop().unwrap();
		sp = pc;
	    },
	    Instruction::JumpIfZero(label) => {
		let top = data.pop().unwrap();
		if top != 0 {
		    pc = pgrm.syms[label];
		}
	    },
	    Instruction::Jump(label) => {
		pc = pgrm.syms[label];
	    },
	    Instruction::Call(label) => {
		calls.push(sp);
		pc = pgrm.syms[label];
		sp = pc;
	    }
	    Instruction::Add => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(left + right);
		pc += 1;
	    },
	    Instruction::Subtract => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(left - right);
		pc += 1;
	    },
	    Instruction::LessThan => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(if left < right { 1 } else { 0 });
		pc += 1;
	    },
	}
    }
}
