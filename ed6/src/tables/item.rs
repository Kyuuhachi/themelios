use enumflags2::*;
use hamu::read::coverage::Coverage;
use hamu::read::le::*;
use hamu::write::le::*;
use crate::archive::Archives;
use crate::util::*;

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemFlag {
	Battle      = 0x01,
	Use         = 0x02, // on battle items, whether they can target allies. Otherwise, equal to Book
	Sell        = 0x04,
	Discard     = 0x08,
	TargetEnemy = 0x10,
	TargetDead  = 0x20,
	Book        = 0x40, // never used together with Battle
	_80         = 0x80,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
	// Actually name and desc come first, but having id first is nicer
	pub id: u16,
	pub name: String,
	pub desc: String, // TODO should be Text
	pub flags: BitFlags<ItemFlag>,
	pub usable_by: u8,
	pub ty: [u8; 4],
	pub _unk1: u8, // 0 on quartz and key items, 2 on cannons, 1 otherwise
	pub stats: [i16; 10],
	pub limit: u16,
	pub price: u32,
}

pub fn read(_arcs: &Archives, t_item: &[u8], t_item2: &[u8]) -> Result<Vec<Item>, ReadError> {
	let mut f1 = Coverage::new(Bytes::new(t_item));
	let mut f2 = Coverage::new(Bytes::new(t_item2));
	let n = f1.clone().u16()? / 2;
	let n2 = f2.clone().u16()? / 2;
	if n != n2 {
		return Err("mismatched item/item2".to_owned().into())
	}

	let mut list = Vec::with_capacity(n as usize);

	for _ in 0..n {
		let mut g1 = f1.clone().at(f1.u16()? as usize)?;
		let mut g2 = f2.clone().at(f2.u16()? as usize)?;

		let id = g1.u16()?;
		let flags = cast(g1.u8()?)?;
		let usable_by = g1.u8()?; // TODO Flags in FC, enum in others
		let ty = [ g1.u8()?, g1.u8()?, g1.u8()?, g1.u8()? ];
		g1.check_u8(0)?;
		let _unk1 = g1.u8()?;
		let stats = [
			g1.i16()?, g1.i16()?, g1.i16()?, g1.i16()?, g1.i16()?,
			g1.i16()?, g1.i16()?, g1.i16()?, g1.i16()?, g1.i16()?,
		];
		let limit = g1.u16()?;
		let price = g1.u32()?;

		let name = g2.clone().at(g2.u16()? as usize)?.string()?;
		let desc = g2.clone().at(g2.u16()? as usize)?.string()?;

		list.push(Item { id, name, desc, flags, usable_by, ty, _unk1, stats, limit, price });
	}

	f1.assert_covered()?;
	f2.assert_covered()?;
	Ok(list)
}

pub fn write(_arcs: &Archives, list: &[Item]) -> Result<(Vec<u8>, Vec<u8>), WriteError> {
	let mut f1 = Out::new();
	let mut g1 = Out::new();
	let mut f2 = Out::new();
	let mut g2 = Out::new();
	let mut count = Count::new();

	for &Item { id, ref name, ref desc, flags, usable_by, ty, _unk1, stats, limit, price } in list {
		let l = count.next();
		f1.delay_u16(l);
		g1.label(l);
		f2.delay_u16(l);
		g2.label(l);

		g1.u16(id);
		g1.u8(flags.bits());
		g1.u8(usable_by);
		g1.array(ty);
		g1.u8(0);
		g1.u8(_unk1);
		for s in stats { g1.i16(s) }
		g1.u16(limit);
		g1.u32(price);

		let l1 = count.next();
		let l2 = count.next();
		g2.delay_u16(l1);
		g2.delay_u16(l2);
		g2.label(l1);
		g2.string(name)?;
		g2.label(l2);
		g2.string(desc)?;
	}
	f1.concat(g1);
	f2.concat(g2);
	Ok((f1.finish()?, f2.finish()?))
}

#[cfg(test)]
mod test {
	use crate::archive::Archives;
	use crate::util::test::*;

	#[test_case::test_case(&FC; "fc")]
	fn roundtrip(arc: &Archives) -> Result<(), Error> {
		let t_item = arc.get_decomp("t_item._dt")?;
		let t_item2 = arc.get_decomp("t_item2._dt")?;
		let items = super::read(arc, &t_item, &t_item2)?;
		let (t_item_, t_item2_) = super::write(arc, &items)?;
		let items2 = super::read(arc, &t_item_, &t_item2_)?;
		check_equal(&items, &items2)?;
		Ok(())
	}
}