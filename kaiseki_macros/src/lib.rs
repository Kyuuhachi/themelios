#![feature(proc_macro_diagnostic)]

use either::Either;
use convert_case::{Case, Casing, Boundary};
use proc_macro::{TokenStream as TS, Diagnostic, Level};
use proc_macro2::{Span, TokenStream};
use quote::{quote, format_ident};
use syn::{
	*,
	spanned::Spanned,
	parse::{Parser, ParseStream},
	punctuated::*
};

type Label = LitInt; // replace with more complex type later

#[derive(Clone, Debug)]
struct Field {
	span: Span,
	expr: Either<Expr, Ident>,
	ty: Box<Type>,
	alias: Ident,
}

#[derive(Clone, Debug)]
struct Instruction {
	span: Span,
	lhs: Label,
	name: Ident,
	fields: Vec<Field>,
	tail: Option<Table>,
}

#[derive(Clone, Debug)]
struct Table {
	span: Span,
	expr: Ident,
	arms: Vec<Instruction>
}

// {{{1 Parsing
macro_rules! cast {
	($p:path, $e:expr) => { {
		let e = $e;
		if let $p(v) = e {
			Ok(v)
		} else {
			Err(syn::Error::new(e.span(), concat!("expected ", stringify!($p))))
		} }
	}
}

