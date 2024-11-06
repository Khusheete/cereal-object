use godot::prelude::*;


#[derive(Debug)]
enum Token {
    OpenCurlyBracket,
    CloseCurlyBracket,
    OpenBracket,
    CloseBracket,
    OpenParenthesis,
    CloseParenthesis,

    Identifier(String),
    String(String),
    Number(String),

    Colon,
    Comma,

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
    
    String,

    Color,
    Vector2,
    Vector3,
    Vector4,
    Vector2i,
    Vector3i,
    Vector4i,

    Rect2,
    Rect2i,
    Aabb,

    Transform2D,
    Transform3D,
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


macro_rules! expect_token_error {
    ($expected_token:pat_param, $token:expr, $line:expr) => {
        error!($line, "Expected token {:?} got {:?}", (stringify!($expected_token)), $token)
    };
}


macro_rules! expect_token {
    ($expected_token:pat_param, $val:expr, $line:expr) => {
        match $val {
            $expected_token => Ok(()),
            token => expect_token_error!($expected_token, token, $line)
        }
    };
}


macro_rules! extract_token {
    ($expected_token:pat_param => $ret:expr, $val:expr, $line:expr) => {
        match $val {
            $expected_token => Ok($ret),
            token => expect_token_error!($expected_token, token, $line)
        }
    };
}


fn _parse(string: &[u8], index: &mut usize, line: &mut usize, depth: usize) -> Result<Variant, String> {
    if depth > crate::MAX_RECURSION_DEPTH {
        return error!(*line, "Reached max recursion depth");
    }

    match get_token(string, index, line) {
        Token::OpenCurlyBracket => return _parse_dict(string, index, line, depth + 1),
        Token::OpenBracket => return _parse_array(string, index, line, depth + 1),
        Token::String(s) => return Ok(Variant::from(s)),
        Token::Number(nb) => _get_number(nb, string, index, line),
        Token::Identifier(ident) => {
            match ident.as_str() {
                "true"             => Ok(Variant::from(true)),
                "false"            => Ok(Variant::from(false)),
                "null"             => Ok(Variant::nil()),
                _ => match get_marker_type(&ident) {
                    MarkerType::Color => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let r = u8::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let g = u8::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let b = u8::from_variant(&_parse_number(string, index, line)?);
                        
                        // Either get the alpha or a closing parenthesis
                        let a = match get_token(string, index, line) {
                            Token::Comma => {
                                let a = u8::from_variant(&_parse_number(string, index, line)?);
                                expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                                a
                            },
                            Token::CloseParenthesis => 255u8,                   
                            token => error!(line, "Expected Comma or CloseParenthesis token, got: {:?}", token)?
                        };
                        Ok(Variant::from(Color::from_rgba8(r, g, b, a)))
                    },
                    MarkerType::Vector2 => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector2::new(x, y)))
                    },
                    MarkerType::Vector3 => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector3::new(x, y, z)))
                    },
                    MarkerType::Vector4 => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let w = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector4::new(x, y, z, w)))
                    },
                    MarkerType::Vector2i => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector2i::new(x, y)))
                    },
                    MarkerType::Vector3i => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector3i::new(x, y, z)))
                    },
                    MarkerType::Vector4i => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let w = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Vector4i::new(x, y, z, w)))
                    },
                    MarkerType::Rect2 => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let w = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let h = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Rect2::new(Vector2::new(x, y), Vector2::new(w, h))))
                    },
                    MarkerType::Rect2i => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let w = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let h = i32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Rect2i::new(Vector2i::new(x, y), Vector2i::new(w, h))))
                    },
                    MarkerType::Aabb => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let w = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let h = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let l = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Aabb::new(Vector3::new(x, y, z), Vector3::new(w, h, l))))
                    },
                    MarkerType::Transform2D => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x0 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let x1 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let y0 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y1 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let ox = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let oy = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Transform2D::from_cols(Vector2::new(x0, x1), Vector2::new(y0, y1), Vector2::new(ox, oy))))
                    },
                    MarkerType::Transform3D => {
                        expect_token!(Token::OpenParenthesis, get_token(string, index, line), *line)?;
                        let x0 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let x1 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let x2 = f32::from_variant(&_parse_number(string, index, line)?);

                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y0 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y1 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let y2 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let z0 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z1 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let z2 = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;

                        let ox = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let oy = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::Comma, get_token(string, index, line), *line)?;
                        let oz = f32::from_variant(&_parse_number(string, index, line)?);
                        expect_token!(Token::CloseParenthesis, get_token(string, index, line), *line)?;
                        Ok(Variant::from(Transform3D::from_cols(
                            Vector3::new(x0, x1, x2),
                            Vector3::new(y0, y1, y2),
                            Vector3::new(z0, z1, z2),
                            Vector3::new(ox, oy, oz)
                        )))
                    },
                    _ => error!(*line, "Unexpected identifier {}", ident)
                }
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


