use crate::replace;
use anyhow::{Context, Result};
use git::{Commit, Repository};
use hashbrown::{HashMap, HashSet};
use std::io::{Stdin, Stdout, Write};

type Table = HashMap<String, Val>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Int(i64),
    Str(String),
    Table(Table),
}

impl std::str::FromStr for Val {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('#')
            .map(str::parse::<i64>)
            .map(|res| res.map(Self::Int))
            .unwrap_or_else(|| Ok(Self::Str(s.to_owned())))
            .map_err(Into::into)
    }
}

impl std::fmt::Display for Val {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(n) => write!(f, "{}", n),
            Self::Str(s) => write!(f, "{}", s),
            Self::Table(_table) => write!(f, "<table>"),
        }
    }
}

#[derive(Debug)]
pub enum Get {
    Val(Val),
    Var(String),
}

impl Get {
    fn val<'a>(&'a self, table: &'a Table) -> Result<&'a Val> {
        match self {
            Self::Val(val) => Ok(val),
            Self::Var(var) => {
                let mut cur = table;
                let mut subs = var.split('/');
                let tail = subs.next_back().unwrap();
                for sub in subs {
                    cur = match cur.get(sub) {
                        Some(Val::Table(table)) => table,
                        Some(_) => anyhow::bail!("tried to access non-table as table: {}", var),
                        None => anyhow::bail!("undefined symbol: {}", var),
                    };
                }
                cur.get(tail)
                    .with_context(|| format!("undefined symbol: {}", var))
            }
        }
    }
}

impl std::str::FromStr for Get {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('$')
            .map(str::to_owned)
            .map(Self::Var)
            .map(Ok)
            .unwrap_or_else(|| s.parse::<Val>().map(Self::Val))
    }
}

#[derive(Debug)]
pub enum Op {
    Nop,
    Set(Get, Get),
    Get(Get, Get),
    Del(Get),
    Exists(Get, Get),
    Branch(Get),
    Enter(Get),
    Exit,
    Match(Get, Get, Vec<(Get, Get)>),
    Print(Get),
    Println(Get),
    // String operations
    Inpln(Get),
    Concat(Get, Get, Get),
    Chars(Get, Get),
    // Numerical binary operations
    Eq(Get, Get, Get),
    Gt(Get, Get, Get),
    Add(Get, Get, Get),
    Sub(Get, Get, Get),
    Mul(Get, Get, Get),
    Div(Get, Get, Get),
    Mod(Get, Get, Get),
    And(Get, Get, Get),
    Or(Get, Get, Get),
    Xor(Get, Get, Get),
}

impl std::str::FromStr for Op {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        macro_rules! parse_args {
        ($op:expr, $tokens:expr $(, $arg:ident)*) => {
            $(
                let $arg = $tokens
                    .next()
                    .with_context(|| format!(concat!("{}: missing ", stringify!($arg)), $op))?
                    .parse()
                    .with_context(|| format!(concat!("{}: invalid ", stringify!($arg)), $op))?;
            )*
        };
    }
        let mut tokens = shellwords::split(s)?.into_iter();
        match tokens.next().as_deref() {
            None => Ok(Self::Nop),
            Some(op @ "set") => {
                parse_args!(op, tokens, var, src);
                Ok(Self::Set(var, src))
            }
            Some(op @ "get") => {
                parse_args!(op, tokens, var, src);
                Ok(Self::Get(var, src))
            }
            Some(op @ "del") => {
                parse_args!(op, tokens, var);
                Ok(Self::Del(var))
            }
            Some(op @ "exists") => {
                parse_args!(op, tokens, var, symbol);
                Ok(Self::Exists(var, symbol))
            }
            Some(op @ "branch") => {
                parse_args!(op, tokens, tag);
                Ok(Self::Branch(tag))
            }
            Some(op @ "enter") => {
                parse_args!(op, tokens, table);
                Ok(Self::Enter(table))
            }
            Some("exit") => Ok(Self::Exit),
            Some(op @ "match") => {
                parse_args!(op, tokens, var, src);
                let (vals, branches): (Vec<_>, Vec<_>) =
                    tokens.enumerate().partition(|&(i, _)| i & 1 == 0);
                let branches = vals
                    .into_iter()
                    .map(|(_, val)| val)
                    .zip(branches.into_iter().map(|(_, branch)| branch))
                    .map(|(val, branch)| {
                        val.parse::<Get>()
                            .and_then(|val| branch.parse::<Get>().map(|branch| (val, branch)))
                    })
                    .collect::<Result<Vec<_>>>()
                    .with_context(|| format!("{}: syntax error", op))?;
                Ok(Op::Match(var, src, branches))
            }
            Some(op @ "print") => {
                parse_args!(op, tokens, arg);
                Ok(Self::Print(arg))
            }
            Some(op @ "println") => {
                parse_args!(op, tokens, arg);
                Ok(Self::Println(arg))
            }
            Some(op @ "inpln") => {
                parse_args!(op, tokens, var);
                Ok(Self::Inpln(var))
            }
            Some(op @ "concat") => {
                parse_args!(op, tokens, var, a, b);
                Ok(Self::Concat(var, a, b))
            }
            Some(op @ "chars") => {
                parse_args!(op, tokens, var, string);
                Ok(Self::Chars(var, string))
            }
            Some(
                op @ ("eq" | "gt" | "add" | "sub" | "mul" | "div" | "mod" | "and" | "or" | "xor"),
            ) => {
                parse_args!(op, tokens, var, a, b);
                Ok(match op {
                    "eq" => Op::Eq,
                    "gt" => Op::Gt,
                    "add" => Op::Add,
                    "sub" => Op::Sub,
                    "mul" => Op::Mul,
                    "div" => Op::Div,
                    "mod" => Op::Mod,
                    "and" => Op::And,
                    "or" => Op::Or,
                    "xor" => Op::Xor,
                    _ => unreachable!(),
                }(var, a, b))
            }
            _ => Err(anyhow::anyhow!("invalid operation: {:?}", s)),
        }
    }
}

