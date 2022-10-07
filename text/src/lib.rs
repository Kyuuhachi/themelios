use ed6::scena::{Scena, Npc, Monster, Trigger, Object, FuncRef, CharId, Pos2, Pos3};
use ed6::scena::code::{InsnArgRef as I, FlatInsn, Label, Insn, Expr, ExprBinop, ExprUnop};
use ed6::text::{Text, TextSegment};

pub struct Context {
	blind: bool,
	indent: usize,
	is_line: bool,
	output: String,
}

impl Context {
	pub fn new() -> Self {
		Self {
			blind: false,
			indent: 0,
			is_line: true,
			output: String::new(),
		}
	}

	pub fn blind(mut self) -> Self {
		self.blind = true;
		self
	}

	pub fn output(self) -> String {
		self.output
	}

}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}

impl Context {
	fn write(&mut self, arg: &str) {
		assert!(!arg.contains('\n'));
		assert!(!arg.contains('\t'));
		self.write_lax(arg)
	}

	fn write_lax(&mut self, arg: &str) {
		if self.is_line {
			for _ in 0..self.indent {
				self.output.push('\t');
			}
		}
		self.output.push_str(arg);
		self.is_line = false
	}

	fn line(&mut self) {
		self.output.push('\n');
		self.is_line = true;
	}

	fn writeln(&mut self, arg: &str) {
		self.write(arg);
		self.line();
	}

	fn block(&mut self, body: impl FnOnce(&mut Self)) {
		self.writeln(":");
		self.indent += 1;
		body(self);
		self.indent -= 1;
	}
}

pub fn dump(f: &mut Context, scena: &Scena) {
	let Scena {
		dir,
		fname,
		town,
		bgm,
		entry_func,
		includes,
		ch,
		cp,
		npcs,
		monsters,
		triggers,
		objects,
		camera_angles,
		functions,
	} = scena;

	f.write("scena");
	object(f, &[
		("dir", I::String(dir)),
		("fname", I::String(fname)),
		("town", I::TownId(town)),
		("bgm", I::BgmId(bgm)),
		("entry_func", I::FuncRef(entry_func)),
	]);
	f.line();

	for (i, a) in includes.iter().enumerate() {
		f.write("include ");
		val(f, I::String(a));
		f.writeln(&format!(" // {i}"));
	}
	if !includes.is_empty() {
		f.line();
	}

	for (i, a) in ch.iter().enumerate() {
		f.write("ch ");
		val(f, I::String(a));
		f.writeln(&format!(" // {i}"));
	}
	if !ch.is_empty() {
		f.line();
	}

	for (i, a) in cp.iter().enumerate() {
		f.write("cp ");
		val(f, I::String(a));
		f.writeln(&format!(" // {i}"));
	}
	if !cp.is_empty() {
		f.line();
	}

	let mut n = 8;

	for Npc { name, pos, angle, ch, cp, flags, init, talk } in npcs {
		f.write("npc ");
		val(f, I::CharId(&CharId(n)));
		object(f, &[
			("name", I::TextTitle(name)),
			("pos", I::Pos3(pos)),
			("angle", I::Angle(angle)),
			("ch0", I::u16(&ch.0)),
			("ch1", I::u16(&ch.1)),
			("cp0", I::u16(&cp.0)),
			("cp1", I::u16(&cp.1)),
			("flags", I::CharFlags(flags)),
			("init", I::FuncRef(init)),
			("talk", I::FuncRef(talk)),
		]);
		f.line();
		n += 1;
	}

	for Monster { name, pos, angle, _1, flags, _2, battle, flag, _3 } in monsters {
		f.write("monster ");
		val(f, I::CharId(&CharId(n)));
		object(f, &[
			("name", I::TextTitle(name)),
			("pos", I::Pos3(pos)),
			("angle", I::Angle(angle)),
			("_1", I::u16(_1)),
			("flags", I::CharFlags(flags)),
			("_2", I::i32(_2)),
			("battle", I::BattleId(battle)),
			("flag", I::Flag(flag)),
			("_3", I::u16(_3)),
		]);
		f.line();
		n += 1;
	}

	for Trigger { pos1, pos2, flags, func, _1 } in triggers {
		f.write("trigger");
		object(f, &[
			("pos1", I::Pos3(pos1)),
			("pos2", I::Pos3(pos2)),
			("flags", I::u16(flags)),
			("func", I::FuncRef(func)),
			("_1", I::u16(_1)),
		]);
		f.line();
	}

	for (n, Object { pos, radius, bubble_pos, flags, func, _1 }) in objects.iter().enumerate() {
		f.write("object ");
		val(f, I::ObjectId(&(n as u16)));
		object(f, &[
			("pos", I::Pos3(pos)),
			("radius", I::u32(radius)),
			("bubble_pos", I::Pos3(bubble_pos)),
			("flags", I::u16(flags)),
			("func", I::FuncRef(func)),
			("_1", I::u16(_1)),
		]);
		f.line();
	}

	for ca in camera_angles {
		f.writeln(&format!("{:?}", ca)); // cheap cop-out, but whatever
	}
	f.line();

	for (i, func) in functions.iter().enumerate() {
		f.write("fn ");
		val(f, I::FuncRef(&FuncRef(0, i as u16)));
		flat_func(f, func);
		f.line();
	}
}