macro_rules! _parse_collection {
    (Variant; $closing_token:ident) => {
        |string: &[u8], index: &mut usize, line: &mut usize, depth: usize| {
        let mut collection = Array::<Variant>::new();
        let mut first = true;

        loop {
            // Look for the ending token or comma
            let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

            if let Token::$closing_token = la_token {
                *index = la_index;
                *line = la_line;
                break Ok(Variant::from(collection));
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
            collection.push(variant);

            // No longer the first pass
            first = false;
        }
    }};
    ($array_type:ident, $godot_type:ident, $rust_type:ident; $closing_token:ident) => {
        |string: &[u8], index: &mut usize, line: &mut usize, depth: usize| {
        let mut collection = $array_type::new();
        let mut first = true;

        loop {
            // Look for the ending token or comma
            let (la_token, la_index, la_line) = lookahead_token(string, *index, *line);

            if let Token::$closing_token = la_token {
                *index = la_index;
                *line = la_line;
                break Ok(Variant::from(collection));
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
                collection.push($rust_type::from_variant(&variant));
            } else {
                return error!(*line, "Expected {}, found {}", (stringify!($godot_type)), variant);
            }

            // No longer the first pass
            first = false;
        }
    }};
}


fn _parse_number(string: &[u8], index: &mut usize, line: &mut usize) -> Result<Variant, String> {
    extract_token!(
        Token::Number(nb) => _get_number(nb, string, index, line)?,
        get_token(string, index, line), *line
    )
}


fn _get_number(nb_part: String, string: &[u8], index: &mut usize, line: &mut usize) -> Result<Variant, String> {
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
            match nb_part.parse::<u8>() {
                Ok(value) => return Ok(Variant::from(value)),
                Err(err) =>
                    return error!(*line, "Malformed Byte: {} ({})", nb_part, err)
            }
        },
        MarkerType::Float32 => {
            *index = la_index;
            *line  = la_line;
            match nb_part.parse::<f32>() {
                Ok(value) => return Ok(Variant::from(value)),
                Err(err) =>
                    return error!(*line, "Malformed Float32: {} ({})", nb_part, err)
            }
        },
        MarkerType::Float64 => {
            *index = la_index;
            *line  = la_line;
            match nb_part.parse::<f64>() {
                Ok(value) => return Ok(Variant::from(value)),
                Err(err) =>
                    return error!(*line, "Malformed Float64: {} ({})", nb_part, err)
            }
        },
        MarkerType::Int32   => {
            *index = la_index;
            *line  = la_line;
            match nb_part.parse::<i32>() {
                Ok(value) => return Ok(Variant::from(value)),
                Err(err) =>
                    return error!(*line, "Malformed Int32: {} ({})", nb_part, err)
            }
        },
        MarkerType::Int64   => {
            *index = la_index;
            *line  = la_line;
            match nb_part.parse::<i64>() {
                Ok(value) => return Ok(Variant::from(value)),
                Err(err) =>
                    return error!(*line, "Malformed Int64: {} ({})", nb_part, err)
            }
        },

        _ => {
            if nb_part.contains(".") {
                match nb_part.parse::<f64>() {
                    Ok(value) => return Ok(Variant::from(value)),
                    Err(err) =>
                        return error!(*line, "Malformed Float64: {} ({})", nb_part, err)
                }
            } else {
                match nb_part.parse::<i64>() {
                    Ok(value) => return Ok(Variant::from(value)),
                    Err(err) =>
                        return error!(*line, "Malformed Int64: {} ({})", nb_part, err)
                }
            }
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


    // Parse the array
    match array_type {
        MarkerType::Variant => _parse_collection!(Variant; CloseBracket)(string, index, line, depth),
        MarkerType::Float32 => _parse_collection!(PackedFloat32Array, Float, f32; CloseBracket)(string, index, line, depth),
        MarkerType::Float64 => _parse_collection!(PackedFloat64Array, Float, f64; CloseBracket)(string, index, line, depth),
        MarkerType::Int32 => _parse_collection!(PackedInt32Array, Int, i32; CloseBracket)(string, index, line, depth),
        MarkerType::Int64 => _parse_collection!(PackedInt64Array, Int, i64; CloseBracket)(string, index, line, depth),
        MarkerType::Byte => _parse_collection!(PackedByteArray, Int, u8; CloseBracket)(string, index, line, depth),
        MarkerType::String => _parse_collection!(PackedStringArray, String, GString; CloseBracket)(string, index, line, depth),
        MarkerType::Vector2 => _parse_collection!(PackedVector2Array, Vector2, Vector2; CloseBracket)(string, index, line, depth),
        MarkerType::Vector3 => _parse_collection!(PackedVector3Array, Vector3, Vector3; CloseBracket)(string, index, line, depth),
        _ => error!(*line, "Unsupported array type: {:?}", array_type)
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
            b'('  => {*index += 1; return Token::OpenParenthesis},
            b')'  => {*index += 1; return Token::CloseParenthesis},
            b':'  => {*index += 1; return Token::Colon},
            b',' | b';'  => {*index += 1; return Token::Comma}, // Commas and semicolons are considered the same
            b'"' | b'\'' => {
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
        "Float32"     | "float32"     | "f32"             => MarkerType::Float32,
        "Float64"     | "float64"     | "f64" | "F" | "f" => MarkerType::Float64,
        "Int32"       | "int32"       | "i32"             => MarkerType::Int32,
        "Int64"       | "int64"       | "i64" | "I" | "i" => MarkerType::Int64,
        "Byte"        | "byte"        | "B"   | "b"       => MarkerType::Byte,
        "String"      | "string"                          => MarkerType::String,
        "Color"       | "color"                           => MarkerType::Color,
        "Vector2"     | "vector2"     | "vec2"            => MarkerType::Vector2,
        "Vector3"     | "vector3"     | "vec3"            => MarkerType::Vector3,
        "Vector4"     | "vector4"     | "vec4"            => MarkerType::Vector4,
        "Vector2i"    | "vector2i"    | "vec2i"           => MarkerType::Vector2i,
        "Vector3i"    | "vector3i"    | "vec3i"           => MarkerType::Vector3i,
        "Vector4i"    | "vector4i"    | "vec4i"           => MarkerType::Vector4i,
        "Rect2"       | "rect2"                           => MarkerType::Rect2,
        "Rect2i"      | "rect2i"                          => MarkerType::Rect2i,
        "AABB"        | "Aabb"        | "aabb"            => MarkerType::Aabb,
        "Transform2D" | "transform2d" | "trans2d"         => MarkerType::Transform2D,
        "Transform3D" | "transform3d" | "trans3d"         => MarkerType::Transform3D,
        _                                                 => MarkerType::Variant
    }
}