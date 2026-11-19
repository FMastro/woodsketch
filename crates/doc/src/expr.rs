//! Arithmetic for recipes, so a part's size can be worked out rather than written down: a
//! cabinet whose panel is `width - 2*stile` resizes by changing one number instead of twenty.
//!
//! Deliberately small. Numbers, the four operators, parentheses, a comparison, and seven
//! functions. No strings, no assignment, nothing that runs twice. A length a person would type
//! (`3/4"`, `1 1/2"`, `2'6`) is read by `units::parse_length` before anything here is tried, so
//! the fractions and feet people write keep working and never have to mean division; inside an
//! expression a length is a plain number (`0.75`) or carries a simple unit (`300mm`, `24in`).

use std::collections::HashMap;

use crate::units::parse_length;

/// The functions a recipe may call. `if` is here because "two doors over 24 inches" is a rule
/// cabinets actually have, and a component written without it would have to be written again.
const FUNCTIONS: [&str; 7] = ["min", "max", "round", "floor", "ceil", "abs", "if"];

type Names = HashMap<String, f64>;

/// Work out `text`, with `names` holding every parameter's value in inches. A bare number is in
/// the recipe's own units, so `mm` says which those are.
pub fn eval(text: &str, mm: bool, names: &Names) -> Result<f64, String> {
    let tokens = scan(text, mm)?;
    let mut at = 0;
    let value = compare(&tokens, &mut at, names)?;
    match tokens.get(at) {
        None => Ok(value),
        Some(extra) => Err(format!("\"{text}\" doesn't read as a sum ({} is left over).", extra.say())),
    }
}

/// The parameters `text` reads, so they can be worked out before it is. Function names and
/// numbers are not parameters.
pub fn names_in(text: &str, mm: bool) -> Vec<String> {
    let Ok(tokens) = scan(text, mm) else { return Vec::new() };
    let mut found = Vec::new();
    for (i, token) in tokens.iter().enumerate() {
        if let Token::Name(name) = token {
            // A name followed by a bracket is a function, not something to look up.
            if tokens.get(i + 1) != Some(&Token::Open) && !found.contains(name) {
                found.push(name.clone());
            }
        }
    }
    found
}

/// Whether `text` is worth trying as a sum at all: it mentions something by name, or works on
/// two numbers. Without this, a typo in a plain length would be reported as bad arithmetic.
pub fn looks_like_a_sum(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_alphabetic() || "+*/()<>=".contains(c))
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Number(f64),
    Name(String),
    Open,
    Close,
    Comma,
    Op(char),
    /// `>=`, `<=`, `==` and `!=`, kept as the two characters they were written with.
    Pair([char; 2]),
}

impl Token {
    fn say(&self) -> String {
        match self {
            Token::Number(n) => format!("{n}"),
            Token::Name(name) => name.clone(),
            Token::Open => "(".into(),
            Token::Close => ")".into(),
            Token::Comma => ",".into(),
            Token::Op(c) => c.to_string(),
            Token::Pair([a, b]) => format!("{a}{b}"),
        }
    }
}

fn scan(text: &str, mm: bool) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
        } else if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            // A unit stuck to the number (300mm, 24in, 18"): the app's own reader takes it, so
            // only one place has to know what a unit is called.
            let plain_until = i;
            while i < chars.len() && (chars[i].is_ascii_alphabetic() || chars[i] == '"') {
                i += 1;
            }
            let number: String = chars[start..plain_until].iter().collect();
            let unit: String = chars[plain_until..i].iter().collect();
            let value = if unit.is_empty() {
                let plain: f64 = number.parse().map_err(|_| format!("\"{number}\" isn't a number."))?;
                plain / if mm { 25.4 } else { 1.0 }
            } else {
                parse_length(&format!("{number}{unit}")).map_err(|e| format!("\"{number}{unit}\": {e}"))?
            };
            tokens.push(Token::Number(value));
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push(Token::Name(chars[start..i].iter().collect()));
        } else if "><=!".contains(c) && chars.get(i + 1) == Some(&'=') {
            tokens.push(Token::Pair([c, '=']));
            i += 2;
        } else {
            tokens.push(match c {
                '(' => Token::Open,
                ')' => Token::Close,
                ',' => Token::Comma,
                '+' | '-' | '*' | '/' | '>' | '<' => Token::Op(c),
                other => return Err(format!("\"{other}\" isn't anything a sum can do.")),
            });
            i += 1;
        }
    }
    if tokens.is_empty() {
        return Err("there is nothing to work out.".into());
    }
    Ok(tokens)
}

