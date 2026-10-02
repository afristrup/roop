use roop_syntax::EnumDef;
use std::collections::HashMap;

pub type Enums<'a> = HashMap<&'a str, &'a EnumDef>;