fn object(f: &mut Context, vals: &[(&str, I)]) {
	f.block(|f| {
		for (k, v) in vals {
			f.write(k);
			f.write(" ");
			val(f, *v);
			f.line();
		}
	});
}

fn flat_func(f: &mut Context, func: &[FlatInsn]) {
	f.block(|f| {
		for i in func {
			match i {
				FlatInsn::Unless(e, l) => {
					f.write("Unless ");
					val(f, I::Expr(e));
					f.write(" ");
					label(f, l);
				},
				FlatInsn::Goto(l) => {
					f.write("Goto ");
					label(f, l);
				},
				FlatInsn::Switch(e, cs, l) => {
					f.write("Switch ");
					val(f, I::Expr(e));
					f.write(" {");
					for (v, l) in cs {
						val(f, I::u16(v));
						f.write(": ");
						label(f, l);
						f.write(", ");
					}
					f.write("default: ");
					label(f, l);
					f.write("}");
				},
				FlatInsn::Insn(i) => {
					insn(f, i);
				},
				FlatInsn::Label(l) => {
					f.write("@");
					label(f, l);
				},
			}
			f.line();
		}
	});
}

fn insn(f: &mut Context, i: &Insn) {
	f.write(i.name());
	for &a in i.args().iter() {
		f.write(" ");
		val(f, a);
	}
}

fn label(f: &mut Context, l: &Label) {
	f.write(&format!("L{}", l.0));
}

fn val(f: &mut Context, a: I) {
	match a {
		I::i8(v)  => f.write(&format!("{v}")),
		I::i16(v) => f.write(&format!("{v}")),
		I::i32(v) => f.write(&format!("{v}")),
		I::u8(v)  => f.write(&format!("{v}")),
		I::u16(v) => f.write(&format!("{v}")),
		I::u32(v) => f.write(&format!("{v}")),
		I::String(v) => f.write(&format!("{v:?}")),

		I::Flag(v) => f.write(&format!("F{}", v.0)),
		I::Attr(v) => f.write(&format!("A{}", v.0)),
		I::Var(v) => f.write(&format!("V{}", v.0)),
		I::CharAttr(v) => { val(f, I::CharId(&v.0)); f.write(&format!(":{}", v.1)) },

		I::Flags(v)      => f.write(&format!("0x{:08X}", v.0)),
		I::CharFlags(v)  => f.write(&format!("0x{:04X}", v.0)),
		I::QuestFlags(v) => f.write(&format!("0x{:02X}", v.0)),
		I::Color(v)      => f.write(&format!("#{:08X}", v.0)),

		I::CharId(v)   => f.write(&format!("{v:?}")),
		I::BattleId(v) => f.write(&format!("{v:?}")),
		I::BgmId(v)    => f.write(&format!("{v:?}")),
		I::ChcpId(v)   => f.write(&format!("{v:?}")),
		I::ExitId(v)   => f.write(&format!("{v:?}")),
		I::ForkId(v)   => f.write(&format!("{v:?}")),
		I::ItemId(v)   => f.write(&format!("{v:?}")),
		I::MagicId(v)  => f.write(&format!("{v:?}")),
		I::MenuId(v)   => f.write(&format!("{v:?}")),
		I::ObjectId(v) => f.write(&format!("{v:?}")),
		I::QuestId(v)  => f.write(&format!("{v:?}")),
		I::ShopId(v)   => f.write(&format!("{v:?}")),
		I::SoundId(v)  => f.write(&format!("{v:?}")),
		I::TownId(v)   => f.write(&format!("{v:?}")),

		I::Expr(v) => expr(f, v),
		I::Fork(v) => {
			f.write("[");
			f.indent += 1;
			f.line();
			for i in v {
				insn(f, i);
				f.line();
			}
			f.indent -= 1;
			f.write("]");
		},
		I::FuncRef(v) => {
			if v.0 != 0 {
				f.write(&format!("{}", v.0))
			}
			f.write(&format!(":{}", v.1))
		},

		I::TextTitle(_) if f.blind => f.write("\"…\""),
		I::TextTitle(v) => f.write(&format!("{v:?}")),
		I::Text(_) if f.blind => f.write("{…}"),
		I::Text(v) => text(f, v),
		I::Menu(v) if f.blind => f.write("{…}"),
		I::Menu(v) => f.write(&format!("{v:?}")),

		I::Angle(v)   => f.write(&format!("{v}°")),
		I::Angle32(v) => f.write(&format!("{v}°₃₂")),
		I::Speed(v)   => f.write(&format!("{v}mm/s")),
		I::Time(v)    => f.write(&format!("{v}ms")),
		I::Time16(v)  => f.write(&format!("{v}ms₁₆")),

		I::Pos2(Pos2(x,z))   => f.write(&format!("({x}, -, {z})")),
		I::Pos3(Pos3(x,y,z)) => f.write(&format!("({x}, {y}, {z})")),
		I::RelPos3(Pos3(x,y,z)) => f.write(&format!("({x:+}, {y:+}, {z:+})")),

		I::Emote(v) => f.write(&format!("{v:?}")),
		I::Member(v) => f.write(&format!("{v:?}")),
		I::MemberAttr(v) => f.write(&format!("{v:?}")),
		I::QuestList(v) => f.write(&format!("{v:?}")),
		I::QuestTask(v) => f.write(&format!("{v:?}")),
		I::SepithElement(v) => f.write(&format!("{v:?}")),

		I::AviFileRef(v)   => f.write(&format!("{v:?}")),
		I::EffFileRef(v)   => f.write(&format!("{v:?}")),
		I::MapFileRef(v)   => f.write(&format!("{v:?}")),
		I::OpFileRef(v)    => f.write(&format!("{v:?}")),
		I::ScenaFileRef(v) => f.write(&format!("{v:?}")),
		I::VisFileRef(v)   => f.write(&format!("{v:?}")),
	}
}

