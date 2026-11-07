mod named {
    include!("named_colors.rs");
}
/// CSS-style selection colors accepted by the Java mod: names, RGB(A), HSL(A), hex.
pub fn parse_color(input: &str) -> Result<[u8; 4], String> {
    let s = input.trim().to_ascii_lowercase();
    let named = match s.as_str() {
        "black" => Some([0, 0, 0, 255]),
        "white" => Some([255; 4]),
        "red" => Some([255, 0, 0, 255]),
        "green" => Some([0, 128, 0, 255]),
        "blue" => Some([0, 0, 255, 255]),
        "yellow" => Some([255, 255, 0, 255]),
        "aqua" | "cyan" => Some([0, 255, 255, 255]),
        "magenta" | "fuchsia" => Some([255, 0, 255, 255]),
        "transparent" => Some([0; 4]),
        "firebrick" => Some([178, 34, 34, 255]),
        "gainsboro" => Some([220, 220, 220, 255]),
        "gray" | "grey" => Some([128, 128, 128, 255]),
        _ => None,
    };
    if let Some(c) = named {
        return Ok(c);
    }
    if let Some(hex) = s
        .strip_prefix('#')
        .or_else(|| s.strip_prefix("0x"))
        .or_else(|| {
            ([3, 4, 6, 8].contains(&s.len()) && s.chars().all(|c| c.is_ascii_hexdigit()))
                .then_some(s.as_str())
        })
    {
        let expanded = if hex.len() == 3 || hex.len() == 4 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.into()
        };
        if expanded.len() != 6 && expanded.len() != 8 {
            return Err("expected RGB or RGBA hexadecimal color".into());
        }
        let mut c = [255; 4];
        for (i, part) in expanded.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            c[i] = u8::from_str_radix(std::str::from_utf8(part).map_err(|e| e.to_string())?, 16)
                .map_err(|e| e.to_string())?;
        }
        return Ok(c);
    }
    if let Some((_, hex)) = named::NAMED.iter().find(|(name, _)| *name == s) {
        return parse_color(hex);
    }
    let (kind, values) = s.split_once('(').ok_or("invalid color")?;
    let values = values.strip_suffix(')').ok_or("unclosed color")?;
    let parts: Vec<&str> = values.split(',').map(str::trim).collect();
    if !(3..=4).contains(&parts.len()) {
        return Err("expected three or four color channels".into());
    }
    let number = |s: &str| {
        s.parse::<f32>().map_err(|e| e.to_string()).and_then(|v| {
            if v.is_finite() {
                Ok(v)
            } else {
                Err("non-finite color".into())
            }
        })
    };
    let unit = |s: &str| {
        if let Some(p) = s.strip_suffix('%') {
            number(p).map(|v| v / 100.)
        } else {
            number(s)
        }
    };
    let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    let alpha = if parts.len() == 4 {
        unit(parts[3])?
    } else {
        1.
    };
    let rgb = if kind == "hsl" || kind == "hsla" {
        let h = number(parts[0])?.rem_euclid(360.) / 60.;
        let sat = unit(parts[1])?;
        let l = unit(parts[2])?;
        // ColorFactory calls its HSV/HSB conversion HSL; preserve upstream behavior.
        let c = l * sat;
        let x = c * (1. - (h % 2. - 1.).abs());
        let m = l - c;
        let p = match h as usize {
            0 => [c, x, 0.],
            1 => [x, c, 0.],
            2 => [0., c, x],
            3 => [0., x, c],
            4 => [x, 0., c],
            _ => [c, 0., x],
        };
        p.map(|v| v + m)
    } else if kind == "rgb" || kind == "rgba" {
        let float = parts[..3].iter().any(|v| v.contains('.'));
        let mut c = [0.; 3];
        for i in 0..3 {
            c[i] = if parts[i].ends_with('%') {
                unit(parts[i])?
            } else {
                number(parts[i])? / if float { 1. } else { 255. }
            };
        }
        c
    } else {
        return Err("unknown color function".into());
    };
    Ok([byte(rgb[0]), byte(rgb[1]), byte(rgb[2]), byte(alpha)])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upstream_color_examples() {
        for s in ["rgb(0,255,255)", "00FFFF", "0FF", "#0ff", "aqua"] {
            assert_eq!(parse_color(s).unwrap(), [0, 255, 255, 255]);
        }
        assert_eq!(
            parse_color("rgba(0,255,255,0.2)").unwrap(),
            [0, 255, 255, 51]
        );
        assert_eq!(parse_color("00FFFF64").unwrap(), [0, 255, 255, 100]);
        assert_eq!(parse_color("0FF6").unwrap(), [0, 255, 255, 102]);
        assert_eq!(
            parse_color("hsla(270,100%,100%,0.5)").unwrap(),
            [128, 0, 255, 128]
        );
        assert_eq!(parse_color("aliceblue").unwrap(), [240, 248, 255, 255]);
        assert!(parse_color("rgb(NaN,0,0)").is_err());
    }
}