// `Instance` is isolated into a module to prevent all but
// select methods from accessing its fields directly.
pub use instance::Instance;
mod instance {
    use super::*;

    pub struct Instance {
        table: Table,
        entered: Vec<*mut Table>,
    }

    // The methods in this `impl` are the only ones allowed to access
    // `self`'s fields directly.
    impl Instance {
        pub fn new() -> Self {
            Self {
                table: Table::new(),
                entered: Vec::new(),
            }
        }

        pub fn table(&self) -> &Table {
            // SAFETY: The pointers stored in `self.entered` will always point to data
            //         contained within `self.table`. This data is not moved when
            //         `self.table` is moved, so the pointers will not be
            //         invalidated. Modifying `self.table` directly will invalidate
            //         any pointers in `self.entered`, and as such should not happen.
            self.entered
                .last()
                .map(|ptr| unsafe { &**ptr })
                .unwrap_or(&self.table)
        }

        pub fn table_mut(&mut self) -> &mut Table {
            // SAFETY: The pointers stored in `self.entered` will always point to data
            //         contained within `self.table`. This data is not moved when
            //         `self.table` is moved, so the pointers will not be
            //         invalidated. Modifying `self.table` directly will invalidate
            //         any pointers in `self.entered`, and as such should not happen.
            //         This method cannot create multiple mutable references to
            //         `self.table` because it always uses the top pointer of
            //         `self.entered`, and the returned borrow's lifetime must end
            //         before `self.entered` can be modified. It avoids creating
            //         multiple simultaneous mutable references internally by
            //         casting the `self.table` fallback to a raw pointer and
            //         passing it to `unwrap_or`, then dereferencing and borrowing
            //         the unwrapped pointer.
            let ptr = self.entered.last().copied().unwrap_or(&mut self.table);
            unsafe { &mut *ptr }
        }

        pub fn enter_table(&mut self, table: &str) -> Result<()> {
            for table in table.split('/') {
                let table = match self
                    .table_mut()
                    .entry(table.to_owned())
                    .or_insert_with(|| Val::Table(Table::new()))
                {
                    Val::Table(table) => table,
                    _ => anyhow::bail!("tried to access non-table as table: {}", table),
                } as *mut Table;
                self.entered.push(table);
            }
            Ok(())
        }

        pub fn exit_table(&mut self) -> bool {
            self.entered.pop().is_some()
        }
    }
}

