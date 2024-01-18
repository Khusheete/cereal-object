use godot::prelude::*;


#[derive(Debug)]
enum Token {
    OpenCurlyBracket,
    CloseCurlyBracket,
    OpenBracket,
    CloseBracket,

    Identifier(String),
    String(String),
    Number(String),

    Colon,
    Comma,
    // TODO: vector and stuff

    Eof,
    Error(String)
}


#[derive(Debug)]
enum MarkerType {
    Variant,
    Float32,
    Float64,
    Int32,
    Int64,
    Byte,
    String
}



pub fn parse(string: String) -> Variant {
    let string = string.as_bytes();
    let mut index = 0usize;
    let mut line = 1usize;

    return match _parse(string, &mut index, &mut line, 0) {
        Ok(var) => var,
        Err(e) => {
            godot_error!("[CerealObject] {}", e);
            Variant::nil()
        }
    }; // TODO: better error management
}


macro_rules! error {
    ($line:expr) => (
        Err(format!("Error line {}", $line))
    );
    ($line:expr, $($x:tt),+) => (
        Err(format!("Error line {}: {}", $line, format!($($x),+)))
    );
}


fn _parse(string: &[u8], index: &mut usize, line: &mut usize, depth: usize) -> Result<Variant, String> {
    if depth > crate::MAX_RECURSION_DEPTH {
        return error!(*line, "Reached max recursion depth");
    }

    match get_token(string, index, line) {
        Token::OpenCurlyBracket => return _parse_dict(string, index, line, depth + 1),
        Token::OpenBracket => return _parse_array(string, index, line, depth + 1),
        Token::String(s) => return Ok(Variant::from(s)),
        Token::Number(nb) => {
            // Look for type marker afterwards
            let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);
            
            let type_marker = match la_token {
                Token::Identifier(ident) => get_marker_type(ident.as_str()),
                _ => MarkerType::Variant
            };

            match type_marker {
                MarkerType::Byte    => {
                    *index = la_index;
                    *line  = la_line;
                    match nb.parse::<u8>() {
                        Ok(value) => return Ok(Variant::from(value)),
                        Err(err) =>
                            return error!(*line, "Malformed Byte: {} ({})", nb, err)
                    }
                },
                MarkerType::Float32 => {
                    *index = la_index;
                    *line  = la_line;
                    match nb.parse::<f32>() {
                        Ok(value) => return Ok(Variant::from(value)),
                        Err(err) =>
                            return error!(*line, "Malformed Float32: {} ({})", nb, err)
                    }
                },
                MarkerType::Float64 => {
                    *index = la_index;
                    *line  = la_line;
                    match nb.parse::<f64>() {
                        Ok(value) => return Ok(Variant::from(value)),
                        Err(err) =>
                            return error!(*line, "Malformed Float64: {} ({})", nb, err)
                    }
                },
                MarkerType::Int32   => {
                    *index = la_index;
                    *line  = la_line;
                    match nb.parse::<i32>() {
                        Ok(value) => return Ok(Variant::from(value)),
                        Err(err) =>
                            return error!(*line, "Malformed Int32: {} ({})", nb, err)
                    }
                },
                MarkerType::Int64   => {
                    *index = la_index;
                    *line  = la_line;
                    match nb.parse::<i64>() {
                        Ok(value) => return Ok(Variant::from(value)),
                        Err(err) =>
                            return error!(*line, "Malformed Int64: {} ({})", nb, err)
                    }
                },

                _ => {
                    if nb.contains(".") {
                        match nb.parse::<f64>() {
                            Ok(value) => return Ok(Variant::from(value)),
                            Err(err) =>
                                return error!(*line, "Malformed Float64: {} ({})", nb, err)
                        }
                    } else {
                        match nb.parse::<i64>() {
                            Ok(value) => return Ok(Variant::from(value)),
                            Err(err) =>
                                return error!(*line, "Malformed Int64: {} ({})", nb, err)
                        }
                    }
                }
            }
        }, // TODO: do distinct types
        Token::Identifier(ident) => {
            if ident == "true" {
                return Ok(Variant::from(true));
            } else if ident == "false" {
                return Ok(Variant::from(false));
            } else if ident == "null" {
                return Ok(Variant::nil());
            } else {
                return error!(*line, "Unexpected identifier {}", ident);
            }
        },
        Token::Error(err) => {
            return error!(*line, "{}", err);
        },
        t => {
            return error!(*line, "Unexpected token: {:?}", t);
        }
    }
}


