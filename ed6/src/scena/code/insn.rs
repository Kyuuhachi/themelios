use super::*;

ed6_macros::bytecode! {
	|arc: &GameData|
	match {
		0x01 => Return(),
		0x05 => Call(func_ref() -> FuncRef),
		0x06 => NewScene(file_ref(arc) -> String alias ScenaFileRef, u8, u8, u8, u8),
		0x08 => Sleep(u32 alias Time),
		0x09 => FlagsSet(u32 as Flags),
		0x0A => FlagsUnset(u32 as Flags),
		0x0B => FadeOn(u32 alias Time, u32 as Color, u8),
		0x0C => FadeOff(u32 alias Time, u32 as Color),
		0x0D => _0D(),
		0x0E => Blur(u32 alias Time),
		0x0F => Battle(u16 as BattleId, u16, u16, u16, u8, u16, i8),
		0x10 => ExitSetEnabled(u8 alias ExitId, u8),
		0x11 => Fog(u8, u8, u8, u32, u32, u32), // First three are color; TODO parse it as one. Last is always 0.
		0x12 => _12(i32, i32, u32),
		0x13 => PlaceSetName(u16 as TownId),
		0x16 => Map(match {
			0x00 => Hide(),
			0x01 => Show(),
			0x02 => Set(i32, Pos2, file_ref(arc) -> String alias MapFileRef),
		}),
		0x17 => Save(),
		0x19 => EventBegin(u8),
		0x1A => EventEnd(u8),
		0x1B => _1B(u16, u16),
		0x1C => _1C(u16, u16),
		0x1D => BgmSet(u8 as BgmId),
		0x1E => _1E(),
		0x1F => BgmSetVolume(u8, u32 alias Time),
		0x20 => _20(u32 alias Time),
		0x21 => _21(), // Always paired with _20
		0x22 => SoundPlay(u16 as SoundId, u8, u8),
		0x23 => SoundStop(u16 as SoundId),
		0x24 => SoundLoop(u16 as SoundId, u8),
		0x25 => _Sound25(u16 as SoundId, Pos3, u32, u32, u8, u32),
		0x26 => _Sound26(u16 as SoundId),
		0x28 => Quest(u16 as QuestId, match {
			0x01 => TaskSet(u16 alias QuestTask),
			0x02 => TaskUnset(u16 alias QuestTask),
			0x03 => FlagsSet(u8 as QuestFlags),
			0x04 => FlagsUnset(u8 as QuestFlags),
		}),
		0x29 => Quest(u16 as QuestId, match {
			0x00 => FlagsGet(u8 as QuestFlags),
			0x01 => TaskGet(u16 alias QuestTask),
		}),
		0x2A => QuestList(quest_list() -> Vec<QuestId> alias QuestList),
		0x2B => QuestBonusBp(u16 as QuestId, u16),
		0x2C => QuestBonusMira(u16 as QuestId, u16),
		0x2D => PartyAdd(u8 as Member, u8),
		0x2E => PartyRemove(u8 as Member, u8),
		0x30 => _Party30(u8),
		0x31 => PartySetAttr(u8 as Member, u8 as MemberAttr, u16),
		0x34 => PartyAddArt(u8 as Member, u16 as MagicId),
		0x35 => PartyAddCraft(u8 as Member, u16 as MagicId),
		0x36 => PartyAddSCraft(u8 as Member, u16 as MagicId),
		0x37 => PartySet(u8 as Member, u8, u8),
		0x38 => SepithAdd(u8 as Element alias SepithElement, u16),
		0x39 => SepithRemove(u8 as Element alias SepithElement, u16),
		0x3A => MiraAdd(u16),
		0x3B => MiraSub(u16),
		0x3C => BpAdd(u16),
		// I have a guess what 3D is, but it doesn't exist in any scripts
		0x3E => ItemAdd(u16 as ItemId, u16),
		0x3F => ItemRemove(u16 as ItemId, u16),
		0x40 => ItemHas(u16 as ItemId), // or is it ItemGetCount?
		0x41 => PartyEquip(u8 as Member, u16 as ItemId, party_equip_slot(&_1) -> i8),
		0x42 => PartyPosition(u8 as Member),
		0x43 => CharForkFunc(u16 as CharId, u8 alias ForkId, func_ref() -> FuncRef),
		0x44 => CharForkQuit(u16 as CharId, u8 alias ForkId),
		0x45 => CharFork(u16 as CharId, u8 alias ForkId, u8, fork(arc) -> Vec<Insn> alias Fork),
		0x46 => CharForkLoop(u16 as CharId, u8 alias ForkId, u8, fork_loop(arc) -> Vec<Insn> alias Fork),
		0x47 => CharForkAwait(u16 as CharId, u8 alias ForkId, u8),
		0x48 => Yield(), // Used in tight loops, probably wait until next frame
		0x49 => Event(func_ref() -> FuncRef), // Not sure how this differs from Call
		0x4A => _Char4A(u16 as CharId, u8),
		0x4B => _Char4B(u16 as CharId, u8),
		0x4D => Var(u16 as Var, expr(arc) -> Expr),
		0x4F => Attr(u8 as Attr, expr(arc) -> Expr),
		0x51 => CharAttr(char_attr() -> CharAttr, expr(arc) -> Expr),
		0x52 => TextStart(u16 as CharId),
		0x53 => TextEnd(u16 as CharId),
		0x54 => TextMessage(text() -> Text),
		0x56 => TextReset(u8),
		0x58 => TextWait(),
		0x59 => _59(),
		0x5A => TextSetPos(i16, i16, i16, i16),
		0x5B => TextTalk(u16 as CharId, text() -> Text),
		0x5C => TextTalkNamed(u16 as CharId, String alias TextTitle, text() -> Text),
		0x5D => Menu(u16 alias MenuId, i16, i16, u8, menu() -> Vec<String> alias Menu),
		0x5E => MenuWait(u16 as Var),
		0x5F => MenuClose(u16 alias MenuId),
		0x60 => TextSetName(String alias TextTitle),
		0x61 => _61(u16 as CharId),
		0x62 => Emote(u16 as CharId, i32, u32 alias Time, emote() -> Emote, u8),
		0x63 => EmoteStop(u16 as CharId),
		0x64 => _64(u8, u16),
		0x65 => _65(u8, u16),
		0x66 => _Cam66(u16),
		0x67 => CamOffset(i32, i32, i32, u32 alias Time),
		0x69 => CamLookAt(u16 as CharId, u32 alias Time),
		0x6A => _Char6A(u16 as CharId),
		0x6B => CamDistance(i32, u32 alias Time),
		0x6C => CamAngle(i32 alias Angle32, u32 alias Time),
		0x6D => CamPos(Pos3, u32 alias Time),
		0x6E => _Cam6E(u8, u8, u8, u8, u32 alias Time),
		0x6F => _Obj6F(u16 alias ObjectId, u32),
		0x70 => _Obj70(u16 alias ObjectId, u32),
		0x71 => _Obj71(u16 alias ObjectId, u16),
		0x72 => _Obj72(u16 alias ObjectId, u16),
		0x73 => _Obj73(u16 alias ObjectId),
		0x74 => _74(u16, u32, u16),
		0x75 => _75(u8, u32, u8),
		0x76 => _76(u16, u32, u16, Pos3, u8, u8),
		0x77 => _77(u32 as Color, u32 alias Time),
		0x78 => _78(u8, u16),
		0x79 => _79(u8, u16),
		0x7A => _7A(u8, u16),
		0x7B => _7B(),
		0x7C => Shake(u32, u32, u32, u32 alias Time),
		0x7E => _7E(i16, i16, u16, u8, u32),
		0x7F => EffLoad(u8, String alias EffFileRef),
		0x80 => EffPlay(u8, u8, i16, Pos3, u16, u16, u16, u32, u32, u32, u16, u32, u32, u32, u32),
		0x81 => EffPlay2(u16, u8, String alias EffFileRef, Pos3, u16, u16, u16, u32, u32, u32, u32),
		0x82 => _82(u16),
		0x83 => Achievement(u8, u8),
		0x84 => _84(u8),
		0x85 => _85(u16),
		0x86 => CharSetChcp   (u16 as CharId, u16 alias ChcpId),
		0x87 => CharSetFrame  (u16 as CharId, u16),
		0x88 => CharSetPos    (u16 as CharId, Pos3, u16 alias Angle),
		0x89 => _Char89       (u16 as CharId, Pos3, u16),
		0x8A => CharLookAt    (u16 as CharId, u16 as CharId, u16 alias Time16),
		0x8B => _Char8B       (u16 as CharId, Pos2, u16),
		0x8C => CharSetAngle  (u16 as CharId, u16 alias Angle, u16 alias Time16),
		0x8D => CharIdle      (u16 as CharId, Pos2, Pos2, u32 alias Speed),
		0x8E => CharWalkTo    (u16 as CharId, Pos3, u32 alias Speed, u8),
		0x8F => CharWalkTo2   (u16 as CharId, Pos3, u32 alias Speed, u8), // how are these two different?
		0x90 => DontGoThere   (u16, Pos3 alias RelPos3, u32, u8),
		0x91 => _Char91       (u16 as CharId, Pos3 alias RelPos3, i32, u8),
		0x92 => _Char92       (u16 as CharId, u16 as CharId, u32, u32 alias Time, u8),
		0x93 => _Char93       (u16 as CharId, u16, u32, u32, u8),
		0x94 => _94       (u8, u16 as CharId, u16, u32, u32, u8), // used with chickens
		0x95 => CharJump      (u16 as CharId, Pos3 alias RelPos3, u32 alias Time, u32),
		0x96 => _Char96       (u16 as CharId, Pos3, i32, i32),
		0x97 => _Char97       (u16 as CharId, Pos2, i32 alias Angle32, u32 alias Time, u16), // used with pigeons
		0x99 => CharAnimation (u16 as CharId, u8, u8, u32 alias Time),
		0x9A => CharFlagsSet  (u16 as CharId, u16 as CharFlags),
		0x9B => CharFlagsUnset(u16 as CharId, u16 as CharFlags),
		0x9C => _Char9C       (u16 as CharId, u16),
		0x9D => _Char9D       (u16 as CharId, u16),
		0x9E => CharShake     (u16 as CharId, u32, u32, u32, u32 alias Time),
		0x9F => CharColor     (u16 as CharId, u32 as Color, u32 alias Time),
		0xA1 => _CharA1(u16 as CharId, u16),
		0xA2 => FlagSet(u16 as Flag),
		0xA3 => FlagUnset(u16 as Flag),
		0xA5 => FlagAwaitUnset(u16 as Flag),
		0xA6 => FlagAwaitSet(u16 as Flag),
		0xA9 => ShopOpen(u8 as ShopId),
		0xAC => RecipeLearn(u16), // TODO check type
		0xAD => ImageShow(file_ref(arc) -> String alias VisFileRef, u16, u16, u32 alias Time),
		0xAE => ImageHide(u32 alias Time),
		0xAF => QuestSubmit(u8 as ShopId, u16 as QuestId),
		0xB0 => _B0(u16, u8), // Used along with 6F, 70, and 73 during T0700#11
		0xB1 => OpLoad(String alias OpFileRef),
		0xB2 => _B2(u8, u8, u16),
		0xB3 => Video(match {
			0x00 => _00(String alias AviFileRef),
			0x01 => _01(u8),
		}),
		0xB4 => ReturnToTitle(u8),
		// fc only region
		0xB5 => PartySlot(u8 as Member, u8, u8),
		0xB6 => _B6(u8),
		0xB7 => _B7(u8 as Member, u8, u8),
		0xB8 => _B8(u8 as Member),
		0xB9 => ReadBook(u16 as ItemId, u16),
		0xBA => PartyHasSpell(u8 as Member, u16 as MagicId),
		0xBB => PartyHasSlot(u8 as Member, u8),
		0xDE => SaveClearData(),
	}
}

