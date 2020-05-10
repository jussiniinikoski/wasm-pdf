// Some parts of this code picked up from:
// https://github.com/kaj/rust-pdf/

use std::collections::HashMap;

pub fn encode(text: &str) -> Vec<u8> {
    let mut result = Vec::new();
    for ch in text.chars() {
        match encode_char(ch) {
            Some(b'\\') => {
                result.push(b'\\');
                result.push(b'\\')
            }
            Some(b'(') => {
                result.push(b'\\');
                result.push(b'(')
            }
            Some(b')') => {
                result.push(b'\\');
                result.push(b')')
            }
            Some(ch) => result.push(ch),
            None => result.push(b'?'),
        }
    }
    result
}

pub fn get_code(name: &str) -> Option<u8> {
    WIN_ANSI_NAMES.get(name).cloned()
}

pub fn encode_char(ch: char) -> Option<u8> {
    WIN_ANSI_CHARS.get(&ch).cloned()
}

lazy_static! {
    static ref WIN_ANSI_CHARS: HashMap<char, u8> = {
        let mut chars = HashMap::new();
        for code in 32..255 {
            chars.insert(code as char, code);
        }
        chars.insert('€', 128);
        chars.insert('‚', 130);
        chars.insert('ƒ', 131);
        chars.insert('„', 132);
        chars.insert('…', 133);
        chars.insert('†', 134);
        chars.insert('‡', 135);
        chars.insert('ˆ', 136);
        chars.insert('‰', 137);
        chars.insert('Š', 138);
        chars.insert('‹', 139);
        chars.insert('Œ', 140);
        chars.insert('Ž', 142);
        chars.insert('‘', 145);
        chars.insert('’', 146);
        chars.insert('“', 147);
        chars.insert('”', 148);
        chars.insert('•', 149);
        chars.insert('–', 150);
        chars.insert('—', 151);
        chars.insert('˜', 152);
        chars.insert('™', 153);
        chars.insert('š', 154);
        chars.insert('›', 155);
        chars.insert('ž', 158);
        chars.insert('Ÿ', 159);
        chars
    };
}