fn _parse_dict(string: &[u8], index: &mut usize, line: &mut usize, depth: usize) -> Result<Variant, String> {
    let mut first = true;
    let mut dict = Dictionary::new();
    
    loop {
        // Look for end of dict or comma
        let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

        if let Token::CloseCurlyBracket = la_token {
            *index = la_index;
            *line  = la_line;
            return Ok(Variant::from(dict));
        }

        if let Token::Comma = la_token {
            if first { return error!(la_line, "Unexpected comma"); }
            *index = la_index;
            *line = la_line;
        } else {
            if !first { return error!(la_line, "Expected comma, found: {:?}", la_token); }
        }


        // Parse next dict entry

        // Get identifier
        let ident = match get_token(string, index, line) {
            Token::Identifier(i) => i,
            Token::String(s) => s,
            token => return error!(*line, "Expected dictionary key, found: {:?}", token)
        };

        // Check for colon
        match get_token(string, index, line) {
            Token::Colon => {},
            token => return error!(*line, "Expected colon, found: {:?}", token)
        }

        // Get variant
        let variant = _parse(string, index, line, depth)?;

        dict.insert(ident, variant);

        // No longer the first pass
        first = false;
    }
}


fn _parse_array(string: &[u8], index: &mut usize, line: &mut usize, depth: usize) -> Result<Variant, String> {
    // Get the array type
    let array_type = {
        let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

        if let Token::Identifier(ident) = la_token {
            match get_marker_type(&ident) {
                MarkerType::Variant => MarkerType::Variant,
                marker => {
                    // We have a marker, so update the index to go after it
                    *index = la_index;
                    *line  = la_line;
                    // Check if we have a comma after
                    // if we do, go after the comma
                    if let (Token::Comma, la_index, la_line) = lookahead_token(string, *index, *line) {
                        *index = la_index;
                        *line  = la_line;
                    }

                    marker
                }
            }
        } else {
            MarkerType::Variant
        }
    };
    

    // Macro to define parsing code for each array
    macro_rules! parse_typed_array {
        ($array_type:ident, $godot_type:ident, $rust_type:ident) => {{
            let mut arr = $array_type::new();
            let mut first = true;

            loop {
                // Look for the end of the array or for comma
                let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

                if let Token::CloseBracket = la_token {
                    *index = la_index;
                    *line = la_line;
                    return Ok(Variant::from(arr));
                }

                if let Token::Comma = la_token {
                    if first { return error!(la_line, "Unexpected comma"); }
                    *index = la_index;
                    *line = la_line;
                } else {
                    if !first { return error!(la_line, "Expected comma, found: {:?}", la_token); }
                }

                // Add variant to array
                let variant = _parse(string, index, line, depth)?;
                
                
                if variant.get_type() == VariantType::$godot_type {
                    arr.push($rust_type::from_variant(&variant));
                } else {
                    return error!(*line, "Expected {}, found {}", (stringify!($godot_type)), variant);
                }

                // No longer the first pass
                first = false;
            }
        }};
    }


    // Parse the array
    match array_type {
        MarkerType::Variant => {
            let mut arr = Array::<Variant>::new();
            let mut first = true; 

            loop {
                // Look for array end
                let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

                if let Token::CloseBracket = la_token {
                    *index = la_index;
                    *line = la_line;
                    return Ok(Variant::from(arr));
                }

                // Check for comma between array members
                if let Token::Comma = la_token {
                    if first { return error!(la_line, "Unexpected comma"); }
                    *index = la_index;
                    *line = la_line;
                } else {
                    if !first { return error!(la_line, "Expected comma, found: {:?}", la_token); }
                }


                // Add variant to array
                let variant = _parse(string, index, line, depth)?;

                arr.push(variant);

                // This is no longer the first element in the array
                first = false;
            }
        },
        MarkerType::Float32 => parse_typed_array!(PackedFloat32Array, Float, f32),
        MarkerType::Float64 => parse_typed_array!(PackedFloat64Array, Float, f64),
        MarkerType::Int32 => parse_typed_array!(PackedInt32Array, Int, i32),
        MarkerType::Int64 => parse_typed_array!(PackedInt64Array, Int, i64),
        MarkerType::Byte => parse_typed_array!(PackedByteArray, Int, u8),
        MarkerType::String => parse_typed_array!(PackedStringArray, String, GString),
    }
}


fn lookahead_token(string: &[u8], index: usize, line: usize) -> (Token, usize, usize) {
    let mut lookahead_index = index;
    let mut lookahead_line = line;
    let token = get_token(string, &mut lookahead_index, &mut lookahead_line);
    return (token, lookahead_index, lookahead_line);
}