mod quest_list {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>) -> Result<Vec<QuestId>, ReadError> {
		let mut quests = Vec::new();
		loop {
			match f.u16()? {
				0xFFFF => break,
				q => quests.push(QuestId(q))
			}
		}
		Ok(quests)
	}

	pub(super) fn write(f: &mut impl Out, v: &Vec<QuestId>) -> Result<(), WriteError> {
		for &i in v {
			f.u16(i.0);
		}
		f.u16(0xFFFF);
		Ok(())
	}
}

mod fork {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>, arc: &GameData) -> Result<Vec<Insn>, ReadError> {
		let len = f.u8()? as usize;
		let pos = f.pos();
		let mut insns = Vec::new();
		while f.pos() < pos+len {
			insns.push(Insn::read(f, arc)?);
		}
		ensure!(f.pos() == pos+len, "overshot while reading fork");
		f.check_u8(0)?;
		Ok(insns)
	}

	pub(super) fn write(f: &mut impl OutDelay, arc: &GameData, v: &[Insn]) -> Result<(), WriteError> {
		let (l1, l1_) = HLabel::new();
		let (l2, l2_) = HLabel::new();
		f.delay(move |l| Ok(u8::to_le_bytes(hamu::write::cast_usize(l(l2)? - l(l1)?)?)));
		f.label(l1_);
		for i in v {
			Insn::write(f, arc, i)?;
		}
		f.label(l2_);
		f.u8(0);
		Ok(())
	}
}