lazy_static! {
    static ref WIN_ANSI_NAMES: HashMap<&'static str, u8> = {
        let mut names = HashMap::new();
        names.insert("space", 0x0020);
        names.insert("exclam", 0x0021);
        names.insert("quotedbl", 0x0022);
        names.insert("numbersign", 0x0023);
        names.insert("dollar", 0x0024);
        names.insert("percent", 0x0025);
        names.insert("ampersand", 0x0026);
        names.insert("quotesingle", 0x0027);
        names.insert("parenleft", 0x0028);
        names.insert("parenright", 0x0029);
        names.insert("asterisk", 0x002A);
        names.insert("plus", 0x002B);
        names.insert("comma", 0x002C);
        names.insert("hyphen", 0x002D);
        names.insert("period", 0x002E);
        names.insert("slash", 0x002F);
        names.insert("zero", 0x0030);
        names.insert("one", 0x0031);
        names.insert("two", 0x0032);
        names.insert("three", 0x0033);
        names.insert("four", 0x0034);
        names.insert("five", 0x0035);
        names.insert("six", 0x0036);
        names.insert("seven", 0x0037);
        names.insert("eight", 0x0038);
        names.insert("nine", 0x0039);
        names.insert("colon", 0x003A);
        names.insert("semicolon", 0x003B);
        names.insert("less", 0x003C);
        names.insert("equal", 0x003D);
        names.insert("greater", 0x003E);
        names.insert("question", 0x003F);
        names.insert("at", 0x0040);
        names.insert("A", 0x0041);
        names.insert("B", 0x0042);
        names.insert("C", 0x0043);
        names.insert("D", 0x0044);
        names.insert("E", 0x0045);
        names.insert("F", 0x0046);
        names.insert("G", 0x0047);
        names.insert("H", 0x0048);
        names.insert("I", 0x0049);
        names.insert("J", 0x004A);
        names.insert("K", 0x004B);
        names.insert("L", 0x004C);
        names.insert("M", 0x004D);
        names.insert("N", 0x004E);
        names.insert("O", 0x004F);
        names.insert("P", 0x0050);
        names.insert("Q", 0x0051);
        names.insert("R", 0x0052);
        names.insert("S", 0x0053);
        names.insert("T", 0x0054);
        names.insert("U", 0x0055);
        names.insert("V", 0x0056);
        names.insert("W", 0x0057);
        names.insert("X", 0x0058);
        names.insert("Y", 0x0059);
        names.insert("Z", 0x005A);
        names.insert("bracketleft", 0x005B);
        names.insert("backslash", 0x005C);
        names.insert("bracketright", 0x005D);
        names.insert("asciicircum", 0x005E);
        names.insert("underscore", 0x005F);
        names.insert("grave", 0x0060);
        names.insert("a", 0x0061);
        names.insert("b", 0x0062);
        names.insert("c", 0x0063);
        names.insert("d", 0x0064);
        names.insert("e", 0x0065);
        names.insert("f", 0x0066);
        names.insert("g", 0x0067);
        names.insert("h", 0x0068);
        names.insert("i", 0x0069);
        names.insert("j", 0x006A);
        names.insert("k", 0x006B);
        names.insert("l", 0x006C);
        names.insert("m", 0x006D);
        names.insert("n", 0x006E);
        names.insert("o", 0x006F);
        names.insert("p", 0x0070);
        names.insert("q", 0x0071);
        names.insert("r", 0x0072);
        names.insert("s", 0x0073);
        names.insert("t", 0x0074);
        names.insert("u", 0x0075);
        names.insert("v", 0x0076);
        names.insert("w", 0x0077);
        names.insert("x", 0x0078);
        names.insert("y", 0x0079);
        names.insert("z", 0x007A);
        names.insert("braceleft", 0x007B);
        names.insert("bar", 0x007C);
        names.insert("braceright", 0x007D);
        names.insert("asciitilde", 0x007E);
        names.insert("Euro", 0x0080);
        names.insert("quotesinglbase", 0x0082);
        names.insert("florin", 0x0083);
        names.insert("quotedblbase", 0x0084);
        names.insert("ellipsis", 0x0085);
        names.insert("dagger", 0x0086);
        names.insert("daggerdbl", 0x0087);
        names.insert("circumflex", 0x0088);
        names.insert("perthousand", 0x0089);
        names.insert("Scaron", 0x008A);
        names.insert("guilsinglleft", 0x008B);
        names.insert("OE", 0x008C);
        names.insert("Zcaron", 0x008E);
        names.insert("quoteleft", 0x0091);
        names.insert("quoteright", 0x0092);
        names.insert("quotedblleft", 0x0093);
        names.insert("quotedblright", 0x0094);
        names.insert("bullet", 0x0095);
        names.insert("endash", 0x0096);
        names.insert("emdash", 0x0097);
        names.insert("tilde", 0x0098);
        names.insert("trademark", 0x0099);
        names.insert("scaron", 0x009A);
        names.insert("guilsinglright", 0x009B);
        names.insert("oe", 0x009C);
        names.insert("zcaron", 0x009E);
        names.insert("Ydieresis", 0x009F);
        names.insert("exclamdown", 0x00A1);
        names.insert("cent", 0x00A2);
        names.insert("sterling", 0x00A3);
        names.insert("currency", 0x00A4);
        names.insert("yen", 0x00A5);
        names.insert("brokenbar", 0x00A6);
        names.insert("section", 0x00A7);
        names.insert("dieresis", 0x00A8);
        names.insert("copyright", 0x00A9);
        names.insert("ordfeminine", 0x00AA);
        names.insert("guillemotleft", 0x00AB);
        names.insert("logicalnot", 0x00AC);
        names.insert("registered", 0x00AE);
        names.insert("macron", 0x00AF);
        names.insert("degree", 0x00B0);
        names.insert("plusminus", 0x00B1);
        names.insert("twosuperior", 0x00B2);
        names.insert("threesuperior", 0x00B3);
        names.insert("acute", 0x00B4);
        names.insert("mu", 0x00B5);
        names.insert("paragraph", 0x00B6);
        names.insert("periodcentered", 0x00B7);
        names.insert("cedilla", 0x00B8);
        names.insert("onesuperior", 0x00B9);
        names.insert("ordmasculine", 0x00BA);
        names.insert("guillemotright", 0x00BB);
        names.insert("onequarter", 0x00BC);
        names.insert("onehalf", 0x00BD);
        names.insert("threequarters", 0x00BE);
        names.insert("questiondown", 0x00BF);
        names.insert("Agrave", 0x00C0);
        names.insert("Aacute", 0x00C1);
        names.insert("Acircumflex", 0x00C2);
        names.insert("Atilde", 0x00C3);
        names.insert("Adieresis", 0x00C4);
        names.insert("Aring", 0x00C5);
        names.insert("AE", 0x00C6);
        names.insert("Ccedilla", 0x00C7);
        names.insert("Egrave", 0x00C8);
        names.insert("Eacute", 0x00C9);
        names.insert("Ecircumflex", 0x00CA);
        names.insert("Edieresis", 0x00CB);
        names.insert("Igrave", 0x00CC);
        names.insert("Iacute", 0x00CD);
        names.insert("Icircumflex", 0x00CE);
        names.insert("Idieresis", 0x00CF);
        names.insert("Eth", 0x00D0);
        names.insert("Ntilde", 0x00D1);
        names.insert("Ograve", 0x00D2);
        names.insert("Oacute", 0x00D3);
        names.insert("Ocircumflex", 0x00D4);
        names.insert("Otilde", 0x00D5);
        names.insert("Odieresis", 0x00D6);
        names.insert("multiply", 0x00D7);
        names.insert("Oslash", 0x00D8);
        names.insert("Ugrave", 0x00D9);
        names.insert("Uacute", 0x00DA);
        names.insert("Ucircumflex", 0x00DB);
        names.insert("Udieresis", 0x00DC);
        names.insert("Yacute", 0x00DD);
        names.insert("Thorn", 0x00DE);
        names.insert("germandbls", 0x00DF);
        names.insert("agrave", 0x00E0);
        names.insert("aacute", 0x00E1);
        names.insert("acircumflex", 0x00E2);
        names.insert("atilde", 0x00E3);
        names.insert("adieresis", 0x00E4);
        names.insert("aring", 0x00E5);
        names.insert("ae", 0x00E6);
        names.insert("ccedilla", 0x00E7);
        names.insert("egrave", 0x00E8);
        names.insert("eacute", 0x00E9);
        names.insert("ecircumflex", 0x00EA);
        names.insert("edieresis", 0x00EB);
        names.insert("igrave", 0x00EC);
        names.insert("iacute", 0x00ED);
        names.insert("icircumflex", 0x00EE);
        names.insert("idieresis", 0x00EF);
        names.insert("eth", 0x00F0);
        names.insert("ntilde", 0x00F1);
        names.insert("ograve", 0x00F2);
        names.insert("oacute", 0x00F3);
        names.insert("ocircumflex", 0x00F4);
        names.insert("otilde", 0x00F5);
        names.insert("odieresis", 0x00F6);
        names.insert("divide", 0x00F7);
        names.insert("oslash", 0x00F8);
        names.insert("ugrave", 0x00F9);
        names.insert("uacute", 0x00FA);
        names.insert("ucircumflex", 0x00FB);
        names.insert("udieresis", 0x00FC);
        names.insert("yacute", 0x00FD);
        names.insert("thorn", 0x00FE);
        names.insert("ydieresis", 0x00FF);
        names
    };
}