fn to_ident(path: &Path) -> Result<Ident> {
	Ok(parse_quote! { #path })
}

fn to_snake(ident: &Ident) -> Ident {
	Ident::new(
		&ident.to_string().with_boundaries(&[Boundary::LowerUpper]).to_case(Case::Snake),
		ident.span(),
	)
}

fn emit<A>(e: impl FnOnce() -> Result<A>) -> Option<A> {
	match e() {
		Ok(a) => Some(a),
		Err(e) => {
			Diagnostic::spanned(e.span().unwrap(), Level::Error, e.to_string()).emit();
			None
		},
	}
}

fn parse_fn(item: &ItemFn) -> Result<(&Ident, Table)> {
	let first_arg = item.sig.inputs.first()
		.ok_or_else(|| Error::new(item.sig.span(), "need at least one argument"))?;
	let first_arg = cast!(FnArg::Typed, first_arg)?;
	let first_arg = cast!(Pat::Ident, &*first_arg.pat)?;
	let first_arg = &first_arg.ident;

	let expr = match &item.block.stmts[..] {
		[Stmt::Expr(a)] => a,
		_ => return Err(Error::new(item.block.span(), "expected a singular Stmt::Expr"))
	};
	let expr = cast!(Expr::Match, expr)?;
	let table = parse_table(expr)?;

	Ok((first_arg, table))
}

fn parse_table(e: &ExprMatch) -> Result<Table> {
	let span = e.span();

	let expr = cast!(Expr::Path, &*e.expr)?;
	let expr = to_ident(&expr.path)?;
	let mut arms = Vec::new();
	for arm in &e.arms {
		emit(|| {
			arms.push(parse_arm(arm)?);
			Ok(())
		});
	}
	Ok(Table { span, expr, arms })
}

fn parse_arm(arm: &Arm) -> Result<Instruction> {
	let span = arm.span();

	let pat = cast!(Pat::Lit, &arm.pat)?;
	let pat = cast!(Expr::Lit, &*pat.expr)?;
	let lit = cast!(Lit::Int, &pat.lit)?;
	let lhs = lit.clone();

	let body = cast!(Expr::Call, &*arm.body)?;
	let path = cast!(Expr::Path, &*body.func)?;
	let name = to_ident(&path.path)?;

	let mut fields = Vec::new();
	let mut tail = None;

	for a in body.args.pairs() {
		emit(|| {
			match a {
				Pair::End(Expr::Match(e)) => tail = Some(parse_table(e)?),
				Pair::Punctuated(e, _) | Pair::End(e) => fields.push(parse_field(e)?),
			}
			Ok(())
		});
	}

	Ok(Instruction { span, lhs, name, fields, tail })
}

fn parse_field(expr: &Expr) -> Result<Field> {
	let span = expr.span();
	let (expr, alias) = match expr {
		Expr::Binary(ExprBinary { left, op: BinOp::Div {..}, right, ..}) => {
			let alias = cast!(Expr::Path, &**left)?;
			let alias = to_ident(&alias.path)?;
			(&**right, Some(alias))
		}
		expr => (expr, None)
	};

	let (expr, ty) = match expr {
		// I'd prefer to use Type `{expr}: ty` instead of Cast `{expr} as ty`,
		// but rust-analyzer doesn't like that at all
		Expr::Cast(ExprCast { expr, ty, ..}) => {
			let expr = cast!(Expr::Block, &**expr)?;
			(Either::Left(expr.clone().into()), ty.clone())
		}
		Expr::Path(expr) if expr.path.get_ident().is_some() => {
			let name = to_ident(&expr.path).unwrap();
			let name = to_snake(&name);
			(Either::Right(name), parse_quote! { #expr })
		}
		expr => return Err(Error::new(expr.span(), "invalid field"))
	};

	let alias = match alias {
		Some(a) => a,
		None => {
			let path = cast!(Type::Path, &*ty)?;
			let name = to_ident(&path.path)?;
			to_snake(&name)
		}
	};

	Ok(Field { span, expr, ty, alias })
}

// {{{1 Generate parse function
macro_rules! make {
	($ty:ty, $span:expr; $($tt:tt)*) => { {
		let a: $ty = parse_quote_spanned! { $span => $($tt)* };
		a
	} }
}

struct Gen {
	enum_name: String,
	variants: Vec<Variant>,
	visit_arms: Vec<Arm>,
	name_arms: Vec<Arm>,
}

impl Gen {
	fn process_table(
		&mut self,
		table: &Table,
		prefix: String,
		vars: &[(Ident, &Field)],
	) -> Expr {
		let mut arms = Vec::new();
		for arm in &table.arms {
			let lhs = &arm.lhs;
			let mut vars = Vec::from(vars);
			let mut decls = Vec::new();

			let name = format_ident!("{}{}", prefix, &arm.name);

			for field in &arm.fields {
				let field_expr = match &field.expr {
					Either::Left(expr) => expr.clone(),
					Either::Right(name) => make!(Expr, name.span(); i.#name()?),
				};
				let field_name = Ident::new(
					&format!("_{}", vars.len()),
					field_expr.span(),
				);
				decls.push(make!(Stmt, field.span; let #field_name = #field_expr;));
				vars.push((field_name, field));
			}

			let last: Expr = match &arm.tail {
				Some(tail) => self.process_table(tail, name.to_string(), &vars),
				None => {
					let names = vars.iter().map(|a|&a.0).collect::<Vec<_>>();
					let types = vars.iter().map(|a|&a.1.ty).collect::<Vec<_>>();
					let aliases = vars.iter().map(|a|&a.1.alias).collect::<Vec<_>>();
					self.variants.push(make!(Variant, arm.span; #name(#(#types),*)));
					self.visit_arms.push(make!(Arm, arm.span;
						Self::#name(#(#names),*) => {
							#(vis.#aliases(#names);)*
						}
					));
					let name_str = name.to_string();
					self.name_arms.push(make!(Arm, arm.span;
						Self::#name(#(#names),*) => #name_str,
					));
					make!(Expr, arm.span; Self::#name(#(#names),*))
				}
			};

			arms.push(make!(Arm, arm.span; #lhs => { #(#decls)* #last }))
		}

		let description = if prefix.is_empty() {
			self.enum_name.clone()
		} else {
			format!("{}::{}*", self.enum_name, prefix)
		};
		let fallback = make!(Arm, table.span; op => eyre::bail!("Unknown {}: {:02X}", #description, op));

		let name = &table.expr;
		let expr = make!(Expr, name.span(); i.#name()?);
		make!(Expr, table.span; match #expr { #(#arms),* #fallback })
	}
}

// {{{1 Main
#[proc_macro_attribute]
pub fn bytecode(attr: TS, item: TS) -> TS {
	match emit(|| run(attr.into(), item.into())) {
		Some(ts) => ts.into(),
		None => TS::new(),
	}
}

fn run(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
	let (mut the_enum, mut visit_fn, mut name_fn) = Parser::parse2(move |content: ParseStream| {
		Ok((
			content.parse::<ItemEnum>()?,
			content.parse::<ItemFn>()?,
			content.parse::<ItemFn>()?,
		))
	}, attr)?;

	let mut func = parse2::<ItemFn>(item)?;

	let visit_arg = cast!(Some, visit_fn.sig.inputs.iter().nth(1))?;
	let visit_arg = cast!(FnArg::Typed, visit_arg)?;
	let visit_arg = cast!(Pat::Ident, &*visit_arg.pat)?;
	let visit_arg = &visit_arg.ident;

	let (read_arg, table) = parse_fn(&func)?;

	let mut gen = Gen {
		enum_name: the_enum.ident.to_string(),
		variants: Vec::new(),
		visit_arms: Vec::new(),
		name_arms: Vec::new(),
	};

	let body = gen.process_table(&table, String::new(), &[]);
	func.block = Box::new(make!(Block, Span::call_site(); {
		let mut i = #read_arg;
		Ok(#body)
	}));

	the_enum.variants = gen.variants.into_iter().collect();

	let arms = gen.visit_arms;
	visit_fn.block = Box::new(make!(Block, Span::call_site(); {
		let mut vis = #visit_arg;
		match self { #(#arms)* }
	}));

	let arms = gen.name_arms;
	name_fn.block = Box::new(make!(Block, Span::call_site(); {
		match self { #(#arms)* }
	}));

	let enum_name = &the_enum.ident;
	Ok(quote! {
		#the_enum
		impl #enum_name {
			#func
			#visit_fn
			#name_fn
		}
	})
}
