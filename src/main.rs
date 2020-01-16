extern crate num;
use num::BigInt;
use num::Zero;
use num::One;
use num::ToPrimitive;

fn main() {
    let src = std::fs::read_to_string("test.wln").unwrap();
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
        for item in &self.stack
        {
            print!(" {}", item);
        }
        println!("");
    }
}

#[derive(Clone, PartialEq)]
enum Value
{
    String(String),
    Integer(BigInt),
    Array(Vec<Value>)
}

impl std::fmt::Display for Value
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error>
    {
        fmt.write_str(&match self
        {
            Value::String(s) => format!("\"{}\"", s),
            Value::Integer(i) => format!("{}", i),
            Value::Array(a) =>
            {
                let mut res = "[".to_string();
                for v in a
                {
                    res.push_str(&format!("{} ", v));
                }
                res = res.trim_end().to_string();
                res.push_str("]");
                res
            }
        })
    }
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
            '{' =>
            {
                let mut nested_count = 1;
                let mut res = String::new();
                while let Some(c) = buffer.next()
                {
                    if c == '}'
                    {
                        nested_count -= 1;
                        if nested_count == 0
                        {
                            break;
                        }
                        res.push(c);
                    }
                    else if c == '{'
                    {
                        res.push(c);
                        nested_count += 1;
                    }
                    else
                    {
                        res.push(c);
                    }
                }
                state.stack.push(Value::String(res));
            }
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
                        (Value::Array(mut a), Value::Array(mut b)) =>
                        {
                            a.append(&mut b);
                            state.stack.push(Value::Array(a));
                        }
                        (Value::Array(mut a), b) =>
                        {
                            a.push(b);
                            state.stack.push(Value::Array(a));
                        }
                        (a, Value::Array(mut b)) =>
                        {
                            b.insert(0, a);
                            state.stack.push(Value::Array(b));
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
                        (Value::Array(mut a), Value::Integer(mut i)) =>
                        {
                            if i < BigInt::zero()
                            {
                                i = a.len() + i;
                            }
                            let item = a.remove(i.to_usize().unwrap());
                            state.stack.push(Value::Array(a));
                            state.stack.push(item);
                        },
                        (Value::String(s), Value::Integer(mut i)) =>
                        {
                            if i < BigInt::zero()
                            {
                                i = s.len() + i;
                            }
                            let i = i.to_usize().unwrap();
                            let item = s.chars().nth(i).unwrap();
                            let s = s.chars().enumerate().filter(|(i2, _)| *i2 != i).map(|(_, c)| c).collect();
                            state.stack.push(Value::String(s));
                            state.stack.push(Value::String(item.to_string()));
                        }
                        _ => return Err("Indexing can only be done with an array/string and an integer".to_string())
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
                    return Err("Operator '$' expected two values on the stack".to_string());
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
            '[' => state.stack.push(Value::Array(vec![])),
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
                                eval(&mut Buffer::new(t.chars()), state).unwrap();
                            }
                            else
                            {
                                eval(&mut Buffer::new(f.chars()), state).unwrap();
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
            },
            ':' =>
            {
                if state.stack.len() >= 1
                {
                    match state.stack.pop().unwrap()
                    {
                        Value::Array(a) =>
                        {
                            let len = a.len();
                            state.stack.push(Value::Array(a));
                            state.stack.push(Value::Integer(len.into()));
                        },
                        Value::String(s) =>
                        {
                            let len = s.len();
                            state.stack.push(Value::String(s));
                            state.stack.push(Value::Integer(len.into()));
                        },
                        _ => return Err("Operator ':' cannot be applied to integers".to_string())
                    }
                }
                else
                {
                    return Err("Operator ':' expected one value on the stack".to_string());
                }
            },
            '=' =>
            {
                if state.stack.len() >= 2
                {
                    let b = state.stack.pop().unwrap();
                    let a = state.stack.pop().unwrap();

                    if a == b
                    {
                        state.stack.push(Value::Integer(BigInt::one()));
                    }
                    else
                    {
                        state.stack.push(Value::Integer(BigInt::zero()));
                    }
                }
                else
                {
                    return Err("Operator '=' expected two values on the stack".to_string());
                }
            },
            'Z' => state.print(),
            ' ' | '\n' | '\t' | '\r' => (),
            _ => return Err(format!("Unexpected character {}", c))
        }
    }
    Ok(())
}
