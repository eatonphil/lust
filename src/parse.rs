#[derive(Debug)]
enum Literal {
    Identifier(Token),
    Number(Token),
}

#[derive(Debug)]
struct FunctionCall {
    name: Token,
    arguments: Vec<Expression>,
}

#[derive(Debug)]
struct BinaryOperation {
    operator: Token,
    left: Expression,
    right: Expression,
}

#[derive(Debug)]
enum Expression {
    FunctionCall(FunctionCall),
    BinaryOperation(BinaryOperation),
    Literal(Literal),
}

#[derive(Debug)]
struct FunctionDeclaration {
    name: Token,
    parameters: Vec<Token>,
    body: Vec<Statement>,
}

#[derive(Debug)]
struct If {
    test: Expression,
    body: Vec<Statement>,
}

#[derive(Debug)]
struct Local {
    name: Token,
    expression: Expression,
}

#[derive(Debug)]
struct Return {
    expression: Expression,
}

#[derive(Debug)]
enum Statement {
    Expression(Expression),
    If(If),
    FunctionDeclaration(FunctionDeclaration),
    Return(Return),
    Local(Local),
}

type AST = Vec<Statement>;

fn expect_keyword(tokens: Vec<Token>, index: i32, value: &str) bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Keyword && t.value == value;
}

fn expect_syntax(tokens: Vec<Token>, index: i32, value: &str) bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Syntax && t.value == value;
}

fn expect_identifier(tokens: Vec<Token>, index: i32) bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Identifier;
}

fn expect_number(tokens: Vec<Token>, index: i32) bool {
    if index >= tokens.len() {
	return false;
    }

    let t = tokens[index];
    return t.kind == TokenKind::Number;
}