mod fork_loop {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>, arc: &GameData) -> Result<Vec<Insn>, ReadError> {
		let len = f.u8()? as usize;
		let pos = f.pos();
		let mut insns = Vec::new();
		while f.pos() < pos+len {
			insns.push(Insn::read(f, arc)?);
		}
		ensure!(f.pos() == pos+len, "overshot while reading fork loop");
		ensure!(read_raw_insn(f, arc)? == RawIInsn::Insn(Insn::Yield()), "invalid loop");
		ensure!(read_raw_insn(f, arc)? == RawIInsn::Goto(pos), "invalid loop");
		Ok(insns)
	}

	pub(super) fn write(f: &mut impl OutDelay, arc: &GameData, v: &[Insn]) -> Result<(), WriteError> {
		let (l1, l1_) = HLabel::new();
		let (l2, l2_) = HLabel::new();
		let l1c = l1.clone();
		f.delay(|l| Ok(u8::to_le_bytes(hamu::write::cast_usize(l(l2)? - l(l1)?)?)));
		f.label(l1_);
		for i in v {
			Insn::write(f, arc, i)?;
		}
		f.label(l2_);
		write_raw_insn(f, arc, RawOInsn::Insn(&Insn::Yield()))?;
		write_raw_insn(f, arc, RawOInsn::Goto(l1c))?;
		Ok(())
	}
}

