#[derive(Copy, Clone, Debug)]
pub struct Location {
    col: i32,
    line: i32,
    index: usize,
}

impl Location {
    fn increment(&self, newline: bool) -> Location {
	if newline {
	    Location{ index: self.index + 1, col: 0, line: self.line + 1 }
	} else {
	    Location{ index: self.index + 1, col: self.col + 1, line: self.line }
	}
    }

    pub fn debug<S: Into<String>>(&self, raw: &Vec<char>, msg: S) -> String {
	let mut line = 0;
	let mut line_str = String::new();
	// Find the whole line of original source
	for c in raw {
	    if *c == '\n' {
		line += 1;

		// Done discovering line in question
		if line_str.len() > 0 {
		    break;
		}

		continue;
	    }

	    if self.line == line {
		line_str.push_str(&c.to_string());
	    }
	}

	let space = " ".repeat(self.col as usize);
	format!("{}\n\n{}\n{}^ Near here", msg.into(), line_str, space)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Identifier,
    Syntax,
    Keyword,
    Number,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: String,
    pub kind: TokenKind,
    pub loc: Location,
}

fn lex_syntax(raw: &Vec<char>, initial_loc: Location) -> Option<(Token, Location)> {
    let syntax = [
    	";",
	"=",
	"+",
	"-",
	"<",
	"(",
	")",
    ];

    for possible_syntax in syntax {
	let c = raw[initial_loc.index];
	let next_loc = initial_loc.increment(false);
	// TODO: this won't work with multiple-character syntax bits like >= or ==
	if possible_syntax == c.to_string() {
	    return Some((Token{ value: possible_syntax.to_string(), loc: initial_loc, kind: TokenKind::Syntax }, next_loc));
	}
    }

    None
}

fn lex_keyword(raw: &Vec<char>, initial_loc: Location) -> Option<(Token, Location)> {
    let syntax = [
	"function",
	"end",
	"if",
	"then",
	"local",
	"return",
    ];

    let mut next_loc = initial_loc;
    let mut value = String::new();
    'outer: for possible_syntax in syntax {
	value = String::new();
	let mut c = raw[initial_loc.index];
	next_loc = initial_loc;
	while c.is_alphanumeric() || c == '_' {
	    value.push_str(&c.to_string());
	    next_loc = next_loc.increment(false);
	    c = raw[next_loc.index];

	    let n = next_loc.index - initial_loc.index;
	    if value[..n] != possible_syntax[..n] {
		continue 'outer;
	    }
	}

	// If it got to this point it found a match, so exit early.
	// We don't need a longest match.
	break;
    }

    // If the next character would be part of a valid identifier, then
    // this is not a keyword.
    if next_loc.index < raw.len() - 2 {
	let next_c = raw[next_loc.index+1];
	if next_c.is_alphanumeric() || next_c == '_' {
	    return None;
	}
    }

    Some((Token{ value: value, loc: initial_loc, kind: TokenKind::Keyword }, next_loc))
}

fn lex_identifier(raw: &Vec<char>, initial_loc: Location) -> Option<(Token, Location)> {
    let mut ident = String::new();
    let mut next_loc = initial_loc;
    let mut c = raw[initial_loc.index];
    while c.is_alphanumeric() || c == '_' {
	next_loc = next_loc.increment(false);
	ident.push_str(&c.to_string());
	c = raw[next_loc.index];
    }

    // First character must not be a digit
    if ident.len() > 0 && !ident.chars().nth(0).unwrap().is_digit(10) {
	Some((Token{ value: ident, loc: initial_loc, kind: TokenKind::Identifier }, next_loc))
    } else {
	None
    }
}

fn lex_number(raw: &Vec<char>, initial_loc: Location) -> Option<(Token, Location)> {
    let mut ident = String::new();
    let mut next_loc = initial_loc;
    let mut c = raw[initial_loc.index];
    while c.is_digit(10) {
	next_loc = next_loc.increment(false);
	ident.push_str(&c.to_string());
	c = raw[next_loc.index];
    }

    if ident.len() > 0 {
	Some((Token{ value: ident, loc: initial_loc, kind: TokenKind::Number }, next_loc))
    } else {
	None
    }
}


fn eat_whitespace(raw: &Vec<char>, initial_loc: Location) -> Location {
    let mut c = raw[initial_loc.index];
    let mut next_loc = initial_loc;
    while [' ', '\n', '\r', '\t'].contains(&c) {
	next_loc = next_loc.increment(c == '\n');
	c = raw[next_loc.index];
    }

    next_loc
}

pub fn lex(s: &Vec<char>) -> Result<Vec<Token>, String> {
    let mut loc = Location{col: 0, index: 0, line: 0};
    let size = s.len();
    let mut tokens: Vec<Token> = vec![];

    let lexers = [lex_keyword, lex_number, lex_identifier, lex_syntax];
    'outer: while loc.index < size {
	loc = eat_whitespace(s, loc);

	for lexer in lexers {
	    let res = lexer(s, loc);
	    if res.is_some() {
		let (t, next_loc) = res.unwrap();
		loc = next_loc;
		tokens.push(t);
		continue 'outer;
	    }
	}

	return Err(loc.debug(s, "Unrecognized character while lexing:"));
    }

    Ok(tokens)
}