fn expr(f: &mut Context, e: &Expr) {
	expr_prio(f, e, 0)
}

fn expr_prio(f: &mut Context, e: &Expr, prio: u8) {
	match e {
		Expr::Const(v)    => val(f, I::u32(v)),
		Expr::Flag(v)     => val(f, I::Flag(v)),
		Expr::Var(v)      => val(f, I::Var(v)),
		Expr::Attr(v)     => val(f, I::Attr(v)),
		Expr::CharAttr(v) => val(f, I::CharAttr(v)),
		Expr::Rand        => f.write("Rand"),

		Expr::Binop(op, a, b) => {
			let (text, prio2) = binop(*op);
			if prio2 < prio { f.write("("); }
			expr_prio(f, a, prio2);
			f.write(" ");
			f.write(text);
			f.write(" ");
			expr_prio(f, b, prio2+1);
			if prio2 < prio { f.write(")"); }
		},
		Expr::Unop(op, e) => {
			let (text, is_assign) = unop(*op);
			if is_assign {
				f.write(text);
				f.write(" ");
				expr_prio(f, e, 0);
			} else {
				f.write(text);
				expr_prio(f, e, 100);
			}
		},
		Expr::Insn(i) => insn(f, i),
	}
}

fn binop(op: ExprBinop) -> (&'static str, u8) {
	match op {
		ExprBinop::Eq      => ("==", 4),
		ExprBinop::Ne      => ("!=", 4),
		ExprBinop::Lt      => ("<",  4),
		ExprBinop::Gt      => (">",  4),
		ExprBinop::Le      => ("<=", 4),
		ExprBinop::Ge      => (">=", 4),
		ExprBinop::BoolAnd => ("&&", 3),
		ExprBinop::And     => ("&", 3),
		ExprBinop::Or      => ("|", 1),
		ExprBinop::Add     => ("+", 5),
		ExprBinop::Sub     => ("-", 5),
		ExprBinop::Xor     => ("^", 2),
		ExprBinop::Mul     => ("*", 6),
		ExprBinop::Div     => ("/", 6),
		ExprBinop::Mod     => ("%", 6),
	}
}

fn unop(op: ExprUnop) -> (&'static str, bool) {
	match op {
		ExprUnop::Not    => ("!", false),
		ExprUnop::Neg    => ("-", false),
		ExprUnop::Inv    => ("~", false),
		ExprUnop::Ass    => ("=",  true),
		ExprUnop::MulAss => ("*=", true),
		ExprUnop::DivAss => ("/=", true),
		ExprUnop::ModAss => ("%=", true),
		ExprUnop::AddAss => ("+=", true),
		ExprUnop::SubAss => ("-=", true),
		ExprUnop::AndAss => ("&=", true),
		ExprUnop::XorAss => ("^=", true),
		ExprUnop::OrAss  => ("|=", true),
	}
}

fn text(f: &mut Context, v: &Text) {
	f.write("{");
	f.indent += 1;
	f.line();
	for v in v.iter() {
		match v {
			TextSegment::String(s) => {
				f.write(&s.replace('{', "{{").replace('}', "{}"))
			},
			TextSegment::Line => {
				f.line()
			},
			TextSegment::Wait => {
				f.write("{wait}")
			},
			TextSegment::Page => {
				f.indent -= 1;
				f.line();
				f.write("} {");
				f.line();
				f.indent += 1;
			},
			TextSegment::_05 => {
				f.write("{05}")
			},
			TextSegment::_06 => {
				f.write("{06}")
			},
			TextSegment::Color(n) => {
				f.write(&format!("{{color {n}}}"));
			},
			TextSegment::_09 => {
				f.write("{09}")
			},
			TextSegment::Item(n) => {
				f.write("{item ");
				val(f, I::ItemId(&n));
				f.write("}");
			},
			TextSegment::Hash(h) => {
				f.write(&h.to_hash());
			},
			TextSegment::Error(s) => {
				f.write("{error");
				for b in s {
					f.write(&format!(" {b:02X}"));
				}
				f.write("}");
			},
		}
	}
	f.line();
	f.indent -= 1;
	f.write("}");
}