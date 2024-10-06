use std::error::Error;

use serde_json::Value;

pub struct MyBEncodedBuf {
    pub pos: usize,
    pub string_buf: String,
}
type MyBEncodedResult<T> = Result<T, Box<dyn Error>>;
impl MyBEncodedBuf {
    pub fn get_current_slice(&self) -> &str {
        let a = &self.string_buf[self.pos..];
        a
    }
    pub fn len_bound(&self) -> usize {
        self.string_buf.len()
    }
    pub fn read(&mut self) -> MyBEncodedResult<char> {
        let ret = self.peek()?;
        self.seek(self.pos + ret.len_utf8())?;

        Ok(ret)
    }
    pub fn peek(&self) -> MyBEncodedResult<char> {
        if self.pos > self.len_bound() {
            return Err("read".into());
        }
        let mut chars = self.string_buf[self.pos..].chars();
        let ret = chars.next().ok_or("peek")?;

        Ok(ret)
    }
    pub fn seek(&mut self, p: usize) -> MyBEncodedResult<()> {
        if p > self.len_bound() {
            return Err("seek".into());
        }
        self.pos = p;
        Ok(())
    }
    pub fn step(&mut self, steps: usize) -> MyBEncodedResult<()> {
        let target = self.pos + steps;
        if target > self.len_bound() {
            return Err(format!("pos {} step {}", self.pos, steps).into());
        }
        self.pos = target;
        Ok(())
    }
    pub fn parse_str(&mut self) -> MyBEncodedResult<Value> {
        let a = self.get_current_slice().split_once(':').ok_or("split")?;

        let n = a.0.parse::<usize>()?;
        let s = a.1[0..n].to_string();
        self.step(1 + a.0.len() + n)?;

        Ok(s.into())
    }
    pub fn parse_integer(&mut self) -> MyBEncodedResult<Value> {
        self.get_current_slice();

        self.step(1)?;

        let a = self.get_current_slice().split_once('e').ok_or("split")?;
        let n = a.0.parse::<i64>()?;
        self.step(a.0.len() + 1)?;

        Ok(n.into())
    }
    pub fn parse_list(&mut self) -> MyBEncodedResult<Value> {
        self.step(1)?;
        let mut v = vec![];
        while let Ok(c) = self.peek() {
            match c {
                'e' => {
                    self.step(1)?;
                    break;
                }
                _ if c == 'i' => {
                    let value = self.parse_integer()?;
                    v.push(value);
                    continue;
                }
                _ if c == 'l' => {
                    let value = self.parse_list()?;
                    v.push(value);
                    continue;
                }
                _ if c.is_digit(10) => {
                    let value = self.parse_str()?;
                    v.push(value);
                    continue;
                }
                _ => {
                    panic!("Unhandled list encoded value: {}", c)
                }
            };
        }

        Ok(v.into())
    }
    pub fn parse(&mut self) -> MyBEncodedResult<Value> {
        let c = self.peek()?;
        let a = match c {
            'i' => {
                let n = self.parse_integer()?;
                n
            }
            'l' => {
                let n = self.parse_list()?;
                n
            }

            _ if c.is_digit(10) => {
                let n = self.parse_str()?;
                n
            }

            _ => {
                panic!("Unhandled encoded value: {}", self.string_buf)
            }
        };
        Ok(a)
    }
}
impl From<&str> for MyBEncodedBuf {
    fn from(value: &str) -> Self {
        Self {
            pos: 0,
            string_buf: value.to_owned(),
        }
    }
}
impl From<&String> for MyBEncodedBuf {
    fn from(value: &String) -> Self {
        Self {
            pos: 0,
            string_buf: value.to_owned(),
        }
    }
}
