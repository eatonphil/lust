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
    syms: &HashTable<String, usize>;
    instructions: &Vec<Instruction>;
}

fn compile_binary_operation(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, bop: BinaryOperation) {
    compile_expression(prgrm, raw, locals, bop.left);
    compile_expression(prgrm, raw, locals, bop.right);
    match bop.op {
	"+" => program.instructions.push(Instruction::Add),
	"-" => program.instructions.push(Instruction::Subtract),
	"<" => program.instructions.push(Instruction::LessThan),
	_ => panic!(bop.op.debug(raw, "Unable to compile binary operation:")),
    }
}

fn compile_function_call(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, fc: FunctionCall) {
    for arg in fc.args {
	compile_expression(pgrm, raw, locals, arg);
    }

    pgrm.instructions.push(Instruction::Call(fc.name));
}

fn compile_literal(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, lit: Literal) {
    match lit {
	Literal::Number(i) => pgrm.instructions.push(Instruction::Store(i));
	Literal::Identifier(ident) => pgrm.instructions.push(Instruction::DupPlusSP(locals[ident]));
    }
}

fn compile_expression(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, exp: Expression) {
    match exp {
	Expression::BinaryOperation(bop) => compile_binary_operation(pgrm, raw, locals, exp),
	Expression::FunctionCall(fc) => compile_function_call(pgrm, raw, locals, fc),
	Expression::Literal(lit) => compile_literal(pgrm, raw, locals, lit),
    }
}

fn compile_declaration(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, fd: FunctionDeclaration) {
    // Jump to end of function to guard top-level
    let done_label = format!("function_done_{}", pgrm.instructions.len());
    pgrm.instructions.push(Instruction::Jump(done_label));

    pgrm.syms[fd.name] = pgrm.instructions.len();
    for (i, param) in fd.parameters.iter() {
	pgrm.instructions.push(Instruction::DupMinusSP(fd.parameters.len() - (i + 1)));
	locals[param.name] = i;
    }

    for stmt in fd.body {
	compile_statement(pgrm, raw, locals, stmt);
    }

    pgrm.syms[done_label] = pgrm.instructions.len();
}

fn compile_return(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, ret: Return) {
    pgrm.instructions.push(Instruction::Return);
}

fn compile_if(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, if_: If) {
    compile_expression(pgrm, raw, locals, if_.test);
    let done_label = format!("if_else_{}", pgrm.instructions.len());
    pgrm.instructions.push(Instruction::JumpIfZero(done_label));
    for stmt in if_.body {
	compile_statement(pgrm, raw, locals, stmt);
    }
    pgrm.syms[done_label] = prgrm.instructions.len();
}

fn compile_local(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, local: Local) {
    locals[local.name] = prgm.instructions.len();
    compile_expression(pgrm, raw, locals, local.expression);
}

fn compile_statement(pgrm: &mut Program, raw: &Vec<char>, locals: &HashMap<String, usize>, stmt: Statement) {
    match stmt.kind {
	Statement::FunctionDeclaration(fd) => compile_declaration(pgrm, raw, locals, fd),
	Statement::Return(r) => compile_return(pgrm, raw, locals, r),
	Statement::If(fd) => compile_if(pgrm, raw, locals, fd),
	Statement::Local(fd) => compile_local(pgrm, raw, locals, fd),
	Statement::Expression(fd) => compile_expression(pgrm, raw, locals, fd.expression),
    }
}

fn compile(raw: &Vec<char>, locals: &HashMap<String, usize>, pgrm: AST) Vec<Instruction> {
    let mut pgrm = Program{};
    for stmt in pgrm {
	compile_statement(&pgrm, raw, locals, stmt);
    }

    pgrm
}

fn eval(pgrm: AST) {
    let mut pc = 0;
    let mut sp = 0;
    let mut calls: Vec<i32> = vec![];
    let mut data: Vec<i32> = vec![];

    while pc < pgrm.instructions.len() {
	match pgrm.instructions[pc] {
	    DupMinusSP(i) => data.push(data[sp - i as usize]),
	    DupPlusSP(i) => data.push(data[sp + i as usize]),
	    Return => {
		pc = calls.pop().unwrap();
		sp = pc;
	    },
	    JumpIfZero(label) => {
		let top = data.pop().unwrap();
		if top != 0 {
		    pc = pgrm.syms[label];
		}
	    },
	    Jump(label) => {
		pc = pgrm.syms[label];
	    },
	    Call(label) => {
		calls.push(sp);
		pc = pgrm.syms[label];
		sp = pc;
	    }
	    Add => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(left + right);
		pc += 1;
	    },
	    Subtract => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(left - right);
		pc += 1;
	    },
	    LessThan => {
		let left = data.pop().unwrap();
		let right = data.pop().unwrap();
		data.push(left < right ? 1 : 0);
		pc += 1;
	    },
	}
    }
}
