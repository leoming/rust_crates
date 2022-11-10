use super::*;

use super::types::*;

fn make_result(success: &str, opts: &GenOpts) -> String {
    if opts.methodtype.is_some() {
        format!("Result<{}, tree::MethodErr>", success)
    } else if opts.crossroads {
        format!("Result<{}, dbus::MethodErr>", success)
    } else if opts.connectiontype == ConnectionType::Nonblock {
        format!("nonblock::MethodReply<{}>", success)
    } else {
        format!("Result<{}, dbus::Error>", success)
    }
}

pub (super) fn module_header(s: &mut String, opts: &GenOpts) {
    *s += &format!("// This code was autogenerated with `dbus-codegen-rust {}`, see https://github.com/diwic/dbus-rs\n", opts.command_line);
    *s += &format!("use {} as dbus;\n", opts.dbuscrate);
    *s += "#[allow(unused_imports)]\n";
    *s += &format!("use {}::arg;\n", opts.dbuscrate);
    if opts.methodtype.is_some() {
        *s += "use dbus_tree as tree;\n";
    } else if opts.crossroads {
        *s += "use dbus_crossroads as crossroads;\n";
    } else {
        *s += &format!("use {}::{};\n", opts.dbuscrate, match opts.connectiontype {
            ConnectionType::Ffidisp => "ffidisp",
            ConnectionType::Blocking => "blocking",
            ConnectionType::Nonblock => "nonblock",
        });
    }
}

fn write_method_decl(s: &mut String, m: &Method, opts: &GenOpts) -> Result<(), Box<dyn error::Error>> {
    let genvar = opts.genericvariant;
    let g: Vec<String> = if genvar {
        let mut g = vec!();
        for z in m.iargs.iter().chain(m.oargs.iter()) {
            let (_, mut z) = z.typename(genvar)?;
            g.append(&mut z);
        }
        g
    } else { vec!() };

    m.annotations.get("org.freedesktop.DBus.Deprecated").iter().for_each(|v| {
        *s += &format!("    #[deprecated(note = \"{}\")]\n", v);
    });
    *s += &format!("    fn {}{}(&{}self", m.fn_name,
        if g.len() > 0 { format!("<{}>", g.join(",")) } else { "".into() },
        if opts.crossroads { "mut " } else { "" }
    );

    for a in m.iargs.iter() {
        let t = a.typename(genvar)?.0;
        *s += &format!(", {}: {}", a.varname(), t);
    }

    let r = match m.oargs.len() {
        0 => "()".to_string(),
        1 => m.oargs[0].typename(genvar)?.0,
        _ => {
            let v: Result<Vec<String>, _> = m.oargs.iter().map(|z| z.typename(genvar).map(|t| t.0)).collect();
            format!("({})", v?.join(", "))
        }
    };
    *s += &format!(") -> {}", make_result(&r, opts));

    Ok(())
}

fn write_prop_decl(s: &mut String, p: &Prop, opts: &GenOpts, set: bool) -> Result<(), Box<dyn error::Error>> {
    p.annotations.get("org.freedesktop.DBus.Deprecated").iter().for_each(|v| {
        *s += &format!("    #[deprecated(note = \"{}\")]\n", v);
    });
    if set {
        *s += &format!("    fn {}(&self, value: {}) -> {}",
            p.set_fn_name, make_type(&p.typ, true, &mut None)?, make_result("()", opts));
    } else {
        *s += &format!("    fn {}(&self) -> {}",
            p.get_fn_name, make_result(&make_type(&p.typ, true, &mut None)?, opts));
    };
    Ok(())
}

pub (super) fn intf_name(s: &mut String, i: &Intf) -> Result<(), Box<dyn error::Error>> {
    let const_name = make_snake(&i.shortname, false).to_uppercase();
    *s += &format!("\npub const {}_NAME: &str = \"{}\";\n", const_name, i.origname);
    Ok(())
}