impl Instance {
    pub fn run<'a>(
        &mut self,
        repo: &'a Repository,
        start: Commit<'a>,
        mut end: Commit<'a>,
    ) -> Result<()> {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut stdin = std::io::stdin();
        let mut stdout = std::io::stdout();

        replace(repo, &mut end);

        let end_id = end.id();
        let children = crate::tree::collect_children(repo, end);

        let mut cur = start;
        loop {
            replace(repo, &mut cur);
            let op = cur
                .message()
                .context("syntax error")
                .and_then(str::parse::<Op>)?;
            if let Op::Branch(tag) = op {
                let tag = tag.val(self.table())?.to_string();
                let next = children
                    .get(&cur.id())
                    .and_then(|nexts| Self::find_tag(repo, &tag, nexts))
                    .with_context(|| format!("{}: branch: failed to find target", cur.id()))?;
                cur = next.clone();
                continue;
            }
            if let Err(e) = self.exec(op, &mut stdin, &mut stdout) {
                anyhow::bail!("{}: {}", cur.id(), e);
            }

            if cur.id() == end_id {
                break Ok(());
            }
            if let Some(next) = children.get(&cur.id()).and_then(|set| {
                let mut iter = set.iter().cloned();
                iter.nth(rng.gen::<usize>() % iter.len())
            }) {
                cur = next;
            } else {
                break Err(anyhow::anyhow!(
                    "{}: failed to find child to continue",
                    cur.id()
                ));
            }
        }
    }

    fn exec(&mut self, op: Op, stdin: &mut Stdin, stdout: &mut Stdout) -> Result<()> {
        fn num_binop(
            var: Get,
            a: Get,
            b: Get,
            instance: &mut Instance,
            op: impl Fn(i64, i64) -> i64,
            opname: &str,
        ) -> Result<()> {
            let table = instance.table();
            match (a.val(table)?, b.val(table)?) {
                (&Val::Int(a), &Val::Int(b)) => {
                    let val = var.val(table)?.clone();
                    instance.set(&val.to_string(), Val::Int(op(a, b)))
                }
                (a, b) => Err(anyhow::anyhow!("{}: invalid args: {} {}", opname, a, b)),
            }
        }

        match op {
            Op::Nop => Ok(()),
            Op::Set(var, src) => {
                let var = var.val(self.table())?.to_string();
                let val = src.val(self.table())?.clone();
                self.set(&var, val)
            }
            Op::Get(var, src) => {
                let var = var.val(self.table())?.to_string();
                let val = Get::Var(src.val(self.table())?.to_string())
                    .val(self.table())?
                    .clone();
                self.set(&var, val)
            }
            Op::Del(var) => {
                let var = var.val(self.table())?.to_string();
                let mut cur = self.table_mut();
                let mut subs = var.split('/');
                let tail = subs.next_back().unwrap().to_owned();
                for sub in subs {
                    cur = match cur
                        .entry(sub.to_owned())
                        .or_insert_with(|| Val::Table(Table::new()))
                    {
                        Val::Table(table) => table,
                        _ => anyhow::bail!("tried to access non-table as table: {}", var),
                    };
                }
                cur.remove(&tail);
                Ok(())
            }
            Op::Exists(var, symbol) => {
                let var = var.val(self.table())?.to_string();
                let symbol = symbol.val(self.table())?.to_string();
                let mut cur = self.table_mut();
                let mut subs = symbol.split('/');
                let tail = subs.next_back().unwrap();
                let mut exists = true;
                for sub in subs {
                    // Check is separated from the `get_mut` to avoid a borrow error.
                    if cur.contains_key(sub) {
                        cur = match cur.get_mut(sub).unwrap() {
                            Val::Table(table) => table,
                            _ => anyhow::bail!("tried to access non-table as table: {}", var),
                        };
                    } else {
                        exists = false;
                        break;
                    }
                }
                exists &= cur.contains_key(tail);
                self.set(&var, Val::Int(exists as i64))
            }
            Op::Enter(table) => self.enter_table(&table.val(self.table())?.to_string()),
            Op::Exit => {
                self.exit_table();
                Ok(())
            }
            Op::Match(var, src, branches) => {
                let val = src.val(self.table())?;
                for branch in branches {
                    if *branch.0.val(self.table())? == *val {
                        let new_val = branch.1.val(self.table())?.clone();
                        let var = var.val(self.table())?.to_string();
                        self.set(&var, new_val)?;
                        break;
                    }
                }
                Ok(())
            }
            Op::Print(arg) => arg
                .val(self.table())
                .and_then(|val| write!(stdout, "{}", val).map_err(From::from))
                .and_then(|_| stdout.flush().map_err(From::from)),
            Op::Println(arg) => arg
                .val(self.table())
                .and_then(|val| writeln!(stdout, "{}", val).map_err(From::from)),
            Op::Inpln(var) => {
                let var = var.val(self.table())?.to_string();
                let mut s = String::new();
                stdin.read_line(&mut s)?;
                s.pop();
                if s.ends_with('\r') {
                    s.pop();
                }
                self.set(&var, Val::Str(s))?;
                Ok(())
            }
            Op::Concat(var, a, b) => {
                let var = var.val(self.table())?.to_string();
                let concat = format!("{}{}", a.val(self.table())?, b.val(self.table())?);
                self.set(&var, Val::Str(concat))?;
                Ok(())
            }
            Op::Chars(var, string) => {
                let var = var.val(self.table())?.to_string();
                let mut table = Table::new();
                string
                    .val(self.table())?
                    .to_string()
                    .chars()
                    .enumerate()
                    .map(|(i, c)| (i.to_string(), Val::Str(c.to_string())))
                    .for_each(|(i, c)| drop(table.insert(i, c)));
                table.insert("len".to_owned(), Val::Int(table.len() as i64));
                self.set(&var, Val::Table(table))
            }
            Op::Eq(var, a, b) => {
                let a = a.val(self.table())?;
                let b = b.val(self.table())?;
                let eq = (a == b) as i64;
                let var = var.val(self.table())?.to_string();
                self.set(&var, Val::Int(eq))?;
                Ok(())
            }
            Op::Gt(var, a, b) => num_binop(var, a, b, self, |a, b| (a > b) as i64, "gt"),
            Op::Add(var, a, b) => num_binop(var, a, b, self, |a, b| a + b, "add"),
            Op::Sub(var, a, b) => num_binop(var, a, b, self, |a, b| a - b, "sub"),
            Op::Mul(var, a, b) => num_binop(var, a, b, self, |a, b| a * b, "mul"),
            Op::Div(var, a, b) => num_binop(var, a, b, self, |a, b| a / b, "div"),
            Op::Mod(var, a, b) => num_binop(var, a, b, self, |a, b| a % b, "mod"),
            Op::And(var, a, b) => num_binop(var, a, b, self, |a, b| a & b, "and"),
            Op::Or(var, a, b) => num_binop(var, a, b, self, |a, b| a | b, "or"),
            Op::Xor(var, a, b) => num_binop(var, a, b, self, |a, b| a ^ b, "xor"),
            Op::Branch(_) => unreachable!(),
        }
    }

    fn set(&mut self, var: &str, val: Val) -> Result<()> {
        let mut cur = self.table_mut();
        let mut subs = var.split('/');
        let tail = subs.next_back().unwrap().to_owned();
        for sub in subs {
            cur = match cur
                .entry(sub.to_owned())
                .or_insert_with(|| Val::Table(Table::new()))
            {
                Val::Table(table) => table,
                _ => anyhow::bail!("tried to access non-table as table: {}", var),
            };
        }
        cur.insert(tail, val);
        Ok(())
    }

    fn find_tag<'a, 'b>(
        repo: &'a Repository,
        tag: &str,
        commits: &'b [Commit<'a>],
    ) -> Option<&'b Commit<'a>> {
        let mut found = Vec::new();

        let mut tag = repo
            .find_reference(&format!("refs/tags/{}", tag))
            .and_then(|tag| tag.peel_to_commit())
            .ok()?;
        replace(repo, &mut tag);

        let mut checked = HashSet::new();
        let mut stack = vec![(tag, 0)];
        let mut dist = 1;
        while let Some((cur, i)) = stack.last_mut() {
            if let Some(commit) = commits.iter().find(|commit| commit.id() == cur.id()) {
                found.push((commit, dist));
            }
            if let Ok(mut parent) = cur.parent(*i) {
                replace(repo, &mut parent);
                if checked.insert(parent.id()) {
                    stack.push((parent, 0));
                    dist += 1;
                } else {
                    *i += 1;
                    continue;
                }
            } else {
                stack.pop();
                dist -= 1;
            }
        }

        found
            .into_iter()
            .min_by_key(|(_, dist)| *dist)
            .map(|(commit, _)| commit)
    }
}
