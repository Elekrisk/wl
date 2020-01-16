extern crate num;
use num::BigInt;
use std::collections::HashMap;

fn main() {
    let src = r#"
    
    "#;
    let mut buffer = Buffer::new(src.chars());
    let mut state = State::new();
    match eval(&mut buffer, &mut state)
    {
        Err(e) => println!("Error: {}", e),
        _ => ()
    }
    state.print();
}

struct State
{
    stack: Vec<Value>,
    funcs: HashMap<String, Function>
}

impl State
{
    pub fn new() -> State
    {
        State
        {
            stack: vec![],
            funcs: HashMap::new()
        }
    }

    pub fn print(&self)
    {
        print!("stack:");
        for v in &self.stack
        {
            print!(" {}", match v
            {
                Value::String(s) => format!("\"{}\"", s),
                Value::Integer(i) => format!("{}", i),
                _ => "???".to_string()
            });
        }
        println!("");
    }
}

enum Value
{
    String(String),
    Integer(BigInt),
    Function(Function)
}

enum Function
{
    User(String),
    Builtin(Box<dyn Fn(&mut State) -> ()>)
}

struct Buffer<T: Iterator<Item = char>>
{
    source: T,
    buffer: Vec<char>
}

impl<T: Iterator<Item = char>> Buffer<T>
{
    fn new(source: T) -> Buffer<T>
    {
        Buffer
        {
            source,
            buffer: vec![]
        }
    }
    fn next(&mut self) -> Option<char>
    {
        if self.buffer.len() > 0
        {
            self.buffer.pop()
        }
        else
        {
            self.source.next()
        }
    }
    fn back(&mut self, c: char)
    {
        self.buffer.push(c);
    }
}

fn eval<T: Iterator<Item = char>>(buffer: &mut Buffer<T>, state: &mut State) -> Result<(), String>
{
    while let Some(c) = buffer.next()
    {
        match c
        {
            '"' =>
            {
                let mut res = String::new();
                while let Some(c) = buffer.next()
                {
                    if c == '\\'
                    {
                        if let Some(c) = buffer.next()
                        {
                            res.push(match c
                            {
                                'n' => '\n',
                                'r' => '\r',
                                't' => '\t',
                                _ => c
                            });
                        }
                    }
                    else if c == '"'
                    {
                        break;
                    }
                    else
                    {
                        res.push(c);
                    }
                }
                state.stack.push(Value::String(res));
            },
            '0' ..= '9' =>
            {
                let mut i: BigInt = c.to_string().parse().unwrap();
                while let Some(c) = buffer.next()
                {
                    if let '0' ..= '9' = c
                    {
                        i *= 10;
                        i += c.to_string().parse::<BigInt>().unwrap();
                    }
                    else
                    {
                        buffer.back(c);
                        break;
                    }
                }
                state.stack.push(Value::Integer(i));
            }
            '+' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    match (a, b)
                    {
                        (Value::String(a), Value::String(b)) =>
                        {
                            let a = a + &b;
                            state.stack.push(Value::String(a));
                        }
                        (Value::Integer(a), Value::String(b)) =>
                        {
                            state.stack.push(Value::String(a.to_string() + &b));
                        }
                        (Value::String(a), Value::Integer(b)) =>
                        {
                            state.stack.push(Value::String(a + &b.to_string()));
                        }
                        (Value::Integer(a), Value::Integer(b)) =>
                        {
                            state.stack.push(Value::Integer(a + b));
                        }
                        _ => return Err("Addition error".to_string())
                    }
                }
                else
                {
                    return Err("Operator '+' expected two values on the stack".to_string())
                } 
            }
            '-' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    match (a, b)
                    {
                        (Value::Integer(a), Value::Integer(b)) =>
                        {
                            state.stack.push(Value::Integer(a - b));
                        },
                        _ => return Err("Subtraction error".to_string())
                    }
                }
                else
                {
                    return Err("Operator '-' expected two values on the stack".to_string());
                }
            },
            '*' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    match (a, b)
                    {
                        (Value::String(a), Value::Integer(b)) =>
                        {
                            use crate::num::Zero;
                            let mut r = String::new();
                            let mut i = BigInt::zero();
                            while i < b
                            {
                                r += &a;
                                i += 1;
                            }
                            state.stack.push(Value::String(a))
                        },
                        (Value::Integer(a), Value::Integer(b)) =>
                        {
                            state.stack.push(Value::Integer(a * b));
                        },
                        _ => return Err("Multiplication error".to_string())
                    }
                }
                else
                {
                    return Err("Operator '*' expected two values on the stack".to_string());
                }
            }
            '/' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    match (a, b)
                    {
                        (Value::Integer(a), Value::Integer(b)) =>
                        {
                            state.stack.push(Value::Integer(a / b));
                        },
                        _ => return Err("Division error".to_string())
                    }
                }
                else
                {
                    return Err("Operator '/' expected two values on the stack".to_string());
                }
            },
            '!' =>
            {
                if state.stack.len() >= 1
                {
                    match state.stack.pop().unwrap()
                    {
                        Value::String(s) => eval(&mut Buffer::new(s.chars()), state)?,
                        _ => return Err("Only strings can be evaluated".to_string())
                    }
                }
                else
                {
                    return Err("Operator '!' expected one value on the stack".to_string())
                }
            }
            ' ' | '\n' | '\t' | '\r' => (),
            _ => return Err(format!("Unexpected character {}", c))
        }
    }
    Ok(())
}