mod party_equip_slot {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>, arg1: &ItemId) -> Result<i8, ReadError> {
		if (600..800).contains(&arg1.0) {
			Ok(f.i8()?)
		} else {
			Ok(-1)
		}
	}

	pub(super) fn write(f: &mut impl Out, arg1: &ItemId, v: &i8) -> Result<(), WriteError> {
		if (600..800).contains(&arg1.0) {
			f.i8(*v);
		} else {
			ensure!(*v == -1, "invalid PartyEquipSlot");
		}
		Ok(())
	}
}

mod menu {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>) -> Result<Vec<String>, ReadError> {
		Ok(f.string()?.split_terminator('\x01').map(|a| a.to_owned()).collect())
	}

	pub(super) fn write(f: &mut impl Out, v: &[String]) -> Result<(), WriteError> {
		let mut s = String::new();
		for line in v {
			s.push_str(line.as_str());
			s.push('\x01');
		}
		f.string(&s)?;
		Ok(())
	}
}

mod emote {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>) -> Result<Emote, ReadError> {
		let a = f.u8()?;
		let b = f.u8()?;
		let c = f.u32()?;
		Ok(Emote(a, b, c))
	}

	pub(super) fn write(f: &mut impl Out, &Emote(a, b, c): &Emote) -> Result<(), WriteError> {
		f.u8(a);
		f.u8(b);
		f.u32(c);
		Ok(())
	}
}

pub(super) mod char_attr {
	use super::*;
	pub fn read<'a>(f: &mut impl In<'a>) -> Result<CharAttr, ReadError> {
		let a = CharId(f.u16()?);
		let b = f.u8()?;
		Ok(CharAttr(a, b))
	}

	pub fn write(f: &mut impl Out, &CharAttr(a, b): &CharAttr) -> Result<(), WriteError> {
		f.u16(a.0);
		f.u8(b);
		Ok(())
	}
}

mod file_ref {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>, arc: &GameData) -> Result<String, ReadError> {
		Ok(arc.name(f.u32()?)?.to_owned())
	}

	pub(super) fn write(f: &mut impl Out, arc: &GameData, v: &str) -> Result<(), WriteError> {
		f.u32(arc.index(v)?);
		Ok(())
	}
}

mod func_ref {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>) -> Result<FuncRef, ReadError> {
		let a = f.u8()? as u16;
		let b = f.u16()?;
		Ok(FuncRef(a, b))
	}

	pub(super) fn write(f: &mut impl Out, &FuncRef(a, b): &FuncRef) -> Result<(), WriteError> {
		f.u8(cast(a)?);
		f.u16(b);
		Ok(())
	}
}

mod text {
	use super::*;
	pub(super) fn read<'a>(f: &mut impl In<'a>) -> Result<Text, ReadError> {
		crate::text::Text::read(f)
	}

	pub(super) fn write(f: &mut impl Out, v: &Text) -> Result<(), WriteError> {
		crate::text::Text::write(f, v)
	}
}