/// `a > b` and friends, which give 1 or 0 so `if` has something to ask.
fn compare(tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    let left = sum(tokens, at, names)?;
    let test = match tokens.get(*at) {
        Some(Token::Op('>')) => ['>', ' '],
        Some(Token::Op('<')) => ['<', ' '],
        Some(Token::Pair(pair)) => *pair,
        _ => return Ok(left),
    };
    *at += 1;
    let right = sum(tokens, at, names)?;
    let yes = match test {
        ['>', ' '] => left > right,
        ['<', ' '] => left < right,
        ['>', '='] => left >= right,
        ['<', '='] => left <= right,
        ['=', '='] => (left - right).abs() < 1e-9,
        ['!', '='] => (left - right).abs() >= 1e-9,
        other => return Err(format!("\"{}{}\" isn't a comparison.", other[0], other[1])),
    };
    Ok(f64::from(u8::from(yes)))
}

fn sum(tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    let mut value = product(tokens, at, names)?;
    while let Some(Token::Op(op @ ('+' | '-'))) = tokens.get(*at) {
        let op = *op;
        *at += 1;
        let next = product(tokens, at, names)?;
        value = if op == '+' { value + next } else { value - next };
    }
    Ok(value)
}

fn product(tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    let mut value = unary(tokens, at, names)?;
    while let Some(Token::Op(op @ ('*' | '/'))) = tokens.get(*at) {
        let op = *op;
        *at += 1;
        let next = unary(tokens, at, names)?;
        if op == '/' && next.abs() < 1e-12 {
            return Err("that divides by nothing.".into());
        }
        value = if op == '*' { value * next } else { value / next };
    }
    Ok(value)
}

fn unary(tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    if tokens.get(*at) == Some(&Token::Op('-')) {
        *at += 1;
        return Ok(-unary(tokens, at, names)?);
    }
    atom(tokens, at, names)
}

fn atom(tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    match tokens.get(*at) {
        Some(Token::Number(n)) => {
            *at += 1;
            Ok(*n)
        }
        Some(Token::Open) => {
            *at += 1;
            let value = compare(tokens, at, names)?;
            if tokens.get(*at) != Some(&Token::Close) {
                return Err("a bracket is left open.".into());
            }
            *at += 1;
            Ok(value)
        }
        Some(Token::Name(name)) => {
            let name = name.clone();
            *at += 1;
            if tokens.get(*at) == Some(&Token::Open) {
                return call(&name, tokens, at, names);
            }
            match names.get(&name) {
                Some(value) => Ok(*value),
                None if FUNCTIONS.contains(&name.as_str()) => Err(format!("{name} needs its numbers in brackets: {name}(...).")),
                None => {
                    let mut known: Vec<&str> = names.keys().map(String::as_str).collect();
                    known.sort_unstable();
                    Err(if known.is_empty() {
                        format!("there is no parameter \"{name}\" (this recipe has none).")
                    } else {
                        format!("there is no parameter \"{name}\" (it has: {}).", known.join(", "))
                    })
                }
            }
        }
        Some(other) => Err(format!("{} isn't a number.", other.say())),
        None => Err("a sum stops in the middle.".into()),
    }
}