pub (super) fn intf(s: &mut String, i: &Intf, opts: &GenOpts) -> Result<(), Box<dyn error::Error>> {

    i.annotations.get("org.freedesktop.DBus.Deprecated").iter().for_each(|v| {
        *s += &format!("\n#[deprecated(note = \"{}\")]", v);
    });
    let iname = make_camel(&i.shortname);
    *s += &format!("\npub trait {} {{\n", iname);
    for m in &i.methods {
        write_method_decl(s, &m, opts)?;
        *s += ";\n";
    }
    for p in &i.props {
        if p.can_get() {
            write_prop_decl(s, &p, opts, false)?;
            *s += ";\n";
        }
        if p.can_set() {
            write_prop_decl(s, &p, opts, true)?;
            *s += ";\n";
        }
    }
    *s += "}\n";
    Ok(())
}


fn write_signal(s: &mut String, i: &Intf, ss: &Signal) -> Result<(), Box<dyn error::Error>> {
    let structname = format!("{}{}", make_camel(&i.shortname), make_camel(&ss.name));
    ss.annotations.get("org.freedesktop.DBus.Deprecated").iter().for_each(|v| {
        *s += &format!("\n#[deprecated(note = \"{}\")]", v);
    });
    *s += "\n#[derive(Debug)]\n";
    *s += &format!("pub struct {} {{\n", structname);
    for a in ss.args.iter() {
        *s += &format!("    pub {}: {},\n", a.varname(), a.typename(false)?.0);
    }
    *s += "}\n\n";

    *s += &format!("impl arg::AppendAll for {} {{\n", structname);
    *s += &format!("    fn append(&self, {}: &mut arg::IterAppend) {{\n", if ss.args.len() > 0 {"i"} else {"_"});
    for a in ss.args.iter() {
        *s += &format!("        arg::RefArg::append(&self.{}, i);\n", a.varname());
    }
    *s += "    }\n";
    *s += "}\n\n";

    *s += &format!("impl arg::ReadAll for {} {{\n", structname);
    *s += &format!("    fn read({}: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {{\n", if ss.args.len() > 0 {"i"} else {"_"});
    *s += &format!("        Ok({} {{\n", structname);
    for a in ss.args.iter() {
        *s += &format!("            {}: i.read()?,\n", a.varname());
    }
    *s += "        })\n";
    *s += "    }\n";
    *s += "}\n\n";

    *s += &format!("impl dbus::message::SignalArgs for {} {{\n", structname);
    *s += &format!("    const NAME: &'static str = \"{}\";\n", ss.name);
    *s += &format!("    const INTERFACE: &'static str = \"{}\";\n", i.origname);
    *s += "}\n";
    Ok(())
}

pub (super) fn signals(s: &mut String, i: &Intf) -> Result<(), Box<dyn error::Error>> {
    for ss in i.signals.iter() { write_signal(s, i, ss)?; }
    Ok(())
}

