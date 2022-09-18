use hamu::read::coverage::Coverage;
use hamu::read::le::*;
use hamu::write::le::*;
use crate::archive::Archives;
use crate::util::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Town(String, TownType);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
#[repr(u8)]
pub enum TownType {
	None       = 0,
	Weapons    = 1, // Arms & Guards    武器・防具
	Goods      = 2, // General Goods    薬・雑貨・食材
	Lodgings   = 3, // Lodgings         休憩・宿泊
	Guild      = 4, // Bracer Guild     遊撃士協会
	Orbment    = 5, // Orbment Factory  オーブメント
	Restaurant = 6, // Restaurant/Inn   食事・休憩
	Church     = 7, // Septian Church   七耀教会
	Cafe       = 8, // Cafe             飲食・喫茶
}

pub fn read(_arcs: &Archives, t_town: &[u8]) -> Result<Vec<Town>, ReadError> {
	let mut f = Coverage::new(Bytes::new(t_town));
	let n = f.u16()?;
	let mut names = Vec::with_capacity(n as usize);
	for _ in 0..n {
		let pos = f.u16()? as usize;
		let mut g = f.clone().at(pos)?;
		let name = g.string()?;
		let type_ = if name.is_empty() {
			0
		} else {
			g.u8()?
		};
		let type_ = cast(type_)?;
		names.push(Town(name, type_));
	}
	f.assert_covered()?;
	Ok(names)
}

pub fn write(_arcs: &Archives, towns: &[Town]) -> Result<Vec<u8>, WriteError> {
	let mut head = Out::new();
	let mut body = Out::new();
	let mut count = Count::new();
	head.u16(cast(towns.len())?);
	for &Town(ref name, kind) in towns {
		let l = count.next();
		head.delay_u16(l);
		body.label(l);
		body.string(name)?;
		if name.is_empty() {
			if kind != TownType::None {
				return Err("empty town must be type None".to_owned().into());
			}
		} else {
			body.u8(kind.into());
		}
	}
	head.concat(body);
	Ok(head.finish()?)
}

#[cfg(test)]
mod test {
	use crate::archive::Archives;
	use crate::util::test::*;

	#[test_case::test_case(&FC; "fc")]
	fn roundtrip(arc: &Archives) -> Result<(), Error> {
		check_roundtrip(arc, "t_town._dt", super::read, super::write)?;
		Ok(())
	}
}
