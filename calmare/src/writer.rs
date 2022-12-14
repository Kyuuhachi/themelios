use std::io::{Write, Result};

use themelios::gamedata::GameData;

#[derive(Clone, Copy, Debug)]
enum Space {
	None,
	Space,
	Newline,
}

pub struct Context<'a> {
	pub game: &'a GameData<'a>,
	pub blind: bool, // These two might belong in a different type,
	pub decompile: bool, //  but then I'd have to reexport all the writing functions and that's a pain
	indent: usize,
	space: Space,
	out: Box<dyn Write + 'a>,
}

impl<'a> Context<'a> {
	pub fn new(game: &'a GameData<'a>, out: impl Write + 'a) -> Self {
		Self {
			game,
			blind: false,
			decompile: true,
			indent: 0,
			space: Space::None,
			out: Box::new(out),
		}
	}

	pub fn blind(mut self) -> Self {
		self.blind = true;
		self
	}

	pub fn flat(mut self) -> Self {
		self.decompile = false;
		self
	}
}

impl<'a> Context<'a> {
	fn put_space(&mut self) -> Result<()> {
		match self.space {
			Space::None => {}
			Space::Space => {
				write!(&mut self.out, " ")?;
			}
			Space::Newline => {
				for _ in 0..self.indent {
					write!(&mut self.out, "\t")?;
				}
			}
		}
		self.space = Space::None;
		Ok(())
	}

	pub fn space(&mut self) -> Result<&mut Self> {
		// Cannot fail, but let's Result it for consistency.
		self.space = Space::Space;
		Ok(self)
	}

	pub fn no_space(&mut self) -> Result<&mut Self> {
		self.space = Space::None;
		Ok(self)
	}

	pub fn kw(&mut self, arg: &str) -> Result<&mut Self> {
		self.put_space()?;
		write!(&mut self.out, "{arg}")?;
		self.space()?;
		Ok(self)
	}

	pub fn pre(&mut self, arg: &str) -> Result<&mut Self> {
		self.put_space()?;
		write!(&mut self.out, "{arg}")?;
		Ok(self)
	}

	pub fn suf(&mut self, arg: &str) -> Result<&mut Self> {
		write!(&mut self.out, "{arg}")?;
		self.space()?;
		Ok(self)
	}

	pub fn line(&mut self) -> Result<&mut Self> {
		writeln!(&mut self.out)?;
		self.space = Space::Newline;
		Ok(self)
	}

	pub fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<()> {
		self.put_space()?;
		self.out.write_fmt(args)
	}

	pub fn indent<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
		self.indent += 1;
		let v = f(self);
		self.indent -= 1;
		v
	}
}
