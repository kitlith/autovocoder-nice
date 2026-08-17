use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use cargo_metadata::MetadataCommand;
use oxrdf::{
    NamedOrBlankNodeRef, TermRef,
    vocab::{
        rdf,
        rdfs,
        xsd,
    },
};
use oxttl::TurtleParser;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

mod pset {
    use oxrdf::NamedNodeRef;

    pub const PRESET: NamedNodeRef<'_> =
        NamedNodeRef::new_unchecked("http://lv2plug.in/ns/ext/presets#Preset");
    pub const VALUE: NamedNodeRef<'_> =
        NamedNodeRef::new_unchecked("http://lv2plug.in/ns/ext/presets#value");
}

mod lv2 {
    use oxrdf::NamedNodeRef;
    pub const PORT: NamedNodeRef<'_> =
        NamedNodeRef::new_unchecked("http://lv2plug.in/ns/lv2core#port");
    pub const SYMBOL: NamedNodeRef<'_> =
        NamedNodeRef::new_unchecked("http://lv2plug.in/ns/lv2core#symbol");
}

struct Preset<'a> {
    label: TermRef<'a>,
    ports: Vec<Port<'a>>,
}

struct Port<'a> {
    symbol: &'a str,
    value: TermRef<'a>,
}

fn term_node(term: TermRef<'_>) -> Option<NamedOrBlankNodeRef<'_>> {
    match term {
        TermRef::NamedNode(node) => Some(NamedOrBlankNodeRef::NamedNode(node)),
        TermRef::BlankNode(node) => Some(NamedOrBlankNodeRef::BlankNode(node)),
        TermRef::Literal(_) => None,
    }
}

fn term_str(term: TermRef<'_>) -> Option<&'_ str> {
    match term {
        TermRef::Literal(lit) => Some(lit.value()),
        _ => None,
    }
}

#[derive(Default)]
enum IntHint {
    #[default]
    Integer,
    Bool,
    Enum(Ident),
}

fn term_value(term: TermRef<'_>, hint: IntHint) -> Option<TokenStream> {
    match term {
        TermRef::Literal(lit) => {
            let ty = lit.datatype();
            if ty == xsd::STRING {
                let value = lit.value();
                Some(quote! { #value })
            } else if ty == xsd::INTEGER {
                let value = lit.value().parse::<i32>().unwrap();
                Some(match hint {
                    IntHint::Integer => quote! { #value },
                    IntHint::Enum(ident) => quote! { <#ident>::from_repr(#value).unwrap() },
                    IntHint::Bool => quote! { #value != 0 },
                })
            } else if ty == xsd::DECIMAL {
                let value = lit.value().parse::<f32>().unwrap();
                Some(quote! { #value })
            } else {
                None
            }
        }
        _ => None,
    }
}

const BOOL_PARAMS: [&'static str; 8] = [
    "comp_on",
    "carrier_chorus_on",
    "output_chorus_on",
    "trem_on",
    "pre_drive_on",
    "post_drive_on",
    "sub_on",
    "crusher_on",
];

const ENUM_PARAMS: [(&'static str, &'static str); 7] = [
    ("mode", "CarrierMode"),
    ("scale", "ScaleKind"),
    ("scale_root", "ScaleRoot"),
    ("chord_type", "ChordVoicing"),
    ("pitch_algo", "PitchAlgorithm"),
    ("trem_target", "LfoTarget"),
    ("drive_mode", "DriveMode"),
];

fn main() {
    let metadata = MetadataCommand::new().exec().unwrap();

    let av_dsp = metadata
        .packages
        .iter()
        .find(|p| p.name.as_str() == "autovocoder-dsp")
        .unwrap();

    let mut preset_path = av_dsp
        .manifest_path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    preset_path.push("autovocoder-lv2/lv2/presets.ttl");

    let reader = BufReader::new(File::open(preset_path).unwrap());

    let graph = oxrdf::Graph::from_iter(TurtleParser::new().for_reader(reader).map(Result::unwrap));

    let presets: Vec<_> = graph
        .subjects_for_predicate_object(rdf::TYPE, pset::PRESET)
        .map(|preset| {
            let label = graph
                .object_for_subject_predicate(preset, rdfs::LABEL)
                .unwrap();
            let ports = graph
                .objects_for_subject_predicate(preset, lv2::PORT)
                .map(|port| {
                    let port = term_node(port).unwrap();
                    let symbol = term_str(
                        graph
                            .object_for_subject_predicate(port, lv2::SYMBOL)
                            .unwrap(),
                    )
                    .unwrap();
                    let value = graph
                        .object_for_subject_predicate(port, pset::VALUE)
                        .unwrap();
                    Port { symbol, value }
                })
                .collect();

            Preset { label, ports }
        })
        .collect();

    let len = presets.len();
    let presets = presets.iter().map(|preset| {
        let label = term_value(preset.label, IntHint::default());
        let ports = preset.ports.iter().map(|port| {
            let symbol = Ident::new(port.symbol, Span::call_site());

            let hint = if BOOL_PARAMS.contains(&port.symbol) {
                IntHint::Bool
            } else if let Some((_, ty)) = ENUM_PARAMS.iter().find(|a| a.0 == port.symbol) {
                IntHint::Enum(Ident::new(ty, Span::call_site()))
            } else {
                IntHint::Integer
            };

            let value = term_value(port.value, hint);
            quote!(#symbol: #value)
        });
        quote!((#label, AutoVocoderPreset {
            #(#ports),*
        }))
    });
    let tokens = quote! {
        pub const PRESETS: [(&'static str, AutoVocoderPreset); #len] = [
            #(#presets),*
        ];
    };

    let generated_string: String;

    #[cfg(feature = "pretty")]
    {
        let syntax_tree = syn::parse2(tokens).unwrap();
        generated_string = prettyplease::unparse(&syntax_tree);
    }
    #[cfg(not(feature = "pretty"))]
    {
        generated_string = tokens.to_string();
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("presets.rs");
    fs::write(dest_path, generated_string).unwrap();
}