fn parse_expression(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<Expression> {
    if !expect_identifier(tokens, index) || expect_number(tokens, index) {
	return None;
    }

    let left = tokens[index];
    let mut next_index = index + 1;
    if expect_syntax(tokens, next_index, "(") {
	next_index += 1; // Skip past open paren

	// Function call
	let mut arguments: Vec<Expression> = vec![];
	while !expect_syntax(tokens, next_index, ")") {
	    if arguments.len() > 0 {
		if !expect_syntax(tokens, next_index, ",") {
		    println!(tokens[next_index].debug(raw, "Expected comma between function call arguments:"));
		    return None;
		}

		next_index += 1; // Skip past comma
	    }

	    let res = parse_expression(tokens, next_index);
	    if res.is_some() {
		let (arg, next_next_index) = res.unwrap();
		next_index = next_next_index;
		arguments.push(arg);
	    } else {
		println!(tokens[next_index].debug(raw, "Expected valid expression in function call arguments:"));
		return None;
	    }
	}

	next_index += 1; // Skip past closing paren

	return Some(Expression::FunctionCall(FunctionCall{}), next_index)
    }

    // Otherwise is a binary operation
    if next_index >= tokens.len() || tokens[next_index].kind != TokenKind::Syntax {
	println!(tokens[next_index].debug(raw, "Expected valid binary operation:"));
	return None;
    }

    let op = tokens[next_index];
    next_index += 1; // Skip past op

    if !expect_identifier(tokens, next_index) || !expect_number(tokens, next_index) {
	println!(tokens[next_index].debug(raw, "Expected valid right hand side binary operand:"));
	return None;
    }

    let right = tokens[next_index];
    next_index += 1; // Skip past right hand operand

    Some(Expression::BinaryOperation(BinaryOperation{left: left, right: right, op: op}), next_index)
}

fn parse_function(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<FunctionDeclaration> {
    if !expect_keyword(tokens, index, "function") {
	return None;
    }

    let mut next_index = index + 1;
    if !expect_identifier(tokens, next_index) {
	println!(tokens[next_index].debug(raw, "Expected valid identifier for function name:"));
	return None;
    }
    let name = tokens[next_index];

    next_index += 1; // Skip past name
    if !expect_syntax(tokens, next_index, "(") {
	println!(tokens[next_index].debug(raw, "Expected open parenthesis in function declaration:"));
	return None;
    }

    next_index += 1; // Skip past open paren
    let parameters: Vec<Token> = vec![];
    while !expect_syntax(")") {
	if parameters.len() > 0 {
	    if !expect_syntax(tokens, next_index, ",") {
		println!(tokens[next_index].debug(raw, "Expected comma or close parenthesis after parameter in function declaration:"));
		return None;
	    }

	    next_index += 1; // Skip past comma
	}

	parameters.push(tokens[next_index]);
    }

    next_index += 1; // Skip past close paren

    let statements: Vec<Statement> = vec![];
    while !expect_keyword(tokens, next_index, "end") {
	let res = parse_statement(raw, tokens, next_index);
	if res.is_some() {
	    let (stmt, next_next_index) = res.unwrap();
	    next_index = next_next_index;
	    statements.push(stmt);
	} else {
	    println!(tokens[next_index].debug(raw, "Expected valid statement in function declaration:"));
	    return None;
	}
    }

    next_index += 1; // Skip past end

    Some(Statement::FunctionDeclaration(FunctionDeclaration{
	name: name,
	parameters: parameters,
	body: statements,
    }), next_index)
}

fn parse_return(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<(Statement, i32)> {
    if !expect_keyword(tokens, index, "return") {
	return None;
    }

    let mut next_index = index + 1; // Skip past return
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!(tokens[next_index].debug(raw, "Expected valid expression in return statement:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;
    if !expect_syntax(tokens, next_index, ";") {
	println!(tokens[next_index].debug(raw, "Expected semicolon in return statement:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some(Statement::Return(Return{expression: expr}), next_index)
}

fn parse_local(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<(Statement, i32)> {
    if !expect_keyword(tokens, index, "local") {
	return None;
    }

    let mut next_index = index + 1; // Skip past local

    if !expect_identifier(tokens, next_index) {
	println!(tokens[next_index].debug(raw, "Expected valid identifier for function name:"));
	return None;
    }

    let name = tokens[next_index];
    next_index += 1; // Skip past name

    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!(tokens[next_index].debug(raw, "Expected valid expression in local declaration:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;

    if !expect_syntax(tokens, next_index, ";") {
	println!(tokens[next_index].debug(raw, "Expected semicolon in return statement:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some(Statement::Local(Local{name: name, expression: expr}), next_index)
}

fn parse_if(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<(Statement, i32)> {
    if !expect_keyword(tokens, index, "if") {
	return None;
    }

    let mut next_index = index + 1; // Skip past if
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!(tokens[next_index].debug(raw, "Expected valid expression for if test:"));
	return None;
    }

    let (test, next_next_index) = res.unwrap();
    next_index = next_next_index;

    if !expect_keyword(tokens, next_index, "then") {
	return None;
    }

    next_index += 1; // Skip past then

    let statements: Vec<Statement> = vec![];
    while !expect_keyword(tokens, next_index, "end") {
	let res = parse_statement(raw, tokens, next_index);
	if res.is_some() {
	    let (stmt, next_next_index) = res.unwrap();
	    next_index = next_next_index;
	    statements.push(stmt);
	} else {
	    println!(tokens[next_index].debug(raw, "Expected valid statement in if body:"));
	    return None;
	}
    }

    next_index += 1; // Skip past end

    Some(Statement::If(If{test: test, body: statements}), next_index)
}

fn parse_expression_statement(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<(Statement, i32)> {
    let mut next_index = index;
    let res = parse_expression(raw, tokens, next_index);
    if !res.is_some() {
	println!(tokens[next_index].debug(raw, "Expected valid expression in statement:"));
	return None;
    }

    let (expr, next_next_index) = res.unwrap();
    next_index = next_next_index;
    if !expect_syntax(tokens, next_index, ";") {
	println!(tokens[next_index].debug(raw, "Expected semicolon after expression:"));
	return None;
    }

    next_index += 1; // Skip past semicolon

    Some(Statement::Expression(expr), next_index)
}

fn parse_statement(raw: &Vec<char>, tokens: Vec<Token>, index: i32) -> Option<(Statement, i32)> {
    let parsers = [parse_if, parse_expression_statement, parse_return, parse_function, parse_local];
    for parser in parsers {
	let res = parser(raw, tokens, index);
	if res.is_some() {
	    return res;
	}
    }

    None
}

pub fn parse(raw: &Vec<char>, tokens: Vec<Token>) -> Result<AST, String> {
    let AST = AST{};
    let mut index = 0;
    'outer: while index < tokens.len() {
	for parser in parsers {
	    let res = parser(raw, tokens, index);
	    if res.is_some() {
		let (stmt, next_index) = res.unwrap();
		index = next_index;
		ast.push(stmt);
		continue 'outer;
	    }
	}

	return Err(loc.debug(raw, "Invalid token while parsing:"));
    }

    Ok(tokens)
}
