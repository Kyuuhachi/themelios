use hamu::read::coverage::Coverage;
use hamu::read::le::*;
use hamu::write::le::*;
use crate::util::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Status {
	pub level: u16,
	pub hp: u32,
	pub atk: u16,
	pub def: u16,
	pub ats: u16,
	pub adf: u16,
	pub dex: u16,
	pub agl: u16,
	pub mov: u16,
	pub spd: u16,
}

pub fn read(data: &[u8]) -> Result<Vec<Vec<Status>>, ReadError> {
	let mut f = Coverage::new(Bytes::new(data));
	let n = f.clone().u16()? / 2;
	let m = (f.clone().at(2)?.u16()? - f.clone().u16()?)/22;
	let mut table = Vec::with_capacity(n as usize);
	for _ in 0..n {
		let mut char = Vec::new();
		let pos = f.u16()? as usize;
		let mut g = f.clone().at(pos)?;
		for _ in 0..m {
			char.push(Status {
				level: g.u16()?,
				hp: g.u32()?,
				atk: g.u16()?,
				def: g.u16()?,
				ats: g.u16()?,
				adf: g.u16()?,
				dex: g.u16()?,
				agl: g.u16()?,
				mov: g.u16()?,
				spd: g.u16()?,
			});
		}
		table.push(char);
	}
	f.assert_covered()?;
	Ok(table)
}

pub fn write(table: &[Vec<Status>]) -> Result<Vec<u8>, WriteError> {
	let mut f = OutBytes::new();
	let mut g = OutBytes::new();
	for char in table {
		f.delay_u16(g.here());
		for status in char {
			g.u16(status.level);
			g.u32(status.hp);
			g.u16(status.atk);
			g.u16(status.def);
			g.u16(status.ats);
			g.u16(status.adf);
			g.u16(status.dex);
			g.u16(status.agl);
			g.u16(status.mov);
			g.u16(status.spd);
		}
	}
	Ok(f.concat(g).finish()?)
}