pub (super) fn prop_struct(s: &mut String, i: &Intf) -> Result<(), Box<dyn error::Error>> {
    // No point generating the properties struct if the interface has no gettable properties.
    if !i.props.iter().any(|property| property.can_get()) {
        return Ok(())
    }

    let struct_name = format!("{}Properties", make_camel(&i.shortname));
    *s += &format!(r#"
#[derive(Copy, Clone, Debug)]
pub struct {0}<'a>(pub &'a arg::PropMap);

impl<'a> {0}<'a> {{
    pub fn from_interfaces(
        interfaces: &'a ::std::collections::HashMap<String, arg::PropMap>,
    ) -> Option<Self> {{
        interfaces.get("{1}").map(Self)
    }}
"#, struct_name, i.origname);

    for p in &i.props {
        if p.can_get() {
            let rust_type = make_type(&p.typ, true, &mut None)?;
            if can_copy_type(&rust_type) {
                *s += &format!(r#"
    pub fn {}(&self) -> Option<{}> {{
        arg::prop_cast(self.0, "{}").copied()
    }}
"#, p.get_fn_name, rust_type, p.name);
            } else {
                *s += &format!(r#"
    pub fn {}(&self) -> Option<&{}> {{
        arg::prop_cast(self.0, "{}")
    }}
"#, p.get_fn_name, rust_type, p.name);
            }
        }
    }
    *s += "}\n";
    Ok(())
}

fn write_server_access(s: &mut String, i: &Intf, saccess: ServerAccess, minfo_is_ref: bool) {
    let z = if minfo_is_ref {""} else {"&"};
    match saccess {
        ServerAccess::AsRefClosure => {
            *s += &format!("        let dd = fclone({}minfo);\n", z);
            *s += "        let d = dd.as_ref();\n";
        },
        ServerAccess::RefClosure => *s += &format!("        let d = fclone({}minfo);\n", z),
        ServerAccess::MethodInfo => *s += &format!("        let d: &dyn {} = {}minfo;\n", make_camel(&i.shortname), z),
    }
}

pub (super) fn intf_client(s: &mut String, i: &Intf, opts: &GenOpts) -> Result<(), Box<dyn error::Error>> {
    let (module, proxy) = match opts.connectiontype {
        ConnectionType::Ffidisp => ("ffidisp", "ConnPath"),
        ConnectionType::Blocking => ("blocking", "Proxy"),
        ConnectionType::Nonblock => ("nonblock", "Proxy"),
    };

    if module == "nonblock" {
        *s += &format!("\nimpl<'a, T: nonblock::NonblockReply, C: ::std::ops::Deref<Target=T>> {} for {}::{}<'a, C> {{\n",
            make_camel(&i.shortname), module, proxy);
    } else if module == "blocking" {
        *s += &format!("\nimpl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target=T>> {} for {}::{}<'a, C> {{\n",
            make_camel(&i.shortname), module, proxy);
    } else {
        assert_eq!(module, "ffidisp");
        *s += &format!("\nimpl<'a, C: ::std::ops::Deref<Target=ffidisp::Connection>> {} for ffidisp::ConnPath<'a, C> {{\n",
            make_camel(&i.shortname));
    }
    for m in &i.methods {
        *s += "\n";
        write_method_decl(s, &m, opts)?;
        *s += " {\n";
        *s += &format!("        self.method_call(\"{}\", \"{}\", (", i.origname, m.name);
        for a in m.iargs.iter() {
            *s += &a.varname_maybewrap(opts.genericvariant);
            *s += ", ";
        }
        *s += "))\n";
        let needs_andthen = (m.oargs.len() == 1) || (m.oargs.iter().any(|oa| oa.can_wrap_variant(opts.genericvariant)));
        if needs_andthen {
            *s += &"            .and_then(|r: (";
            for oa in m.oargs.iter() {
                *s += &oa.typename_maybewrap(opts.genericvariant)?;
                *s += ", ";
            }
            let tuple = m.oargs.len() > 1;
            *s += &format!(")| Ok({}", if tuple { "(" } else { "" });
            for idx in 0..m.oargs.len() {
                *s += &if m.oargs[idx].can_wrap_variant(opts.genericvariant) {
                    format!("(r.{}).0, ", idx)
                } else {
                    format!("r.{}, ", idx)
                };
            }
            *s += &format!("{}))\n", if tuple { ")" } else { "" });
        }
        *s += "    }\n";
    }

    let propintf = format!("{}::stdintf::org_freedesktop_dbus::Properties", module);

    for p in i.props.iter().filter(|p| p.can_get()) {
        *s += "\n";
        write_prop_decl(s, &p, opts, false)?;
        *s += " {\n";
        *s += &format!("        <Self as {}>::get(&self, \"{}\", \"{}\")\n", propintf, i.origname, p.name);
        *s += "    }\n";
    }

    for p in i.props.iter().filter(|p| p.can_set()) {
        *s += "\n";
        write_prop_decl(s, &p, opts, true)?;
        *s += " {\n";
        *s += &format!("        <Self as {}>::set(&self, \"{}\", \"{}\", value)\n", propintf, i.origname, p.name);
        *s += "    }\n";
    }

    *s += "}\n";
    Ok(())

}

fn cr_types(args: &[Arg]) -> Result<String, Box<dyn error::Error>> {
    let mut r = String::new();
    for arg in args {
        r += &format!("{},", arg.typename_norefs()?);
    }
    Ok(r)
}

fn cr_strs(args: &[Arg]) -> String {
    args.iter().fold(String::new(), |mut ss, arg| { ss += &format!("\"{}\",", arg.name); ss })
}

fn cr_anno(a: &HashMap<String, String>, prefix: &str, suffix: &str) -> String {
    let mut r = String::new();
    for (name, value) in a.iter() {
        r.push_str(&format!("\n{}.annotate(\"{}\", \"{}\"){}", prefix, name, value, suffix));
    }
    r
}

pub (super) fn intf_cr(s: &mut String, i: &Intf) -> Result<(), Box<dyn error::Error>> {
    *s += &format!(r#"
pub fn register_{}<T>(cr: &mut crossroads::Crossroads) -> crossroads::IfaceToken<T>
where T: {} + Send + 'static
{{
    cr.register("{}", |b| {{
"#, make_snake(&i.shortname, false), make_camel(&i.shortname), i.origname);
    *s += &cr_anno(&i.annotations, "        b", ";\n");
    for z in &i.signals {
        *s += &format!("        b.signal::<({}), _>(\"{}\", ({})){};\n",
            cr_types(&z.args)?, z.name, cr_strs(&z.args), cr_anno(&z.annotations, "            ", ""));
    }
    for m in &i.methods {
        let ivars = m.iargs.iter().fold(String::new(), |mut ss, arg| { ss += &format!("{},", arg.name); ss });

        *s += &format!("        b.method(\"{}\", ({}), ({}), |_, t: &mut T, ({})| {{\n",
            m.name, cr_strs(&m.iargs), cr_strs(&m.oargs), ivars);
        *s += &format!("            t.{}({})\n", m.fn_name, ivars);
        if m.oargs.len() == 1 {
            *s += "                .map(|x| (x,))\n";
        }
        *s += &format!("        }}){};\n", cr_anno(&m.annotations, "            ", ""));
    }
    for p in &i.props {
        *s += &format!("        b.property::<{}, _>(\"{}\")", make_type(&p.typ, true, &mut None)?, p.name);
        if p.can_get() {
            *s += &format!("\n            .get(|_, t| t.{}())", p.get_fn_name);
        }
        if p.can_set() {
            // TODO: Handle EmitsChangedSignal correctly here.
            *s += &format!("\n            .set(|_, t, value| t.{}(value).map(|_| None))", p.set_fn_name);
        }
        *s += &cr_anno(&p.annotations, "            ", "");
        *s += ";\n";
    }

    *s += "    })\n}\n";
    Ok(())
}

// Should we implement this for
// 1) MethodInfo? That's the only way receiver can check Sender, etc - ServerAccess::MethodInfo
// 2) D::ObjectPath?
// 3) A user supplied struct?
// 4) Something reachable from minfo - ServerAccess::RefClosure

pub (super) fn intf_tree(s: &mut String, i: &Intf, mtype: &str, saccess: ServerAccess, genvar: bool) -> Result<(), Box<dyn error::Error>> {
    let hasf = saccess != ServerAccess::MethodInfo;
    let hasm = mtype == "MethodType";

    let treem: String = if hasm { "M".into() } else { format!("tree::{}<D>", mtype) };

    *s += &format!("\npub fn {}_server<{}{}D>(factory: &tree::Factory<{}, D>, data: D::Interface{}) -> tree::Interface<{}, D>\n",
        make_snake(&i.shortname, false), if hasf {"F, T, "} else {""}, if hasm {"M, "} else {""}, treem, if hasf {", f: F"} else {""}, treem);

    let mut wheres: Vec<String> = vec!["D: tree::DataType".into(), "D::Method: Default".into()];
    if i.props.len() > 0 {
        wheres.push("D::Property: Default".into());
    };
    if i.signals.len() > 0 {
        wheres.push("D::Signal: Default".into());
    };
    if hasm {
        wheres.push("M: MethodType<D>".into());
    };
    match saccess {
        ServerAccess::RefClosure => {
            wheres.push(format!("T: {}", make_camel(&i.shortname)));
            wheres.push(format!("F: 'static + for <'z> Fn(& 'z tree::MethodInfo<tree::{}<D>, D>) -> & 'z T", mtype));
        },
        ServerAccess::AsRefClosure => {
            wheres.push(format!("T: AsRef<dyn {}>", make_camel(&i.shortname)));
            wheres.push(format!("F: 'static + Fn(&tree::MethodInfo<tree::{}<D>, D>) -> T", mtype));
        },
        ServerAccess::MethodInfo => {},
    };
    if let ServerAccess::RefClosure | ServerAccess::AsRefClosure = saccess {
        if mtype == "MTSync" {
            wheres.push("F: Send + Sync".into());
        }
    }
    *s += "where\n";
    for w in wheres { *s += &format!("    {},\n", w); }
    *s += "{\n";

    *s += &format!("    let i = factory.interface(\"{}\", data);\n", i.origname);
    if hasf {
        *s += "    let f = ::std::sync::Arc::new(f);";
    }
    for m in &i.methods {
        if hasf {
            *s += "\n    let fclone = f.clone();\n";
        }
        *s += &format!("    let h = move |minfo: &tree::MethodInfo<{}, D>| {{\n", treem);
        if m.iargs.len() > 0 {
            *s += "        let mut i = minfo.msg.iter_init();\n";
        }
        for a in &m.iargs {
            *s += &format!("        let {}: {} = i.read()?;\n", a.varname(), a.typename(genvar)?.0);
        }
        write_server_access(s, i, saccess, true);
        let argsvar = m.iargs.iter().map(|q| q.varname()).collect::<Vec<String>>().join(", ");
        let retargs = match m.oargs.len() {
            0 => String::new(),
            1 => format!("let {} = ", m.oargs[0].varname()),
            _ => format!("let ({}) = ", m.oargs.iter().map(|q| q.varname()).collect::<Vec<String>>().join(", ")),
        };
        *s += &format!("        {}d.{}({})?;\n",
            retargs, m.fn_name, argsvar);
        *s += "        let rm = minfo.msg.method_return();\n";
        for r in &m.oargs {
            *s += &format!("        let rm = rm.append1({});\n", r.varname());
        }
        *s += "        Ok(vec!(rm))\n";
        *s += "    };\n";
        *s += &format!("    let m = factory.method{}(\"{}\", Default::default(), h);\n", if hasm {"_sync"} else {""}, m.name);
        for a in &m.iargs {
            *s += &format!("    let m = m.in_arg((\"{}\", \"{}\"));\n", a.name, a.typ);
        }
        for a in &m.oargs {
            *s += &format!("    let m = m.out_arg((\"{}\", \"{}\"));\n", a.name, a.typ);
        }
        *s +=          "    let i = i.add_m(m);\n";
    }
    for p in &i.props {
        *s += &format!("\n    let p = factory.property::<{}, _>(\"{}\", Default::default());\n", make_type(&p.typ, false, &mut None)?, p.name);
        *s += &format!("    let p = p.access(tree::Access::{});\n", match &*p.access {
            "read" => "Read",
            "readwrite" => "ReadWrite",
            "write" => "Write",
            _ => return Err(format!("Unexpected access value {}", p.access).into()),
        });
        if p.can_get() {
            if hasf {
                *s += "    let fclone = f.clone();\n";
            }
            *s += "    let p = p.on_get(move |a, pinfo| {\n";
            *s += "        let minfo = pinfo.to_method_info();\n";
            write_server_access(s, i, saccess, false);
            *s += &format!("        a.append(d.{}()?);\n", &p.get_fn_name);
            *s += "        Ok(())\n";
            *s += "    });\n";
        }
        if p.can_set() {
            if hasf {
                *s += "    let fclone = f.clone();\n";
            }
            *s += "    let p = p.on_set(move |iter, pinfo| {\n";
            *s += "        let minfo = pinfo.to_method_info();\n";
            write_server_access(s, i, saccess, false);
            *s += &format!("        d.{}(iter.read()?)?;\n", &p.set_fn_name);
            *s += "        Ok(())\n";
            *s += "    });\n";
        }
        *s +=          "    let i = i.add_p(p);\n";
    }
    for ss in &i.signals {
        *s += &format!("    let s = factory.signal(\"{}\", Default::default());\n", ss.name);
        for a in &ss.args {
            *s += &format!("    let s = s.arg((\"{}\", \"{}\"));\n", a.name, a.typ);
        }
        *s += "    let i = i.add_s(s);\n";
    }
    *s +=          "    i\n";
    *s +=          "}\n";
    Ok(())
}
