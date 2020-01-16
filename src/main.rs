extern crate num;
use num::BigInt;
use num::Zero;
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
    stack: Vec<Value>
}

impl State
{
    pub fn new() -> State
    {
        State
        {
            stack: vec![]
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

#[derive(Clone)]
enum Value
{
    String(String),
    Integer(BigInt),
    Array(Vec<Value>)
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
            },
            '@' =>
            {
                if state.stack.len() >= 2
                {
                    let i = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();
                    match (a, i)
                    {
                        (Value::Array(mut a), Value::Integer(i)) =>
                        {
                            use num::ToPrimitive;
                            let item = a.remove(i.to_usize().unwrap());
                            state.stack.push(Value::Array(a));
                            state.stack.push(item);
                        },
                        _ => return Err("Indexing can only be done with an array and an integer".to_string())
                    }
                }
                else
                {
                    return Err("Operator '@' expected two values on the stack".to_string());
                }
            },
            '#' =>
            {
                if state.stack.len() >= 2
                {
                    let n = state.stack.pop().unwrap();
                    let code = state.stack.pop().unwrap();

                    match (code, n)
                    {
                        (Value::String(code), Value::Integer(n)) =>
                        {
                            let mut i = BigInt::zero();
                            while i < n {
                                state.stack.push(Value::Integer(i.clone()));
                                eval(&mut Buffer::new(code.chars()), state)?;
                                i += 1;
                            }
                        },
                        _ => return Err("Operator '#' expected a string and an integer".to_string())
                    }
                }
                else
                {
                    return Err("Operator '#' expected two values on the stack".to_string());
                }
            },
            '$' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    state.stack.push(b);
                    state.stack.push(a);
                }
                else
                {
                    return Err("Operator '$' expected one value on the stack".to_string());
                }
            },
            '%' =>
            {
                if state.stack.len() >= 1
                {
                    let a = state.stack.pop().unwrap();
                    let b = a.clone();

                    state.stack.push(a);
                    state.stack.push(b);
                }
                else
                {
                    return Err("Operator '%' expected one value on the stack".to_string());
                }
            },
            '[' =>
            {
                if state.stack.len() >= 1
                {
                    let num = state.stack.pop().unwrap();
                    
                    match num
                    {
                        Value::Integer(num) =>
                        {
                            let mut vec = vec![];
                            let mut i = BigInt::zero();
                            while i < num
                            {
                                let val = state.stack.pop().unwrap();
                                vec.insert(0, val);
                                i += 1;
                            }
                            state.stack.push(Value::Array(vec));
                        }
                        _ => return Err("Operator '[' expected an integer".to_string())
                    }
                }
                else
                {
                    return Err("Operator '[' expected at least one value on the stack".to_string());
                }
            },
            ']' =>
            {
                if state.stack.len() >= 1
                {
                    let vec = state.stack.pop().unwrap();
                    match vec
                    {
                        Value::Array(mut vec) =>
                        {
                            while vec.len() > 0
                            {
                                let item = vec.remove(0);
                                state.stack.push(item);
                            }
                        },
                        _ => return Err("Operator ']' expected an array".to_string())
                    }
                }
                else
                {
                    return Err("Operator ']' expected one value on the stack".to_string());
                }
            },
            '?' =>
            {
                if state.stack.len() >= 3
                {
                    let c = state.stack.pop().unwrap();
                    let f = state.stack.pop().unwrap();
                    let t = state.stack.pop().unwrap();

                    match (t, f, c)
                    {
                        (Value::String(t), Value::String(f), Value::Integer(c)) =>
                        {
                            if c != BigInt::zero()
                            {
                                eval(&mut Buffer::new(t.chars()), state);
                            }
                            else
                            {
                                eval(&mut Buffer::new(f.chars()), state);
                            }
                        },
                        _ => return Err("Operator '?' expected two strings and an integer".to_string())
                    }
                }
                else
                {
                    return Err("Operator '?' expected three values on the stack".to_string());
                }
            },
            ';' =>
            {
                if state.stack.len() >= 1
                {
                    state.stack.pop().unwrap();
                }
                else
                {
                    return Err("Operator ';' expected one value on the stack".to_string());
                }
            }
            ' ' | '\n' | '\t' | '\r' => (),
            _ => return Err(format!("Unexpected character {}", c))
        }
    }
    Ok(())
}