fn call(name: &str, tokens: &[Token], at: &mut usize, names: &Names) -> Result<f64, String> {
    *at += 1; // the bracket
    let mut args = Vec::new();
    if tokens.get(*at) != Some(&Token::Close) {
        loop {
            args.push(compare(tokens, at, names)?);
            if tokens.get(*at) == Some(&Token::Comma) {
                *at += 1;
                continue;
            }
            break;
        }
    }
    if tokens.get(*at) != Some(&Token::Close) {
        return Err(format!("{name}(...) is left open."));
    }
    *at += 1;
    let wrong = |wanted: &str| Err(format!("{name} takes {wanted}, not {}.", args.len()));
    match (name, args.len()) {
        ("min", 2) => Ok(args[0].min(args[1])),
        ("max", 2) => Ok(args[0].max(args[1])),
        ("round", 1) => Ok(args[0].round()),
        ("floor", 1) => Ok(args[0].floor()),
        ("ceil", 1) => Ok(args[0].ceil()),
        ("abs", 1) => Ok(args[0].abs()),
        // Zero is no and anything else is yes, since every comparison here gives 1 or 0.
        ("if", 3) => Ok(if args[0].abs() > 1e-9 { args[1] } else { args[2] }),
        ("min" | "max", _) => wrong("two numbers"),
        ("round" | "floor" | "ceil" | "abs", _) => wrong("one number"),
        ("if", _) => wrong("a test and two numbers"),
        _ => Err(format!("there is no function \"{name}\" ({}).", FUNCTIONS.join(", "))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(pairs: [(&str, f64); 3]) -> Names {
        pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
    }

    #[test]
    fn a_size_is_worked_out_from_the_parameters() {
        let names = names([("width", 24.0), ("stile", 2.25), ("height", 30.0)]);
        let eval = |text: &str| eval(text, false, &names);
        assert_eq!(eval("width - 2*stile"), Ok(19.5));
        assert_eq!(eval("(width + height) / 2"), Ok(27.0));
        assert_eq!(eval("-stile + width"), Ok(21.75));
        assert_eq!(eval("width"), Ok(24.0));
        assert_eq!(eval("3.5"), Ok(3.5));
    }

    #[test]
    fn a_number_is_in_the_recipes_own_units_unless_it_says_otherwise() {
        let names = names([("width", 24.0), ("stile", 2.25), ("height", 30.0)]);
        // In a millimeter recipe a bare number is millimeters; a unit written down wins.
        assert_eq!(eval("300", true, &names), Ok(300.0 / 25.4));
        assert_eq!(eval("300mm", false, &names), Ok(300.0 / 25.4));
        assert_eq!(eval("300mm", true, &names), Ok(300.0 / 25.4));
        assert_eq!(eval("2in", true, &names), Ok(2.0));
    }

    #[test]
    fn a_count_can_follow_a_size() {
        let names = names([("height", 60.0), ("stile", 2.25), ("width", 24.0)]);
        // A shelf every 12", and the door count a cabinet rule would use.
        assert_eq!(eval("floor(height / 12)", false, &names), Ok(5.0));
        assert_eq!(eval("if(width > 24, 2, 1)", false, &names), Ok(1.0));
        assert_eq!(eval("if(width >= 24, 2, 1)", false, &names), Ok(2.0));
        assert_eq!(eval("max(1, floor(width / 18))", false, &names), Ok(1.0));
        assert_eq!(eval("round(2.6)", false, &names), Ok(3.0));
        assert_eq!(eval("abs(0 - stile)", false, &names), Ok(2.25));
    }

    #[test]
    fn arithmetic_goes_in_the_order_a_person_expects() {
        let names = names([("width", 24.0), ("stile", 2.25), ("height", 30.0)]);
        let eval = |text: &str| eval(text, false, &names);
        // Times before plus, and both before a comparison, or "width - 2*stile" is the wrong
        // size and nobody would notice until a part came out short.
        assert_eq!(eval("2 + 3 * 4"), Ok(14.0));
        assert_eq!(eval("(2 + 3) * 4"), Ok(20.0));
        assert_eq!(eval("2 * 3 + 4 * 5"), Ok(26.0));
        // Taken left to right, so 10 - 3 - 2 is 5 and not 9.
        assert_eq!(eval("10 - 3 - 2"), Ok(5.0));
        assert_eq!(eval("24 / 2 / 3"), Ok(4.0));
        assert_eq!(eval("24 / 2 * 3"), Ok(36.0));
        // A minus sign in front of something, next to an operator, and in front of a bracket.
        assert_eq!(eval("-2 * 3"), Ok(-6.0));
        assert_eq!(eval("2 - -3"), Ok(5.0));
        assert_eq!(eval("-(2 + 3)"), Ok(-5.0));
        // Spaces change nothing, and a function sits inside a sum like a number.
        assert_eq!(eval("width-2*stile"), Ok(19.5));
        assert_eq!(eval("2 * max(3, 4) + 1"), Ok(9.0));
        assert_eq!(eval("floor((width - 1) / 5)"), Ok(4.0));
        assert_eq!(eval("if(width > 24, width / 2, width)"), Ok(24.0));
        assert_eq!(eval("if(height > width, 1, 0) + 1"), Ok(2.0));
        // A comparison reads the sums either side of it before comparing them.
        assert_eq!(eval("width + 1 > height - 1"), Ok(0.0));
        assert_eq!(eval("width * 2 > height"), Ok(1.0));
    }

    #[test]
    fn units_can_be_mixed_in_one_sum() {
        let names = names([("width", 24.0), ("stile", 2.25), ("height", 30.0)]);
        // Inches and millimeters in the same breath, in a recipe of either unit: everything is
        // inches by the time it is added up.
        assert_eq!(eval("300mm + 1in", false, &names), Ok(300.0 / 25.4 + 1.0));
        assert_eq!(eval("300mm + 1in", true, &names), Ok(300.0 / 25.4 + 1.0));
        assert_eq!(eval("width - 19mm", false, &names), Ok(24.0 - 19.0 / 25.4));
        // A bare number follows the recipe, so the same sum means different things in each.
        assert_eq!(eval("2 + 2", false, &names), Ok(4.0));
        assert_eq!(eval("2 + 2", true, &names), Ok(4.0 / 25.4));
    }

    #[test]
    fn what_a_recipe_reads_is_known_before_it_is_worked_out() {
        assert_eq!(names_in("width - 2*stile", false), vec!["width".to_owned(), "stile".to_owned()]);
        // A function is not a parameter, and nothing is listed twice.
        assert_eq!(names_in("floor(height / rise) + height", false), vec!["height".to_owned(), "rise".to_owned()]);
        assert!(names_in("24", false).is_empty());
    }

    #[test]
    fn a_sum_that_cant_be_worked_out_says_what_is_wrong() {
        let names = names([("width", 24.0), ("stile", 2.25), ("height", 30.0)]);
        let fails = |text: &str, wanted: &str| {
            let said = eval(text, false, &names).expect_err(text);
            assert!(said.contains(wanted), "{text}: {said}");
        };
        fails("widht - 1", "no parameter \"widht\"");
        fails("widht - 1", "it has: height, stile, width");
        fails("sqr(width)", "no function \"sqr\"");
        fails("min(width)", "min takes two numbers, not 1");
        fails("width *", "a sum stops in the middle");
        fails("(width + 1", "a bracket is left open");
        fails("width 24", "is left over");
        fails("width / 0", "divides by nothing");
        fails("", "nothing to work out");
    }

    #[test]
    fn a_length_a_person_would_type_is_not_arithmetic() {
        // These go to `units::parse_length` first and must never reach here, where `3/4"` would
        // be a division that happens to agree and `1 1/2"` would be two numbers in a row.
        assert!(!looks_like_a_sum("0.75"));
        assert!(!looks_like_a_sum("3-4"), "a range of sizes is not a sum either");
        assert!(looks_like_a_sum("width - 2*stile"));
        assert!(looks_like_a_sum("(24 + 6) / 2"));
    }
}
