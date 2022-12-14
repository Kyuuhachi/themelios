use std::collections::{HashMap, HashSet};

use super::{FlatInsn, Insn, Expr, Label};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeInsn {
	If(Vec<(Option<Expr>, Vec<TreeInsn>)>),
	Switch(Expr, Vec<(Option<u16>, Vec<TreeInsn>)>),
	While(Expr, Vec<TreeInsn>),
	Break,
	Continue,
	Insn(Insn),
}

type Range = std::ops::Range<usize>;

#[derive(Debug, thiserror::Error)]
pub enum Error<'a> {
	#[error("could not find label {label:?} in {range:?}")]
	MissingLabel { label: &'a Label, range: Range },
	#[error("unexpected jump to {label:?}")]
	UnexpectedJump { label: &'a Label },
	#[error("{range:?}{} » {next}", brk.map_or(String::new(), |l| format!(":{l:?}")))]
	Block { range: Range, brk: Option<&'a Label>, next: Box<Error<'a>>},
}

#[derive(derive_more::Deref)]
struct Context<'a> {
	#[deref]
	insns: &'a [FlatInsn],
	labels: HashMap<&'a Label, usize>,
}

impl<'a> Context<'a> {
	fn new(insns: &'a [FlatInsn]) -> Self {
		let labels = insns.iter().enumerate().filter_map(|(i, insn)| {
			match insn {
				FlatInsn::Label(l) => Some((l, i)),
				_ => None
			}
		}).collect();
		Context { insns, labels }
	}

	fn label(&self, range: Range, label: &'a Label) -> Result<usize, Error<'a>> {
		self.labels.get(&label)
			.filter(|a| (range.start..=range.end).contains(a))
			.copied()
			.ok_or(Error::MissingLabel { label, range })
	}
}

pub fn decompile(insns: &[FlatInsn]) -> Result<Vec<TreeInsn>, Error> {
	let ctx = Context::new(insns);
	block(&ctx, &mut 0, ctx.len(), None, None)
}

fn block<'a>(ctx: &Context<'a>, pos: &mut usize, end: usize, cont: Option<&'a Label>, brk: Option<&'a Label>) -> Result<Vec<TreeInsn>, Error<'a>> {
	let (body, jump) = block_partial(ctx, pos, end, cont, brk)?;
	if let Some(label) = jump {
		Err(Error::UnexpectedJump { label })
	} else {
		Ok(body)
	}
}

