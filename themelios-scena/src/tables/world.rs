use hamu::read::coverage::Coverage;
use hamu::read::le::*;
use hamu::write::le::*;
use crate::gamedata::Lookup;
use crate::util::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World { scena: String, x: u32, y: u32 }

pub fn read(lookup: &dyn Lookup, data: &[u8]) -> Result<Vec<World>, ReadError> {
	let mut f = Coverage::new(Bytes::new(data));
	let mut table = Vec::with_capacity(f.remaining() / 4);
	while f.remaining() > 12 {
		let scena = lookup.name(f.u32()?)?.to_owned();
		let x = f.u32()?;
		let y = f.u32()?;
		table.push(World { scena, x, y });
	}
	f.check_u32(0xFFFFFFFF)?;
	f.check_u32(0)?;
	f.check_u32(0)?;
	f.assert_covered()?;
	Ok(table)
}

pub fn write(lookup: &dyn Lookup, table: &Vec<World>) -> Result<Vec<u8>, WriteError> {
	let mut out = OutBytes::new();
	for &World { ref scena, x, y } in table {
		out.u32(lookup.index(scena).unwrap());
		out.u32(x);
		out.u32(y);
	}
	out.u32(0xFFFFFFFF);
	out.u32(0);
	out.u32(0);
	Ok(out.finish()?)
}
