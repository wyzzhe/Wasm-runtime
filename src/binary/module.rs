use super::{section::SectionCode, types::FuncType};
use nom::{
    bytes::complete::{tag, take},
    number::complete::{le_u32, le_u8},
    sequence::pair,
    IResult,
};
use nom_leb128::leb128_u32;
use num_traits::FromPrimitive as _;

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub magic: String,
    pub version: u32,
    pub type_section: Option<Vec<FuncType>>,
    pub function_section: Option<Vec<u32>>,
}

// Struct Trait
impl Default for Module {
    // Self = Module
    fn default() -> Self {
        Self {
            magic: "\0asm".to_string(),
            version: 1,
            type_section: None,
            function_section: None,
        }
    }
}

// Struct Function
impl Module {
    pub fn new(input: &[u8]) -> anyhow::Result<Module> {
        let (_, module) =
            Module::decode(input).map_err(|e| anyhow::anyhow!("failed to parse wasm: {}", e))?;
        Ok(module)
    }

    fn decode(input: &[u8]) -> IResult<&[u8], Module> {
        let (input, _) = tag(b"\0asm")(input)?;
        let (input, version) = le_u32(input)?;

        let mut module = Module {
            magic: "\0asm".into(),
            version,
            ..Default::default()
        };

        let mut remaining = input;
        while !remaining.is_empty() { // 1
            match decode_section_header(remaining) {
                Ok((input, (code, size))) => {
                    let (rest, section_contents) = take(size)(input)?; // 3

                    match code {
                        SectionCode::Type => {
                            let (_, types) = decode_type_section(section_contents)?;
                            module.type_section = Some(types);
                        }
                        SectionCode::Function => {
                            let (_, func_idx_list) = decode_function_section(section_contents)?;
                            module.function_section = Some(func_idx_list);
                        }
                        _ => {}
                    };

                    remaining = rest; // 5
                }
                Err(err) => return Err(err),
            }
        }
        Ok((input, module))
    }
}

fn decode_section_header(input: &[u8]) -> IResult<&[u8], (SectionCode, u32)> {
    let (input, (code, size)) = pair(le_u8, leb128_u32)(input)?; // 1
    Ok((
        input,
        (
            // 输入code码，匹配section，01 代表 type
            SectionCode::from_u8(code).expect("unexpect section code"), // 2
            size
        ),
    ))
}

fn decode_type_section(_input: &[u8]) -> IResult<&[u8], Vec<FuncType>> {
    let func_types = vec![FuncType::default()];

    // TODO: Decoding arguments and return values

    Ok((&[], func_types))
}

fn decode_function_section(input: &[u8]) -> IResult<&[u8], Vec<u32>> {
    let mut func_idx_list = vec![];
    let (mut input, count) = leb128_u32(input)?; // 1

    for _ in 0..count { // 2
        let (rest, idx) = leb128_u32(input)?;
        func_idx_list.push(idx);
        input = rest;
    }

    Ok((&[], func_idx_list))
}

#[cfg(test)]
mod tests {
    use crate::binary::module::Module;
    use anyhow::Result;

    #[test]
    fn decode_simplest_module() -> Result<()> {
        let wasm = wat::parse_str("(module)")?;
        let module = Module::new(&wasm)?;

        assert_eq!(module, Module::default());
        Ok(())
    }
}