fn block_partial<'a>(ctx: &Context<'a>, pos: &mut usize, end: usize, cont: Option<&'a Label>, brk: Option<&'a Label>) -> Result<(Vec<TreeInsn>, Option<&'a Label>), Error<'a>> {
	let range = *pos..end;
	block0(ctx, pos, end, cont, brk)
		.map_err(|e| Error::Block { range, brk, next: Box::new(e) })
}

fn block0<'a>(ctx: &Context<'a>, pos: &mut usize, end: usize, cont: Option<&'a Label>, brk: Option<&'a Label>) -> Result<(Vec<TreeInsn>, Option<&'a Label>), Error<'a>> {
	let mut out = Vec::new();
	let mut label = None;
	while *pos < end {
		let this = &ctx[*pos];
		*pos += 1;
		match this {
			FlatInsn::Unless(e, l1) => {
				let target = ctx.label(*pos..end, l1)?;

				let is_loop = matches!(
					ctx[*pos..target].last(),
					Some(FlatInsn::Goto(jump)) if Some(jump) == label,
				);

				if is_loop {
					let body = block(ctx, pos, target-1, label, Some(l1))?;
					*pos += 1;
					out.push(TreeInsn::While(e.clone(), body));
				} else {
					let (body, jump) = block_partial(ctx, pos, target, cont, brk)?;
					if *pos != target {
						return Err(Error::UnexpectedJump { label: jump.unwrap() })
					}
					if let Some(label) = jump {
						let block_end = ctx.label(*pos..end, label)?;
						let body2 = block(ctx, pos, block_end, cont, brk)?;
						let mut cases = vec![(Some(e.clone()), body)];
						match &body2[..] { // TODO poor memory management here
							[TreeInsn::If(more_cases)] => cases.extend(more_cases.iter().cloned()),
							_ => cases.push((None, body2)),
						}
						out.push(TreeInsn::If(cases));
					} else {
						let cases = vec![(Some(e.clone()), body)];
						out.push(TreeInsn::If(cases));
					}
				}
			}

			FlatInsn::Switch(e, cs, l) => {
				let brk_ = brk;
				let mut cases = cs.iter()
					.map(|(a, b)| (Some(*a), b))
					.chain(std::iter::once((None, l)))
					.collect::<Vec<_>>();
				cases.sort_by_key(|a| ctx.labels.get(a.1));

				let ends = cases.iter().map(|a| &a.1).skip(1);

				let last_case = cases.last().unwrap();
				let mut brk = None;
				for case_end in ends.clone() {
					let case_end = ctx.label(*pos..end, case_end)?;
					if let Some(FlatInsn::Goto(label)) = ctx[*pos..case_end].last() {
						if ctx.label(case_end..end, label).is_ok() {
							brk = Some(label);
						}
					}
				}

				let mut arms = Vec::new();
				for ((k, _), case_end) in cases.iter().zip(ends) {
					let case_end = ctx.label(*pos..end, case_end)?;
					arms.push((*k, block(ctx, pos, case_end, cont, brk)?));
				}

				match brk {
					Some(brk) => {
						if brk != last_case.1 {
							let the_end = ctx.label(*pos..end, brk)?;
							arms.push((last_case.0, block(ctx, pos, the_end, cont, Some(brk))?));
						}
						out.push(TreeInsn::Switch(e.clone(), arms));
					}
					None => {
						let (mut body, jump) = block_partial(ctx, pos, end, cont, None)?;
						if jump.is_some() && jump != brk_ && *pos < ctx.len() && ctx[*pos] == FlatInsn::Label(*jump.unwrap()) {
							body.push(TreeInsn::Break);
							arms.push((last_case.0, body));
							out.push(TreeInsn::Switch(e.clone(), arms));
						} else {
							out.push(TreeInsn::Switch(e.clone(), arms));
							out.extend(body);
							if jump.is_some() && jump == brk_ {
								out.push(TreeInsn::Break);
								return Ok((out, None));
							} else {
								return Ok((out, jump));
							}
						}
					}
				}
			}

			FlatInsn::Insn(i) => {
				out.push(TreeInsn::Insn(i.clone()));
			}

			FlatInsn::Goto(label) => {
				if Some(label) == brk {
					out.push(TreeInsn::Break);
				} else if Some(label) == cont {
					out.push(TreeInsn::Continue);
				} else {
					return Ok((out, Some(label)))
				}
			}

			FlatInsn::Label(l) => {
				// This may mess up if there are consecutive labels. But that just means someone else has messed up.
				label = Some(l);
			}
		}
		if !matches!(this, FlatInsn::Label(_)) {
			label = None;
		}
	}
	Ok((out, None))
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
	#[error("else clause must be last")]
	ElseNotLast,
	#[error("invalid break statement")]
	InvalidBreak,
	#[error("invalid continue statement")]
	InvalidContinue,
	#[error("duplicate key {}", key.map_or("default".to_owned(), |a| a.to_string()))]
	DuplicateCase { key: Option<u16> },
}

pub fn recompile(insns: &[TreeInsn]) -> Result<Vec<FlatInsn>, CompileError> {
	let mut out = Vec::new();
	recompile0(insns, &mut out, &mut 0, None, None)?;
	fixup_labels(&mut out);
	Ok(out)
}

fn recompile0(insns: &[TreeInsn], out: &mut Vec<FlatInsn>, count: &mut usize, cont: Option<Label>, brk: Option<Label>) -> Result<(), CompileError> {
	for i in insns {
		match i {
			TreeInsn::If(clauses) => {
				if let Some((last, clauses)) = clauses.split_last() {
					let end = Label(*count); *count += 1;
					for clause in clauses {
						let l2 = Label(*count); *count += 1;
						if let Some(e) = &clause.0 {
							out.push(FlatInsn::Unless(e.clone(), l2));
						} else {
							return Err(CompileError::ElseNotLast);
						}
						recompile0(&clause.1, out, count, cont, brk)?;
						out.push(FlatInsn::Goto(end));
						out.push(FlatInsn::Label(l2));
					}
					if let Some(e) = &last.0 {
						out.push(FlatInsn::Unless(e.clone(), end));
					}
					recompile0(&last.1, out, count, cont, brk)?;
					out.push(FlatInsn::Label(end));
				}
			}

			TreeInsn::Switch(e, clauses) => {
				let brk = Label(*count); *count += 1;
				let pos = out.len();
				let mut labels = Vec::new();
				let mut default = None;
				for arm in clauses.split_inclusive(|a| !a.1.is_empty()) {
					let label = Label(*count); *count += 1;
					out.push(FlatInsn::Label(label));
					// TODO check duplicate cases
					for case in arm {
						if let Some(key) = case.0 {
							labels.push((key, label));
						} else {
							default = Some(label);
						}
					}
					let body = &arm.last().unwrap().1;
					recompile0(body, out, count, cont, Some(brk))?;
				}
				out.insert(pos, FlatInsn::Switch(e.clone(), labels, default.unwrap_or(brk)));
				out.push(FlatInsn::Label(brk));
			}

			TreeInsn::While(e, body) => {
				let cont = Label(*count); *count += 1;
				let brk = Label(*count); *count += 1;
				out.push(FlatInsn::Label(cont));
				out.push(FlatInsn::Unless(e.clone(), brk));
				recompile0(body, out, count, Some(cont), Some(brk))?;
				out.push(FlatInsn::Goto(cont));
				out.push(FlatInsn::Label(brk));
			}

			TreeInsn::Break => {
				out.push(FlatInsn::Goto(brk.ok_or(CompileError::InvalidBreak)?));
			}

			TreeInsn::Continue => {
				out.push(FlatInsn::Goto(cont.ok_or(CompileError::InvalidBreak)?));
			}

			TreeInsn::Insn(i) => {
				out.push(FlatInsn::Insn(i.clone()));
			}
		}
	}
	Ok(())
}

/// Combines duplicate labels, removes unused ones, and orders them in a consistent order.
fn fixup_labels(insns: &mut Vec<FlatInsn>) {
	use FlatInsn as F;

	// Find referenced labels
	let mut referenced = HashSet::new();
	let mut label = |a: &Label| { referenced.insert(*a) };
	for insn in insns.iter() {
		match insn {
			F::Unless(_, target) => {
				label(target);
			},
			F::Goto(target) => {
				label(target);
			},
			F::Switch(_, branches, default) => {
				for (_, target) in branches {
					label(target);
				}
				label(default);
			}
			F::Insn(_) => {}
			F::Label(_) => {},
		}
	}

	// Collapse consecutive labels
	let mut labels = HashMap::new();
	let mut n = 0;
	let mut current = None;
	insns.retain_mut(|insn| {
		match insn {
			F::Label(l) => {
				if !referenced.contains(l) {
					false
				} else if let Some(replace) = current {
					labels.insert(*l, replace);
					false
				} else {
					let replace = Label(n);
					n += 1;
					current = Some(replace);
					labels.insert(*l, replace);
					true
				}
			},
			_ => {
				current = None;
				true
			}
		}
	});

	// Remap the collapsed consecutive ones
	let label = |a: &mut Label| { *a = labels[a] };
	for insn in insns {
		match insn {
			F::Unless(_, target) => {
				label(target);
			},
			F::Goto(target) => {
				label(target);
			},
			F::Switch(_, branches, default) => {
				for (_, target) in branches {
					label(target);
				}
				label(default);
			}
			F::Insn(_) => {}
			F::Label(l) => label(l),
		}
	}
}

pub fn fixup_eddec(vanilla: &[FlatInsn], eddec: &[FlatInsn]) -> Option<Vec<FlatInsn>> {
	use FlatInsn as F;

	let mut eddec = eddec.to_owned();

	let offset = 10000000;
	let label = |a: &mut Label| { a.0 += offset; };
	for insn in eddec.iter_mut() {
		match insn {
			F::Unless(_, target) => {
				label(target);
			},
			F::Goto(target) => {
				label(target);
			},
			F::Switch(_, branches, default) => {
				for (_, target) in branches {
					label(target);
				}
				label(default);
			}
			F::Insn(_) => {}
			F::Label(l) => label(l),
		}
	}

	let mut i = 0;
	let mut j = 0;
	let mut reachable = HashSet::new();
	let mut is_reachable = true;
	while i < vanilla.len() && j < eddec.len() {
		let v = &vanilla[i];
		match v {
			F::Label(l) => {
				if is_reachable {
					reachable.insert(*l);
				}
				is_reachable = reachable.contains(l);
			}
			F::Unless(_, target) => if is_reachable {
				reachable.insert(*target);
			}
			F::Goto(target) => if is_reachable {
				reachable.insert(*target);
			}
			F::Switch(_, branches, default) => if is_reachable {
				reachable.extend(branches.iter().map(|a| a.1));
				reachable.insert(*default);
			}
			F::Insn(_) => {}
		}
		let i0 = i;
		if is_reachable {
			match (&vanilla[i], &eddec[j]) {
				(F::Insn(e), _) if e != &Insn::Return() => i += 1,
				(_, F::Insn(e)) if e != &Insn::Return() => j += 1,

				| (F::Insn(Insn::Return()), F::Insn(Insn::Return()))
				| (F::Goto(..),   F::Goto(..))
				| (F::Unless(..), F::Unless(..))
				| (F::Switch(..), F::Switch(..)) => {
					i += 1;
					j += 1;
				},
				(F::Label(l),  F::Label(..)) => {
					eddec.insert(j, F::Label(*l));
					i += 1;
					j += 2;
				}
				(F::Label(l), _) => {
					let mut k = j;
					while k > 0 && let F::Insn(e) = &eddec[k-1] && e != &Insn::Return() {
						k -= 1;
					}
					eddec.insert(k, F::Label(*l));
					i += 1;
					j += 1;

					eddec.insert(k, F::Insn(Insn::Hcf()));
					j+=1;
					eddec.insert(j, F::Insn(Insn::Hcf()));
					j+=1;
				}

				a => { // mismatch
					println!("failure: {:?} at ({i}, {j})", a);
					// let mut ctx = super::super::text::Context::new();
					// ctx.indent += 1;
					// super::super::text::flat_func(&mut ctx, vanilla);
					// print!("{}", ctx.output);
					// println!("\n======\n");
					// let mut ctx = super::super::text::Context::new();
					// ctx.indent += 1;
					// super::super::text::flat_func(&mut ctx, eddec);
					// println!("{}\n\n\n\n", ctx.output);
					return None
				},
			}
		} else {
					eddec.insert(j, F::Insn(Insn::Hcf()));
					j += 1;
			eddec.insert(j, v.clone());
			i += 1;
			j += 1;
		}

		if i != i0 {
			match v {
				| F::Goto(..)
				| F::Switch(..)
				| F::Insn(Insn::Return()) => {
					is_reachable = false;
				}
				_ => {}
			}
		}
	}

	eddec.extend_from_slice(&vanilla[i..]);

	fixup_labels(&mut eddec);
	Some(eddec)
}

pub fn to_d2(code: &[FlatInsn], mut f: impl std::io::Write) -> std::io::Result<()> {
	fn label(l: Label, n: usize) -> String {
		format!("L{}{}", l.0, "'".repeat(n))
	}
	let mut cur_label = Label(1000000);
	let mut sublabel = 0;
	let mut reachable = false;
	for i in code {
		match i {
			FlatInsn::Unless(_, l) => {
				let l1 = label(cur_label, sublabel);
				sublabel += 1;
				let l2 = label(cur_label, sublabel);
				let l3 = label(*l, 0);
				writeln!(f, "{l1}.shape: diamond" )?;
				writeln!(f, "{l1} -> {l2}")?;
				writeln!(f, "{l1} -> {l3}: {{ target-arrowhead: {{ shape: diamond }} }}")?;
			},
			FlatInsn::Goto(l) => {
				let l1 = label(cur_label, sublabel);
				let l2 = label(*l, 0);
				writeln!(f, "{l1} -> {l2}: {{ target-arrowhead: {{ shape: diamond; style.filled: true }} }}")?;
				sublabel += 1;
				reachable = false;
			},
			FlatInsn::Switch(_, cs, l) => {
				let l1 = label(cur_label, sublabel);
				writeln!(f, "{l1}.shape: hexagon" )?;
				for (n, l) in cs {
					let l2 = label(*l, 0);
					writeln!(f, "{l1} -> {l2}: \"{n}\" {{ target-arrowhead: {{ shape: diamond }} }}" )?;
				}
				let l2 = label(*l, 0);
				writeln!(f, "{l1} -> {l2}: {{ target-arrowhead: {{ shape: diamond; style.filled: true }} }}")?;
				sublabel += 1;
				reachable = false;
			},
			FlatInsn::Insn(_) => {},
			FlatInsn::Label(l) => {
				if reachable {
					let l1 = label(cur_label, sublabel);
					let l2 = label(*l, 0);
					writeln!(f, "{l1} -> {l2}")?;
				}
				cur_label = *l;
				sublabel = 0;
				reachable = true;
			},
		}
	}
	Ok(())
}