fn get_token(string: &[u8], index: &mut usize, line: &mut usize) -> Token {
    while !_is_eof(string, *index) {
        match string[*index] {
            b'\n' => {*index += 1; *line += 1;},
            b'{'  => {*index += 1; return Token::OpenCurlyBracket},
            b'}'  => {*index += 1; return Token::CloseCurlyBracket},
            b'['  => {*index += 1; return Token::OpenBracket},
            b']'  => {*index += 1; return Token::CloseBracket},
            b':'  => {*index += 1; return Token::Colon},
            b','  => {*index += 1; return Token::Comma},
            b'"' | b'\''  => {
                let double_quote = string[*index] == b'"';
                *index += 1;
                let mut value = Vec::<u8>::new();

                loop {
                    if _is_eof(string, *index) {
                        return Token::Error(format!("Unterminated string"));
                    } else if (double_quote && string[*index] == b'"')
                            || (!double_quote && string[*index] == b'\'') {
                        *index += 1;
                        break;
                    } else if string[*index] == b'\\' {
                        *index += 1;
                        if _is_eof(string, *index) {
                            return Token::Error(format!("Unterminated string"));
                        }
                        match string[*index] {
                            b't' => value.push(b'\t'),
                            b'n' => value.push(b'\n'),
                            b'r' => value.push(b'\r'),
                            b'\n' => { value.push(b'\n'); *line += 1 } // TODO: implement unicode character
                            /*b'u' => { // Unicode escape character - see godot source code on parsing json
                                let mut res = 0u32;
                                for i in 0..4usize {
                                    if _is_eof(string, *index + i) {
                                        return Token::Error(format!("Unterminated string"));
                                    }

                                    let c = string[*index + i];
                                    if !c.is_ascii_hexdigit() {
                                        return Token::Error(format!("Malformed hex in string"));
                                    }

                                    if c.is_ascii_digit() {
                                        let c = c - b'0';
                                        res += c as u32;
                                        value.push(c);
                                    } else if c <= b'F' {
                                        let c = c - b'A' + 10u8;
                                        res += c as u32;
                                        value.push(c);
                                    } else {
                                        let c = c - b'a' + 10u8;
                                        res += c as u32;
                                        value.push(c);
                                    }
                                    res <<= 4;
                                }
                                *index += 3;

                                if res & 0xfffffc00 == 0xd800 {

                                }
                            },*/
                            c => value.push(c)
                        }
                        *index += 1;
                    } else {
                        if string[*index] == b'\n' {
                            *line += 1;
                        }
                        value.push(string[*index]);
                        *index += 1;
                    }
                }

                return Token::String(String::from_utf8(value).unwrap());
            }
            c if c <= 32 => *index += 1, // ignore whitespaces and other non printable characters
            d if d.is_ascii_digit() || d == b'-' || d == b'.' => { // some sort of number
                let start_index = *index;
                
                while !_is_eof(string, *index)
                        && (string[*index].is_ascii_digit() || string[*index] == b'-' || string[*index] == b'.'
                            || string[*index] == b'e' || string[*index] == b'E' || string[*index] == b'x'
                            || string[*index] == b'o') {
                    *index += 1;
                }

                return Token::Number(String::from(
                    unsafe { std::str::from_utf8_unchecked(&string[start_index..*index]) }
                ));
            },
            c if c.is_ascii_alphabetic() || c == b'_' => { // an identifyer
                let mut ident = Vec::<u8>::new();
                while !_is_eof(string, *index) && (string[*index].is_ascii_alphanumeric() || string[*index] == b'_') {
                    ident.push(string[*index]);
                    *index += 1;
                }
                let ident = unsafe { String::from_utf8_unchecked(ident) };

                return Token::Identifier(ident);
            }
            _ => {
                return Token::Error(format!("Unexpected character"));
            }
        }
    }
    return Token::Eof;
}


fn _is_eof(string: &[u8], index: usize) -> bool {
    index >= string.len() || string[index] == 0
}


fn get_marker_type(string: &str) -> MarkerType {
    match string {
        "Float32" | "float32" | "f32"              => MarkerType::Float32,
        "Float64" | "float64" | "f64" | "F" | "f"  => MarkerType::Float64,
        "Int32"   | "int32"   | "i32"              => MarkerType::Int32,
        "Int64"   | "int64"   | "i64" | "I" | "i"  => MarkerType::Int64,
        "Byte"    | "byte"    | "B" | "b"          => MarkerType::Byte,
        "String"  | "string"                       => MarkerType::String,
        _                                          => MarkerType::Variant
    }
}